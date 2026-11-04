//! Frame Motor (`redpower_2:frame_motor`): port of `TileMotor`.
//!
//! Mirrors:
//! - Tile entity registration name `"RPMotor"`, `BlockMachine` metadata 7.
//! - Extended ID 7.
//! - Packet synchronization (Packet 211 `subId = 7`, 4-byte payload `[rot, md, mp+1, flags]`).
//! - Frame structural movement packet (`IFrameSupport`).
//! - IRotatable rotation mechanics (primary 0..=3 face, secondary 0..=5 direction).
//! - Blutricity consumption and movement state tracking.

use super::base::{SUBID_TILE_MACHINE, Rotatable, FrameSupport};
use super::frame::{drive_side, MotorEvent, MotorState, MOVE_TICKS, MOTOR_MIN_VOLTS};
use crate::core::foundation::{Packet211TileDesc, PacketError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tile entity registration name in `GameRegistry`.
pub const MOTOR_TE_NAME: &str = "RPMotor";
/// Extended ID (`getExtendedID`).
pub const MOTOR_EXTENDED_ID: u8 = 7;
/// Block ID.
pub const MOTOR_BLOCK_ID: &str = "redpower_2:frame_motor";
/// BlockMachine metadata index.
pub const MOTOR_METADATA: u8 = 7;

/// Frame Motor Tile Entity (`TileMotor`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileMotor {
    pub rotation: i32,
    pub move_dir: i32,
    pub move_pos: i32,
    pub powered: bool,
    pub active: bool,
    pub charged: bool,
    pub link_size: i32,
    pub charge: f64,
    pub flow: f64,
}

impl Default for TileMotor {
    fn default() -> Self {
        Self::new(0)
    }
}

impl TileMotor {
    pub fn new(rotation: i32) -> Self {
        Self {
            rotation,
            move_dir: 4,
            move_pos: -1,
            powered: false,
            active: false,
            charged: false,
            link_size: -1,
            charge: 0.0,
            flow: 0.0,
        }
    }

    /// Construct with primary facing (0..=3) and secondary side (0..=5).
    pub fn with_facing(side: u8, face: u8) -> Self {
        let rot = ((side as i32 & 7) << 2) | (face as i32 & 3);
        Self::new(rot)
    }

    /// Primary face rotation value (`0..=3`).
    pub fn primary_rotation(&self) -> u8 {
        (self.rotation & 3) as u8
    }

    /// Secondary side direction value (`0..=5`).
    pub fn secondary_rotation(&self) -> u8 {
        ((self.rotation >> 2) & 7) as u8
    }

    /// Side of the motor the first connected frame sits on (`Rotation >> 2 ^ 1`).
    pub fn frame_side(&self) -> i32 {
        (self.rotation >> 2) ^ 1
    }

    /// Direction the motor pushes the frame, calculated from `rotation`.
    pub fn drive_side(&self) -> i32 {
        drive_side(self.rotation)
    }

    /// Can a frame link connect *into* this side of the motor (`TileMotor.canFrameConnectIn`).
    pub fn can_frame_connect_in(&self, dir: i32) -> bool {
        dir != self.frame_side()
    }

    /// Can a frame link connect *out* of this side of the motor (`TileMotor.canFrameConnectOut`).
    pub fn can_frame_connect_out(&self, dir: i32) -> bool {
        dir == self.frame_side()
    }

    /// Power drawn per tick while moving: `100 + 10 * LinkSize`.
    pub fn power_cost(&self) -> f64 {
        100.0 + 10.0 * f64::from(self.link_size.max(0))
    }

    /// Move progress scaled 0.0..=1.0 for rendering (`getMoveScaled`).
    pub fn move_scaled(&self) -> f32 {
        (self.move_pos.max(0) as f32) / MOVE_TICKS as f32
    }

    /// Pack status flags into single byte:
    /// Bit 0: `powered`
    /// Bit 1: `active`
    /// Bit 2: `charged`
    pub fn flags_byte(&self) -> u8 {
        (if self.powered { 1 } else { 0 })
            | (if self.active { 2 } else { 0 })
            | (if self.charged { 4 } else { 0 })
    }

    /// Unpack status flags from byte.
    pub fn set_flags_byte(&mut self, flags: u8) {
        self.powered = (flags & 1) != 0;
        self.active = (flags & 2) != 0;
        self.charged = (flags & 4) != 0;
    }

    /// Convert to lower-level `MotorState`.
    pub fn to_motor_state(&self) -> MotorState {
        MotorState {
            rotation: self.rotation,
            move_dir: self.move_dir,
            move_pos: self.move_pos,
            powered: self.powered,
            active: self.active,
            charged: self.charged,
            link_size: self.link_size,
        }
    }

    /// Sync state from lower-level `MotorState`.
    pub fn update_from_motor_state(&mut self, state: &MotorState) {
        self.rotation = state.rotation;
        self.move_dir = state.move_dir;
        self.move_pos = state.move_pos;
        self.powered = state.powered;
        self.active = state.active;
        self.charged = state.charged;
        self.link_size = state.link_size;
    }

    // ------------------------------------------------------------------------
    // Packet Synchronization (Packet 211, subId = 7)
    // ------------------------------------------------------------------------

    /// Write 4-byte payload to packet buffer: `[rot, md, mp + 1, flags]`.
    pub fn write_to_packet(&self) -> [u8; 4] {
        [
            self.rotation as u8,
            self.move_dir as u8,
            ((self.move_pos + 1) & 0xFF) as u8,
            self.flags_byte(),
        ]
    }

    /// Read 4-byte payload from packet buffer.
    pub fn read_from_packet(&mut self, data: &[u8]) -> Result<(), PacketError> {
        if data.len() < 4 {
            return Err(PacketError::Truncated);
        }
        self.rotation = data[0] as i32;
        self.move_dir = data[1] as i32;
        self.move_pos = (data[2] as i32) - 1;
        self.set_flags_byte(data[3]);
        Ok(())
    }

    /// Create full `Packet211TileDesc` for this motor at `(x, y, z)`.
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

    /// Handle incoming `Packet211TileDesc`. Returns error if subId != 7 or data is truncated.
    pub fn handle_tile_packet(&mut self, pkt: &Packet211TileDesc) -> Result<(), PacketError> {
        if pkt.sub_id != SUBID_TILE_MACHINE {
            return Err(PacketError::BadVlc);
        }
        self.read_from_packet(&pkt.body)
    }

    // ------------------------------------------------------------------------
    // NBT Persistence
    // ------------------------------------------------------------------------

    pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("rot".into(), self.rotation.to_string());
        nbt.insert("md".into(), self.move_dir.to_string());
        nbt.insert("mp".into(), self.move_pos.to_string());
        nbt.insert("ps".into(), self.flags_byte().to_string());
        nbt.insert("ls".into(), self.link_size.to_string());
        nbt.insert("charge".into(), self.charge.to_string());
        nbt.insert("flow".into(), self.flow.to_string());
    }

    pub fn read_nbt(&mut self, nbt: &HashMap<String, String>) {
        if let Some(r) = nbt.get("rot").and_then(|v| v.parse::<i32>().ok()) {
            self.rotation = r;
        }
        if let Some(md) = nbt.get("md").and_then(|v| v.parse::<i32>().ok()) {
            self.move_dir = md;
        }
        if let Some(mp) = nbt.get("mp").and_then(|v| v.parse::<i32>().ok()) {
            self.move_pos = mp;
        }
        if let Some(ps) = nbt.get("ps").and_then(|v| v.parse::<u8>().ok()) {
            self.set_flags_byte(ps);
        }
        if let Some(ls) = nbt.get("ls").and_then(|v| v.parse::<i32>().ok()) {
            self.link_size = ls;
        }
        if let Some(c) = nbt.get("charge").and_then(|v| v.parse::<f64>().ok()) {
            self.charge = c;
        }
        if let Some(f) = nbt.get("flow").and_then(|v| v.parse::<f64>().ok()) {
            self.flow = f;
        }
    }
}

impl Rotatable for TileMotor {
    fn part_max_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if self.move_pos >= 0 {
            0
        } else if secondary {
            5
        } else {
            3
        }
    }

    fn part_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            ((self.rotation >> 2) & 7) as u32
        } else {
            (self.rotation & 3) as u32
        }
    }

    fn set_part_rotation(&mut self, _part: u32, secondary: bool, rot: u32) {
        if self.move_pos < 0 {
            if secondary {
                self.rotation = (self.rotation & 3) | ((rot as i32 & 7) << 2);
            } else {
                self.rotation = (self.rotation & !3) | (rot as i32 & 3);
            }
        }
    }
}

impl FrameSupport for TileMotor {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1], pkt[2], pkt[3]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 5 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileMotor frame packet header".into());
        }
        self.read_from_packet(&data[1..5])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motor_rotation_and_screwdriver() {
        let mut motor = TileMotor::new(0);
        assert_eq!(motor.primary_rotation(), 0);
        assert_eq!(motor.secondary_rotation(), 0);

        // Rotate primary: 0 -> 1 -> 2 -> 3 -> 0
        assert!(motor.rotate_with_screwdriver(0, false));
        assert_eq!(motor.primary_rotation(), 1);
        assert!(motor.rotate_with_screwdriver(0, false));
        assert_eq!(motor.primary_rotation(), 2);
        assert!(motor.rotate_with_screwdriver(0, false));
        assert_eq!(motor.primary_rotation(), 3);
        assert!(motor.rotate_with_screwdriver(0, false));
        assert_eq!(motor.primary_rotation(), 0);

        // Rotate secondary: 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 0
        for expected in 1..=5 {
            assert!(motor.rotate_with_screwdriver(0, true));
            assert_eq!(motor.secondary_rotation(), expected);
        }
        assert!(motor.rotate_with_screwdriver(0, true));
        assert_eq!(motor.secondary_rotation(), 0);

        // When moving (move_pos >= 0), rotation is locked
        motor.move_pos = 5;
        assert!(!motor.rotate_with_screwdriver(0, false));
        assert!(!motor.rotate_with_screwdriver(0, true));
    }

    #[test]
    fn test_motor_packet_and_frame_roundtrip() {
        let mut motor = TileMotor::new(14); // rot = (3 << 2) | 2 = 14
        motor.move_dir = 2;
        motor.move_pos = 8;
        motor.powered = true;
        motor.active = true;
        motor.charged = true;

        let pkt = motor.write_tile_packet(10, 64, -5);
        assert_eq!(pkt.sub_id, SUBID_TILE_MACHINE);
        assert_eq!(pkt.x, 10);
        assert_eq!(pkt.y, 64);
        assert_eq!(pkt.z, -5);
        assert_eq!(pkt.body.len(), 4);

        let mut restored = TileMotor::new(0);
        assert!(restored.handle_tile_packet(&pkt).is_ok());
        assert_eq!(restored.rotation, 14);
        assert_eq!(restored.move_dir, 2);
        assert_eq!(restored.move_pos, 8);
        assert!(restored.powered);
        assert!(restored.active);
        assert!(restored.charged);

        // Frame packet round trip
        let frame_pkt = motor.frame_packet();
        let mut frame_restored = TileMotor::new(0);
        assert!(frame_restored.handle_frame_packet(&frame_pkt).is_ok());
        assert_eq!(frame_restored, restored);
    }

    #[test]
    fn test_motor_nbt_roundtrip() {
        let mut motor = TileMotor::new(9);
        motor.move_dir = 5;
        motor.move_pos = 12;
        motor.powered = true;
        motor.link_size = 42;
        motor.charge = 120.5;
        motor.flow = 15.2;

        let mut nbt = HashMap::new();
        motor.write_nbt(&mut nbt);

        let mut restored = TileMotor::new(0);
        restored.read_nbt(&nbt);
        assert_eq!(restored.rotation, 9);
        assert_eq!(restored.move_dir, 5);
        assert_eq!(restored.move_pos, 12);
        assert!(restored.powered);
        assert_eq!(restored.link_size, 42);
        assert_eq!(restored.charge, 120.5);
        assert_eq!(restored.flow, 15.2);
    }
}
