//! Blulectric Engine (`redpower_2:blulectric_engine`): port of `TileBlueEngine`.
//!
//! Mirrors:
//! - Blutricity consumption at 60V threshold.
//! - Flywheel energy accumulation up to 512.0 MJ.
//! - Kinetic power emission into adjacent BuildCraft `MjBuffer` at `facing^1` on half-stroke pump tick.
//! - Pump speed acceleration from 16 down to 4 ticks per half-stroke.
//! - Redstone on/off switching and chunk reload persistence (`flyw`, `spd`, `ptk`).

use crate::core::foundation::{Packet211TileDesc, PacketError};
use crate::machine::base::{FrameSupport, MachineFlags, Rotatable, SUBID_TILE_MACHINE};
use crate::BluePowerConductor;
use rekkit_core::energy::MjBuffer;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const BLULECTRIC_ENGINE_BLOCK_ID: &str = "redpower_2:blulectric_engine";
/// Tile entity name in `GameRegistry`.
pub const BLULECTRIC_ENGINE_TE_NAME: &str = "RPBlueEngine";

/// Maximum energy stored in the internal mechanical flywheel (512.0 MJ).
pub const FLYWHEEL_CAPACITY_MJ: f64 = 512.0;
/// Minimum voltage threshold for charging (60.0 V).
pub const MIN_VOLTAGE_THRESHOLD: f64 = 60.0;
/// Base operational Blutricity draw while active (50.0 W-t).
pub const BASE_POWER_DRAW: f64 = 50.0;
/// Energy conversion ratio: 1,000 W per 1 MJ.
pub const WATTS_PER_MJ: f64 = 1000.0;
/// Maximum flywheel charge rate per tick (32.0 MJ).
pub const MAX_CHARGE_RATE_MJ: f64 = 32.0;
/// Starting pump speed (16 ticks per half-stroke, 32 ticks full cycle).
pub const INITIAL_PUMP_SPEED: u8 = 16;
/// Maximum pump acceleration (4 ticks per half-stroke, 8 ticks full cycle).
pub const MIN_PUMP_SPEED: u8 = 4;

/// Blulectric Engine state machine (`TileBlueEngine`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileBlueEngine {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub pump_tick: u8,
    pub pump_speed: u8,
    pub flywheel: f64,
    #[serde(skip)]
    pub conductor: BluePowerConductor,
}

impl Default for TileBlueEngine {
    fn default() -> Self {
        Self::new(Direction::North)
    }
}

impl TileBlueEngine {
    /// Create a new Blulectric Engine facing `rotation`.
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            pump_tick: 0,
            pump_speed: INITIAL_PUMP_SPEED,
            flywheel: 0.0,
            conductor: BluePowerConductor::new(4.0),
        }
    }

    /// Facing output direction (`Rotation ^ 1`).
    pub fn output_side(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Available energy above minimum voltage threshold (in Joules / W-t).
    pub fn available_blutricity_energy(&self, v_min: f64) -> f64 {
        if self.conductor.voltage <= v_min {
            0.0
        } else {
            0.5 * self.conductor.capacitance * (self.conductor.voltage.powi(2) - v_min.powi(2))
        }
    }

    /// Draw power (in W-t) from the conductor.
    pub fn draw_power(&mut self, watts: f64) {
        if self.conductor.voltage > 1.0 && watts > 0.0 {
            let dq = watts / self.conductor.voltage;
            self.conductor.add_charge(-dq);
        }
    }

    /// Advance one tick of engine simulation.
    ///
    /// `deliver` is called at half-stroke (`pump_tick == pump_speed`) with `(output_side, available_mj)`
    /// and should return the amount of MJ accepted by the destination.
    pub fn tick<F>(&mut self, mut deliver: F)
    where
        F: FnMut(Direction, f64) -> f64,
    {
        // 1. Voltage threshold & charged state latching
        if self.conductor.voltage < MIN_VOLTAGE_THRESHOLD {
            if self.flags.charged {
                self.flags.charged = false;
            }
        } else {
            if !self.flags.charged {
                self.flags.charged = true;
            }
            if self.flags.charged && self.flags.powered {
                if !self.flags.active {
                    self.pump_speed = INITIAL_PUMP_SPEED;
                }
                self.flags.active = true;
            }
        }

        // 2. Active pump cycle and power delivery
        if self.flags.active {
            self.pump_tick = self.pump_tick.saturating_add(1);

            // Half-stroke: deliver mechanical power to adjacent IPowerReceptor / MjBuffer at facing^1
            if self.pump_tick == self.pump_speed {
                let side = self.output_side();
                let accepted = deliver(side, self.flywheel);
                self.flywheel = (self.flywheel - accepted).max(0.0);
            }

            // Full-stroke: complete cycle, accelerate pump speed
            if self.pump_tick >= self.pump_speed * 2 {
                self.pump_tick = 0;
                if self.pump_speed > MIN_PUMP_SPEED {
                    self.pump_speed -= 1;
                }
                // Transition active state off until re-evaluated next tick
                self.flags.active = self.flags.charged && self.flags.powered;
            }

            // Flywheel energy accumulation: draws up to 32 MJ/tick (1,000 W/MJ)
            if self.flags.powered && self.flywheel < FLYWHEEL_CAPACITY_MJ {
                let headroom = FLYWHEEL_CAPACITY_MJ - self.flywheel;
                let energy_pool = 0.002 * self.available_blutricity_energy(MIN_VOLTAGE_THRESHOLD);
                let draw_mj = headroom.min(MAX_CHARGE_RATE_MJ).min(energy_pool);
                if draw_mj > 0.0 {
                    self.draw_power(WATTS_PER_MJ * draw_mj);
                    self.flywheel = (self.flywheel + draw_mj).min(FLYWHEEL_CAPACITY_MJ);
                }
            }

            // Base operational draw
            self.draw_power(BASE_POWER_DRAW);
        } else {
            self.pump_tick = 0;
        }
    }

    /// Convenience helper to deliver directly into an adjacent `MjBuffer`.
    pub fn tick_into_buffer(&mut self, dest: &mut MjBuffer) {
        self.tick(|_, avail| {
            let space = (dest.capacity_mj - dest.stored_mj).max(0.0);
            let todraw = avail.min(space).min(dest.max_in_mj);
            dest.stored_mj += todraw;
            todraw
        });
    }

    // ------------------------------------------------------------------------
    // NBT Persistence & Packet Descriptors
    // ------------------------------------------------------------------------

    pub fn to_nbt(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("rot".into(), (self.rotation as u8).to_string());
        map.insert("ps".into(), self.flags.to_bits().to_string());
        map.insert("flyw".into(), self.flywheel.to_string());
        map.insert("spd".into(), self.pump_speed.to_string());
        map.insert("ptk".into(), self.pump_tick.to_string());
        map.insert("volts".into(), self.conductor.voltage.to_string());
        map
    }

    pub fn from_nbt(map: &HashMap<String, String>) -> Self {
        let mut eng = Self::new(Direction::North);
        if let Some(r) = map.get("rot").and_then(|v| v.parse::<u8>().ok()) {
            eng.rotation = match r {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                _ => Direction::East,
            };
        }
        if let Some(ps) = map.get("ps").and_then(|v| v.parse::<u8>().ok()) {
            eng.flags = MachineFlags::from_bits(ps);
        }
        if let Some(flyw) = map.get("flyw").and_then(|v| v.parse::<f64>().ok()) {
            eng.flywheel = flyw;
        }
        if let Some(spd) = map.get("spd").and_then(|v| v.parse::<u8>().ok()) {
            eng.pump_speed = spd;
        }
        if let Some(ptk) = map.get("ptk").and_then(|v| v.parse::<u8>().ok()) {
            eng.pump_tick = ptk;
        }
        if let Some(volts) = map.get("volts").and_then(|v| v.parse::<f64>().ok()) {
            eng.conductor.voltage = volts;
        }
        eng
    }

    pub fn write_to_packet(&self) -> [u8; 4] {
        [
            self.rotation as u8,
            self.flags.to_bits(),
            self.pump_speed,
            self.pump_tick,
        ]
    }

    pub fn read_from_packet(&mut self, data: &[u8]) -> Result<(), PacketError> {
        if data.len() < 4 {
            return Err(PacketError::NotEnoughData);
        }
        self.rotation = match data[0] {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            _ => Direction::Down,
        };
        self.flags = MachineFlags::from_bits(data[1]);
        self.pump_speed = data[2];
        self.pump_tick = data[3];
        Ok(())
    }

    pub fn write_tile_packet(&self, x: i32, y: i32, z: i32) -> Packet211TileDesc {
        let payload = self.write_to_packet();
        Packet211TileDesc {
            sub_id: SUBID_TILE_MACHINE,
            x,
            y,
            z,
            body: payload.to_vec(),
        }
    }

    pub fn handle_tile_packet(&mut self, pkt: &Packet211TileDesc) -> Result<(), PacketError> {
        if pkt.sub_id != SUBID_TILE_MACHINE {
            return Err(PacketError::BadVlc);
        }
        self.read_from_packet(&pkt.body)
    }
}

impl Rotatable for TileBlueEngine {
    fn part_max_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            5
        }
    }

    fn part_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            self.rotation as u32
        }
    }

    fn set_part_rotation(&mut self, _part: u32, secondary: bool, rot: u32) {
        if !secondary {
            self.rotation = match rot {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                5 => Direction::East,
                _ => Direction::Down,
            };
        }
    }
}

impl FrameSupport for TileBlueEngine {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1], pkt[2], pkt[3]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 5 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileBlueEngine frame packet header".into());
        }
        self.read_from_packet(&data[1..5])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_initialization_and_orientation() {
        let eng = TileBlueEngine::new(Direction::North);
        assert_eq!(eng.rotation, Direction::North);
        assert_eq!(eng.output_side(), Direction::South); // facing^1
        assert_eq!(eng.pump_speed, 16);
        assert_eq!(eng.pump_tick, 0);
        assert_eq!(eng.flywheel, 0.0);
    }

    #[test]
    fn test_engine_charging_and_pump_acceleration() {
        let mut eng = TileBlueEngine::new(Direction::Up);
        eng.conductor.voltage = 100.0;
        eng.conductor.charge = 100.0;
        eng.flags.powered = true;

        let mut dest = MjBuffer::new(1000.0, 100.0, 100.0);

        // Run through full pump cycles and observe flywheel and acceleration
        for _ in 0..100 {
            eng.conductor.voltage = 100.0;
            eng.conductor.charge = 100.0;
            eng.tick_into_buffer(&mut dest);
        }

        // Engine should have accelerated pump speed down towards 4
        assert!(eng.pump_speed < 16);
        assert!(eng.pump_speed >= MIN_PUMP_SPEED);
        // Dest should have received mechanical power
        assert!(dest.stored_mj > 0.0);
    }
}
