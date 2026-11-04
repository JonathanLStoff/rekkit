//! Blulectric fluid pump (`redpower_2:pump`): port of `TilePump` (machine panel meta 1).
//!
//! Generates +1000 outlet pressure and -1000 inlet suction across attached fluid pipes,
//! consuming blutricity (endpoint conductor, >= 60.0V required to charge).

use crate::core::math::rot_to_side;
use crate::machine::pipe::{FluidBuffer, PIPE_MAX_LEVEL};
use crate::BluePowerConductor;
use serde::{Deserialize, Serialize};

/// TilePump domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePump {
    pub rotation: i32,
    pub active: bool,
    pub charged: bool,
    pub powered: bool,
    pub pump_tick: u8,
    pub inbuf: FluidBuffer,
    pub outbuf: FluidBuffer,
    pub cond: BluePowerConductor,
}

impl TilePump {
    /// Creates a new `TilePump` with the given rotation (0..4).
    pub fn new(rotation: i32) -> Self {
        Self {
            rotation,
            active: false,
            charged: false,
            powered: false,
            pump_tick: 0,
            inbuf: FluidBuffer::default(),
            outbuf: FluidBuffer::default(),
            cond: BluePowerConductor::default(),
        }
    }

    /// `getPipeConnectableSides`: sides where fluid pipes connect.
    /// Bitmask of directions.
    pub fn pipe_connectable_sides(&self) -> u32 {
        12 << (((self.rotation ^ 1) & 1) << 1)
    }

    /// `getPipeFlangeSides`: flange sides matching connectable sides.
    pub fn pipe_flange_sides(&self) -> u32 {
        12 << (((self.rotation ^ 1) & 1) << 1)
    }

    /// Pressure at the given side.
    /// Outlet is +1000, inlet is -1000 when active; otherwise 0.
    pub fn pipe_pressure(&self, side: i32) -> i32 {
        let rt = rot_to_side(self.rotation);
        if !self.active {
            0
        } else if side == rt {
            1000
        } else if side == (rt ^ 1) {
            -1000
        } else {
            0
        }
    }

    /// Retrieves fluid buffer for side: outlet buffer at `rt`, inlet buffer at `rt ^ 1`.
    pub fn pipe_buffer(&self, side: i32) -> Option<&FluidBuffer> {
        let rt = rot_to_side(self.rotation);
        if side == rt {
            Some(&self.outbuf)
        } else if side == (rt ^ 1) {
            Some(&self.inbuf)
        } else {
            None
        }
    }

    /// Retrieves mutable fluid buffer for side.
    pub fn pipe_buffer_mut(&mut self, side: i32) -> Option<&mut FluidBuffer> {
        let rt = rot_to_side(self.rotation);
        if side == rt {
            Some(&mut self.outbuf)
        } else if side == (rt ^ 1) {
            Some(&mut self.inbuf)
        } else {
            None
        }
    }

    /// Moves fluid from `inbuf` to `outbuf`.
    pub fn pump_fluid(&mut self) {
        if self.inbuf.fluid_type != 0 {
            let max_out = (PIPE_MAX_LEVEL - self.outbuf.level).max(0);
            let mut lv = self.inbuf.level.min(max_out);
            lv = lv.min(self.inbuf.level + self.inbuf.delta);
            if lv > 0 && (self.inbuf.fluid_type == self.outbuf.fluid_type || self.outbuf.fluid_type == 0) {
                let ftype = self.inbuf.fluid_type;
                self.outbuf.add_level(ftype, lv);
                self.inbuf.add_level(ftype, -lv);
            }
        }
    }

    /// Tick logic corresponding to Java `TilePump#g()`.
    pub fn tick(&mut self) {
        if self.active {
            self.pump_tick = self.pump_tick.wrapping_add(1);
            if self.pump_tick == 8 {
                self.cond.add_charge(-100.0);
                self.pump_fluid();
            }

            if self.pump_tick >= 16 {
                self.pump_tick = 0;
                self.active = false;
            }

            self.cond.add_charge(-2.0);
        }

        if self.cond.voltage < 60.0 {
            self.charged = false;
        } else {
            if !self.charged {
                self.charged = true;
            }

            if self.charged && self.powered {
                self.active = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pump_pressure_and_sides() {
        let mut pump = TilePump::new(0); // rot_to_side(0) == 5 (East), inlet is 4 (West)
        assert_eq!(pump.pipe_pressure(5), 0);
        assert_eq!(pump.pipe_pressure(4), 0);

        pump.active = true;
        assert_eq!(pump.pipe_pressure(5), 1000);
        assert_eq!(pump.pipe_pressure(4), -1000);
        assert_eq!(pump.pipe_pressure(0), 0);
        assert_eq!(pump.pipe_pressure(1), 0);
    }

    #[test]
    fn test_pump_fluid_transfer() {
        let mut pump = TilePump::new(1); // rot_to_side(1) == 3
        pump.inbuf.fluid_type = 1;
        pump.inbuf.level = 500;

        pump.pump_fluid();
        assert_eq!(pump.outbuf.fluid_type, 1);
        assert_eq!(pump.outbuf.delta, 500);
        assert_eq!(pump.inbuf.delta, -500);
    }

    #[test]
    fn test_pump_power_cycle_and_persistence() {
        let mut pump = TilePump::new(2);
        pump.cond.add_charge(400.0); // 100.0V on 4.0F
        pump.powered = true;

        pump.tick();
        assert!(pump.charged);
        assert!(pump.active);

        // Advance to tick 8
        pump.inbuf.fluid_type = 2;
        pump.inbuf.level = 200;
        for _ in 0..8 {
            pump.tick();
        }
        assert_eq!(pump.pump_tick, 8);
        assert_eq!(pump.outbuf.delta, 200);

        // Serialization roundtrip
        let json = serde_json::to_string(&pump).expect("serialize pump");
        let restored: TilePump = serde_json::from_str(&json).expect("deserialize pump");
        assert_eq!(restored.rotation, 2);
        assert!(restored.charged);
    }
}
