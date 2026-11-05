//! IO Expander flat peripheral (`TileIOExpander`, BlockMultipart meta 0 in peripheralFlat).
//!
//! Connects to ribbon cable / Redbus at configurable address (default 3), mapping 4 memory-mapped
//! registers to 16 distinct bundled cable channels:
//! - Register 0: RBuf low byte (channels 1..=8 input latch)
//! - Register 1: RBuf high byte (channels 9..=16 input latch)
//! - Register 2: WBufNew low byte (channels 1..=8 pending output)
//! - Register 3: WBufNew high byte (channels 9..=16 pending output)
//!
//! Provides atomic 16-bit input latching and output driving for automation control from 6502 assembly.
//! Includes DIP switch configuration GUI (`GuiBusId`, `ContainerBusId`).

use crate::core::math::rot_to_side;
use crate::core::spatial::WorldCoord;
use crate::wiring::rules::{map_rot_to_con, rot_power_state, Connectable, RuleWorld};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default Redbus peripheral address for IO Expander (§2.7).
pub const DEFAULT_ADDRESS: u8 = 3;

/// Ticks between a register write and output reaching the cable (`scheduleTick(2)` in Java).
pub const WRITE_DELAY_TICKS: u32 = 2;

/// Connectable mask for flat multipart resting on bottom face (bits 0..=3: 15).
pub const CONNECTABLE_MASK: u32 = 15;

/// Connection class for Ribbon Cable / Redbus bus connection.
pub const CONNECT_CLASS_RIBBON: i32 = 18;

/// Connection class for Bundled Cable (16 channels).
pub const CONNECT_CLASS_BUNDLED: i32 = 66;

/// GUI dimensions for Set Bus ID screen (123x81).
pub const GUI_SIZE: [f32; 2] = [123.0, 81.0];
pub const GUI_TITLE: &str = "Set Bus Id";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IoExpander {
    /// Horizontal rotation on surface (0..=3).
    pub rotation: u32,
    /// Active 16-channel output driven onto the bundled cable (`WBuf`).
    pub w_buf: u16,
    /// Pending output written by Redbus/CPU before tick delay (`WBufNew`).
    pub w_buf_new: u16,
    /// Atomically sampled 16-channel input from bundled cable (`RBuf`).
    pub r_buf: u16,
    /// Redbus peripheral address (default 3, configurable 0..=255).
    pub rb_addr: u8,
    /// Flag indicating an output update tick has been scheduled.
    pub tick_pending: bool,
    /// Remaining delay ticks before pending output takes effect.
    pub delay: u32,
}

impl Default for IoExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl IoExpander {
    /// Create a new IO Expander with default settings (address 3, rotation 0).
    pub fn new() -> Self {
        Self {
            rotation: 0,
            w_buf: 0,
            w_buf_new: 0,
            r_buf: 0,
            rb_addr: DEFAULT_ADDRESS,
            tick_pending: false,
            delay: 0,
        }
    }

    /// Create an IO Expander with a specific rotation.
    pub fn with_rotation(rotation: u32) -> Self {
        Self {
            rotation: rotation & 3,
            ..Self::new()
        }
    }

    /// Calculate rotation from player placement yaw (`onBlockPlacedBy` in Java).
    /// `Rotation = (int)Math.floor(yaw * 4.0F / 360.0F + 0.5) + 1 & 3`
    pub fn from_yaw(yaw: f32) -> u32 {
        let r = ((yaw * 4.0 / 360.0 + 0.5).floor() as i32 + 1) & 3;
        r as u32
    }

    /// The side on which this flat peripheral connects to Ribbon Cable / Redbus.
    /// In Java: `CoreLib.rotToSide(this.Rotation)`.
    pub fn ribbon_side(&self) -> i32 {
        rot_to_side(self.rotation as i32)
    }

    /// Connection class for each side (`getConnectClass` in Java).
    /// The back/oriented side is Ribbon Cable (18); all other 3 planar sides are Bundled Cable (66).
    pub fn connect_class(&self, side: i32) -> i32 {
        if side == self.ribbon_side() {
            CONNECT_CLASS_RIBBON
        } else {
            CONNECT_CLASS_BUNDLED
        }
    }

    /// Connectable mask: 15 (bits 0..=3 for the 4 planar sides on bottom face).
    pub fn connectable_mask(&self) -> u32 {
        CONNECTABLE_MASK
    }

    /// Corner power mode: 0 (flat peripheral).
    pub fn corner_power_mode(&self) -> i32 {
        0
    }

    /// Read Redbus peripheral address (`rbGetAddr`).
    pub fn rb_get_addr(&self) -> u8 {
        self.rb_addr
    }

    /// Set Redbus peripheral address (`rbSetAddr`).
    pub fn rb_set_addr(&mut self, addr: u8) {
        self.rb_addr = addr;
    }

    /// Memory-mapped Redbus register read (`rbRead` in Java).
    /// - 0: RBuf & 0xFF (low byte of 16-channel input)
    /// - 1: RBuf >> 8 (high byte of 16-channel input)
    /// - 2: WBufNew & 0xFF (low byte of pending output)
    /// - 3: WBufNew >> 8 (high byte of pending output)
    pub fn rb_read(&self, reg: u8) -> u8 {
        match reg {
            0 => (self.r_buf & 0xFF) as u8,
            1 => (self.r_buf >> 8) as u8,
            2 => (self.w_buf_new & 0xFF) as u8,
            3 => (self.w_buf_new >> 8) as u8,
            _ => 0,
        }
    }

    /// Memory-mapped Redbus register write (`rbWrite` in Java).
    /// Writing registers 0 or 2 writes the low byte; 1 or 3 writes the high byte.
    /// Schedules a 2-tick delay before the output reaches the cable.
    pub fn rb_write(&mut self, reg: u8, dat: u8) -> bool {
        match reg {
            0 | 2 => self.w_buf_new = (self.w_buf_new & 0xFF00) | u16::from(dat),
            1 | 3 => self.w_buf_new = (self.w_buf_new & 0x00FF) | (u16::from(dat) << 8),
            _ => return false,
        }
        self.tick_pending = true;
        self.delay = WRITE_DELAY_TICKS;
        true
    }

    /// Powering mask for bundled cable channel (`getPoweringMask(ch)` in Java).
    /// Channel 0 is plain redstone (never driven by IO expander).
    /// Channels 1..=16 correspond to bit (ch - 1) of active `w_buf`.
    pub fn powering_mask(&self, ch: usize) -> u32 {
        if ch == 0 || ch > 16 {
            return 0;
        }
        if (self.w_buf & (1 << (ch - 1))) != 0 {
            map_rot_to_con(8, self.rotation)
        } else {
            0
        }
    }

    /// World tick update (`onTileTick` in Java).
    /// When scheduled tick arrives, updates active `w_buf` from pending `w_buf_new`.
    /// Returns true if output changed and neighbours must be notified.
    pub fn on_tick(&mut self) -> bool {
        self.tick_pending = false;
        self.delay = 0;
        if self.w_buf != self.w_buf_new {
            self.w_buf = self.w_buf_new;
            true
        } else {
            false
        }
    }

    /// Progress one world tick. If a write was pending and delay expires, updates output.
    /// Returns true if output changed.
    pub fn tick_step(&mut self) -> bool {
        if self.tick_pending {
            if self.delay <= 1 {
                self.on_tick()
            } else {
                self.delay -= 1;
                false
            }
        } else {
            false
        }
    }

    /// Atomically sample all 16 bundled cable input channels (`powered(n)` checks channel n + 1).
    /// Updates `r_buf` and returns true if any input changed.
    pub fn sample_inputs(&mut self, powered: impl Fn(usize) -> bool) -> bool {
        let mut new = self.r_buf;
        for n in 0..16 {
            if powered(n) {
                new |= 1 << n;
            } else {
                new &= !(1 << n);
            }
        }
        let changed = new != self.r_buf;
        self.r_buf = new;
        changed
    }

    /// Sample adjacent bundled cables through world rules (`onBlockNeighborChange` in Java).
    pub fn on_neighbor_change<W: RuleWorld>(&mut self, w: &W, p: WorldCoord) -> bool {
        let rot = self.rotation;
        self.sample_inputs(|n| rot_power_state(w, p, 8, rot, n + 1) != 0)
    }

    /// Toggle a bit of the bus ID via GUI click (`GuiBusId` DIP switch 0..=7).
    pub fn toggle_addr_bit(&mut self, bit: usize) -> u8 {
        if bit < 8 {
            self.rb_addr ^= 1 << bit;
        }
        self.rb_addr
    }

    /// Handle GUI event packet (`ContainerBusId.handleGuiEvent`). Event 1 sets the bus address.
    pub fn handle_gui_event(&mut self, event_id: u8, byte_val: u8) -> bool {
        if event_id == 1 {
            self.rb_addr = byte_val;
            true
        } else {
            false
        }
    }

    /// Convert to `Connectable` for wiring graph integration.
    pub fn as_connectable(&self) -> Connectable {
        let mut connect_class = [0i32; 6];
        for (side, val) in connect_class.iter_mut().enumerate() {
            *val = self.connect_class(side as i32);
        }
        let mut powering_mask = Vec::with_capacity(17);
        powering_mask.push(0); // ch 0
        for ch in 1..=16 {
            powering_mask.push(self.powering_mask(ch));
        }
        Connectable {
            connect_class,
            connectable_mask: self.connectable_mask(),
            corner_mode: self.corner_power_mode(),
            powering_mask,
        }
    }

    /// Integer array serialization (`[rot, wb, wbn, rb, rbaddr]`).
    pub fn to_ints(&self) -> [i32; 5] {
        [
            self.rotation as i32,
            self.w_buf as i32,
            self.w_buf_new as i32,
            self.r_buf as i32,
            self.rb_addr as i32,
        ]
    }

    /// Integer array deserialization.
    pub fn from_ints(v: [i32; 5]) -> Self {
        Self {
            rotation: (v[0] as u32) & 3,
            w_buf: (v[1] & 0xFFFF) as u16,
            w_buf_new: (v[2] & 0xFFFF) as u16,
            r_buf: (v[3] & 0xFFFF) as u16,
            rb_addr: (v[4] & 0xFF) as u8,
            tick_pending: false,
            delay: 0,
        }
    }

    /// Load state from NBT tag map.
    pub fn load_nbt(&mut self, nbt: &HashMap<String, String>) {
        if let Some(r) = nbt.get("rot").and_then(|v| v.parse::<u32>().ok()) {
            self.rotation = r & 3;
        }
        if let Some(wb) = nbt.get("wb").and_then(|v| v.parse::<u16>().ok()) {
            self.w_buf = wb;
        }
        if let Some(wbn) = nbt.get("wbn").and_then(|v| v.parse::<u16>().ok()) {
            self.w_buf_new = wbn;
        }
        if let Some(rb) = nbt.get("rb").and_then(|v| v.parse::<u16>().ok()) {
            self.r_buf = rb;
        }
        if let Some(addr) = nbt.get("rbaddr").and_then(|v| v.parse::<u8>().ok()) {
            self.rb_addr = addr;
        }
        if let Some(d) = nbt.get("delay").and_then(|v| v.parse::<u32>().ok()) {
            self.delay = d;
            self.tick_pending = d > 0;
        }
    }

    /// Save state to NBT tag map.
    pub fn save_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("rot".into(), self.rotation.to_string());
        nbt.insert("wb".into(), self.w_buf.to_string());
        nbt.insert("wbn".into(), self.w_buf_new.to_string());
        nbt.insert("rb".into(), self.r_buf.to_string());
        nbt.insert("rbaddr".into(), self.rb_addr.to_string());
        if self.tick_pending && self.delay > 0 {
            nbt.insert("delay".into(), self.delay.to_string());
        } else {
            nbt.remove("delay");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wiring::rules::BlockKind;

    #[test]
    fn test_rotation_and_placement() {
        assert_eq!(IoExpander::from_yaw(0.0), 1);
        assert_eq!(IoExpander::from_yaw(90.0), 2);
        assert_eq!(IoExpander::from_yaw(180.0), 3);
        assert_eq!(IoExpander::from_yaw(270.0), 0);

        let io = IoExpander::with_rotation(0);
        // rot 0 -> side 5 (East)
        assert_eq!(io.ribbon_side(), 5);
        assert_eq!(io.connect_class(5), CONNECT_CLASS_RIBBON);
        assert_eq!(io.connect_class(2), CONNECT_CLASS_BUNDLED);
        assert_eq!(io.connect_class(3), CONNECT_CLASS_BUNDLED);
        assert_eq!(io.connect_class(4), CONNECT_CLASS_BUNDLED);
        assert_eq!(io.connectable_mask(), 15);
        assert_eq!(io.corner_power_mode(), 0);

        let io1 = IoExpander::with_rotation(1);
        // rot 1 -> side 3 (South)
        assert_eq!(io1.ribbon_side(), 3);
        assert_eq!(io1.connect_class(3), CONNECT_CLASS_RIBBON);
        assert_eq!(io1.connect_class(5), CONNECT_CLASS_BUNDLED);
    }

    #[test]
    fn test_rb_read_write_delay_and_driving() {
        let mut io = IoExpander::new();
        assert_eq!(io.rb_get_addr(), 3);

        // Write low byte (channels 1..=8) and high byte (channels 9..=16)
        assert!(io.rb_write(0, 0b1010_0101));
        assert!(io.rb_write(1, 0b1100_0011));

        // Before tick, WBufNew has the written values, but WBuf is not yet driving
        assert_eq!(io.rb_read(2), 0b1010_0101);
        assert_eq!(io.rb_read(3), 0b1100_0011);
        assert_eq!(io.w_buf, 0);
        assert_eq!(io.powering_mask(1), 0);
        assert_eq!(io.powering_mask(16), 0);

        // Delay is 2 ticks
        assert!(!io.tick_step()); // tick 1: delay decrements to 1
        assert_eq!(io.w_buf, 0);
        assert!(io.tick_step()); // tick 2: delay expires, output updates

        // Now WBuf is active
        assert_eq!(io.w_buf, 0b1100_0011_1010_0101);
        // Channel 1 (bit 0): 1 -> active
        assert_ne!(io.powering_mask(1), 0);
        // Channel 2 (bit 1): 0 -> 0
        assert_eq!(io.powering_mask(2), 0);
        // Channel 8 (bit 7): 1 -> active
        assert_ne!(io.powering_mask(8), 0);
        // Channel 9 (bit 8): 1 -> active
        assert_ne!(io.powering_mask(9), 0);
        // Channel 11 (bit 10): 0 -> 0
        assert_eq!(io.powering_mask(11), 0);
        // Channel 16 (bit 15): 1 -> active
        assert_ne!(io.powering_mask(16), 0);
        // Channel 0 is never powered
        assert_eq!(io.powering_mask(0), 0);
        assert_eq!(io.powering_mask(17), 0);
    }

    #[test]
    fn test_atomic_input_latching() {
        let mut io = IoExpander::new();
        assert_eq!(io.r_buf, 0);

        // Sample bundled cable lines: channels 1, 4, 12, 16 high
        let changed = io.sample_inputs(|ch_idx| matches!(ch_idx, 0 | 3 | 11 | 15));
        assert!(changed);
        assert_eq!(io.r_buf, (1 << 0) | (1 << 3) | (1 << 11) | (1 << 15));

        // Read registers 0 and 1
        assert_eq!(io.rb_read(0), (1 << 0) | (1 << 3)); // low byte: bits 0 and 3
        assert_eq!(io.rb_read(1), (1 << (11 - 8)) | (1 << (15 - 8))); // high byte: bits 3 and 7

        // Resampling identical inputs returns changed = false
        assert!(!io.sample_inputs(|ch_idx| matches!(ch_idx, 0 | 3 | 11 | 15)));

        // Inputs drop to zero
        assert!(io.sample_inputs(|_| false));
        assert_eq!(io.rb_read(0), 0);
        assert_eq!(io.rb_read(1), 0);
    }

    #[test]
    fn test_gui_bus_id_and_dip_switches() {
        let mut io = IoExpander::new();
        assert_eq!(io.rb_get_addr(), 3); // bits 0 and 1 set

        // Toggle bit 0 (clear it) -> addr becomes 2
        assert_eq!(io.toggle_addr_bit(0), 2);
        // Toggle bit 4 (set it: +16) -> addr becomes 18
        assert_eq!(io.toggle_addr_bit(4), 18);
        assert_eq!(io.rb_get_addr(), 18);

        // Packet event 1 sets bus address directly
        assert!(io.handle_gui_event(1, 42));
        assert_eq!(io.rb_get_addr(), 42);
        // Unknown event returns false
        assert!(!io.handle_gui_event(2, 99));
        assert_eq!(io.rb_get_addr(), 42);
    }

    #[test]
    fn test_persistence_round_trip() {
        let mut io = IoExpander::with_rotation(2);
        io.rb_set_addr(7);
        io.rb_write(0, 0x12);
        io.rb_write(1, 0x34);
        io.on_tick();
        io.r_buf = 0xABCD;

        // Ints round-trip
        let ints = io.to_ints();
        let from_ints = IoExpander::from_ints(ints);
        assert_eq!(from_ints.rotation, 2);
        assert_eq!(from_ints.rb_addr, 7);
        assert_eq!(from_ints.w_buf, 0x3412);
        assert_eq!(from_ints.r_buf, 0xABCD);

        // NBT round-trip
        let mut nbt = HashMap::new();
        io.save_nbt(&mut nbt);
        let mut loaded = IoExpander::default();
        loaded.load_nbt(&nbt);
        assert_eq!(loaded.rotation, 2);
        assert_eq!(loaded.rb_addr, 7);
        assert_eq!(loaded.w_buf, 0x3412);
        assert_eq!(loaded.r_buf, 0xABCD);
    }

    #[test]
    fn test_connectable_representation() {
        let mut io = IoExpander::with_rotation(0);
        io.rb_write(0, 0x01); // channel 1
        io.on_tick();
        let conn = io.as_connectable();
        assert_eq!(conn.connectable_mask, 15);
        assert_eq!(conn.corner_mode, 0);
        assert_eq!(conn.connect_class[5], CONNECT_CLASS_RIBBON);
        assert_eq!(conn.connect_class[2], CONNECT_CLASS_BUNDLED);
        assert_ne!(conn.powering_mask[1], 0);
        assert_eq!(conn.powering_mask[2], 0);
    }

    // Mock world to test on_neighbor_change with RuleWorld
    struct MockWorld {
        kind: BlockKind,
        opaque: bool,
    }

    impl RuleWorld for MockWorld {
        fn kind(&self, _p: WorldCoord) -> BlockKind {
            self.kind
        }
        fn connectable(&self, _p: WorldCoord) -> Option<Connectable> {
            None
        }
        fn classes(&self) -> &crate::wiring::rules::PowerClasses {
            static CLASSES: std::sync::OnceLock<crate::wiring::rules::PowerClasses> =
                std::sync::OnceLock::new();
            CLASSES.get_or_init(crate::wiring::rules::PowerClasses::new)
        }
        fn is_opaque_cube(&self, _p: WorldCoord) -> bool {
            self.opaque
        }
        fn is_normal_cube(&self, _p: WorldCoord) -> bool {
            self.opaque
        }
        fn is_solid_on_side(&self, _p: WorldCoord, _side: i32) -> bool {
            self.opaque
        }
        fn multipart_side_normal(&self, _p: WorldCoord, _side: i32) -> Option<bool> {
            None
        }
        fn weak_power(&self, _p: WorldCoord, _side: i32) -> bool {
            false
        }
        fn strong_power(&self, _p: WorldCoord, _side: i32) -> bool {
            false
        }
        fn can_connect_redstone(&self, _p: WorldCoord, _side: i32) -> bool {
            false
        }
        fn searching(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_neighbor_change_with_world() {
        let mut io = IoExpander::new();
        let world = MockWorld {
            kind: BlockKind::Air,
            opaque: false,
        };
        // Unpowered world should leave inputs at 0
        let changed = io.on_neighbor_change(&world, WorldCoord::ZERO);
        assert!(!changed);
        assert_eq!(io.r_buf, 0);
    }

    #[test]
    fn test_6502_assembly_automation_integration() {
        let mut cpu = crate::Cpu6502::new();
        let mut io = IoExpander::new();

        // Simulate 6502 CPU reading input from IO Expander and setting outputs.
        // Input on IO expander: sensors on channels 1 & 2 (0x0003)
        io.r_buf = 0x0003;

        // Assembly program:
        // LDA $E000  (read low byte from IO expander: 0x03)
        // CMP #$03
        // BEQ match
        // BRK
        // match:
        // LDA #$80   (activate channel 8 output actuator)
        // STA $E000
        // BRK
        let prog = [
            0xAD, 0x00, 0xE0, // LDA $E000
            0xC9, 0x03, // CMP #$03
            0xF0, 0x01, // BEQ +1
            0x00, // BRK
            0xA9, 0x80, // LDA #$80
            0x8D, 0x00, 0xE0, // STA $E000
            0x00, // BRK
        ];

        // Map IO expander read register 0 to CPU address $E000
        cpu.load_program(0x0400, &prog);
        cpu.io_expander =
            rekkit_core::frequency::BundledSignal::from_digital_mask(io.rb_read(0) as u16);

        cpu.step_tick(20);

        // CPU wrote 0x80 to address 0xE000
        assert_eq!(cpu.ram[0xE000], 0x80);

        // Apply CPU output to IO Expander register 0
        io.rb_write(0, cpu.ram[0xE000]);
        assert_eq!(io.rb_read(2), 0x80);
        io.on_tick();
        assert_ne!(io.powering_mask(8), 0);
        assert_eq!(io.powering_mask(1), 0);
    }

    #[test]
    fn test_6502_assembly_high_byte_automation_integration() {
        let mut cpu = crate::Cpu6502::new();
        let mut io = IoExpander::new();

        // High byte input: sensors on channel 16 (bit 15: 0x8000)
        io.r_buf = 0x8000;

        // Assembly program:
        // LDA $E001  (read high byte from IO expander: 0x80)
        // CMP #$80
        // BEQ match
        // BRK
        // match:
        // LDA #$40   (activate channel 15 output actuator: bit 6 of high byte)
        // STA $E001
        // BRK
        let prog = [
            0xAD, 0x01, 0xE0, // LDA $E001
            0xC9, 0x80, // CMP #$80
            0xF0, 0x01, // BEQ +1
            0x00, // BRK
            0xA9, 0x40, // LDA #$40
            0x8D, 0x01, 0xE0, // STA $E001
            0x00, // BRK
        ];

        cpu.load_program(0x0400, &prog);
        cpu.io_expander =
            rekkit_core::frequency::BundledSignal::from_digital_mask((io.rb_read(1) as u16) << 8);

        cpu.step_tick(20);

        // CPU wrote 0x40 to address 0xE001
        assert_eq!(cpu.ram[0xE001], 0x40);

        // Apply CPU output to IO Expander register 1 (high byte)
        io.rb_write(1, cpu.ram[0xE001]);
        assert_eq!(io.rb_read(3), 0x40);
        io.on_tick();
        // Channel 15 is active (bit 6 of high byte = bit 14 of w_buf = channel 15)
        assert_ne!(io.powering_mask(15), 0);
        assert_eq!(io.powering_mask(16), 0);
        assert_eq!(io.powering_mask(8), 0);
    }
}
