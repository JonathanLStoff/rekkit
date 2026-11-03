//! I/O Expander (`TileIOExpander`): Redbus device mapping a 16-channel bundled cable to
//! four registers. Pure state; the host feeds it the cable's input power and applies output.

use crate::core::spatial::WorldCoord;
use crate::wiring::rules::{map_rot_to_con, rot_power_state, RuleWorld};
use serde::{Deserialize, Serialize};

/// Ticks between a register write and the output reaching the cable.
pub const WRITE_DELAY_TICKS: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IoExpander {
    pub rotation: u32,
    /// Output currently driven onto the cable (`WBuf`).
    pub w_buf: u16,
    /// Pending output (`WBufNew`).
    pub w_buf_new: u16,
    /// Last sampled input (`RBuf`).
    pub r_buf: u16,
    pub rb_addr: u8,
    /// Set when a write scheduled a tick.
    pub tick_pending: bool,
}

impl Default for IoExpander {
    fn default() -> Self {
        Self {
            rotation: 0,
            w_buf: 0,
            w_buf_new: 0,
            r_buf: 0,
            rb_addr: 3,
            tick_pending: false,
        }
    }
}

impl IoExpander {
    /// `rbRead`.
    pub fn rb_read(&self, reg: u8) -> u8 {
        match reg {
            0 => (self.r_buf & 0xFF) as u8,
            1 => (self.r_buf >> 8) as u8,
            2 => (self.w_buf_new & 0xFF) as u8,
            3 => (self.w_buf_new >> 8) as u8,
            _ => 0,
        }
    }

    /// `rbWrite`; the caller schedules a tick `WRITE_DELAY_TICKS` ahead when this returns true.
    pub fn rb_write(&mut self, reg: u8, dat: u8) -> bool {
        match reg {
            0 | 2 => self.w_buf_new = (self.w_buf_new & 0xFF00) | u16::from(dat),
            1 | 3 => self.w_buf_new = (self.w_buf_new & 0x00FF) | (u16::from(dat) << 8),
            _ => return false,
        }
        self.tick_pending = true;
        true
    }

    /// `getPoweringMask(ch)`: channel 0 is plain redstone (never driven); channels 1..=16 are bits.
    pub fn powering_mask(&self, ch: usize) -> u32 {
        if ch == 0 || ch > 16 {
            return 0;
        }
        if self.w_buf & (1 << (ch - 1)) != 0 {
            map_rot_to_con(8, self.rotation)
        } else {
            0
        }
    }

    /// `onTileTick`: returns true when the output changed (neighbours must update).
    pub fn on_tick(&mut self) -> bool {
        self.tick_pending = false;
        if self.w_buf != self.w_buf_new {
            self.w_buf = self.w_buf_new;
            true
        } else {
            false
        }
    }

    /// Resample input from `powered[n]` (channel n+1 powered). Returns true if `RBuf` changed.
    pub fn sample_inputs(&mut self, powered: impl Fn(usize) -> bool) -> bool {
        let mut new = self.r_buf;
        for n in 0..16 {
            if powered(n) {
                new |= 1 << n;
            } else {
                new &= !(1 << n);
            }
        }
        let ch = new != self.r_buf;
        self.r_buf = new;
        ch
    }

    /// `onBlockNeighborChange`: sample the cable through the world's rules.
    pub fn on_neighbor_change<W: RuleWorld>(&mut self, w: &W, p: WorldCoord) -> bool {
        let rot = self.rotation;
        self.sample_inputs(|n| rot_power_state(w, p, 8, rot, n + 1) != 0)
    }

    /// NBT-like persistence (`wb`, `wbn`, `rb`, `rbaddr`, `rot`).
    pub fn to_ints(&self) -> [i32; 5] {
        [
            self.rotation as i32,
            self.w_buf as i32,
            self.w_buf_new as i32,
            self.r_buf as i32,
            self.rb_addr as i32,
        ]
    }

    pub fn from_ints(v: [i32; 5]) -> Self {
        Self {
            rotation: v[0] as u32,
            w_buf: (v[1] & 0xFFFF) as u16,
            w_buf_new: (v[2] & 0xFFFF) as u16,
            r_buf: (v[3] & 0xFFFF) as u16,
            rb_addr: (v[4] & 0xFF) as u8,
            tick_pending: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_is_delayed_until_tick() {
        let mut e = IoExpander::default();
        assert_eq!(e.rb_addr, 3);
        assert!(e.rb_write(0, 0x05));
        assert!(e.rb_write(3, 0x80));
        assert_eq!(e.powering_mask(1), 0);
        assert_eq!(e.rb_read(2), 0x05);
        assert_eq!(e.rb_read(3), 0x80);
        assert!(e.on_tick());
        assert_ne!(e.powering_mask(1), 0);
        assert_eq!(e.powering_mask(2), 0);
        assert_ne!(e.powering_mask(16), 0);
        assert_eq!(e.powering_mask(0), 0);
        assert!(!e.on_tick());
    }

    #[test]
    fn inputs_read_back() {
        let mut e = IoExpander::default();
        assert!(e.sample_inputs(|n| n == 0 || n == 9));
        assert_eq!(e.rb_read(0), 1);
        assert_eq!(e.rb_read(1), 2);
        assert!(!e.sample_inputs(|n| n == 0 || n == 9));
        assert!(e.sample_inputs(|_| false));
        assert_eq!(e.rb_read(0), 0);
    }

    #[test]
    fn persistence_roundtrip() {
        let mut e = IoExpander::default();
        e.rb_write(0, 0xAB);
        e.on_tick();
        e.r_buf = 0x1234;
        assert_eq!(IoExpander::from_ints(e.to_ints()), e);
    }
}
