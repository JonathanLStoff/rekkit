//! ContainerCPU and GuiCPU for the CPU peripheral (`com.eloraam.redpower.control.ContainerCPU`,
//! `com.eloraam.redpower.control.GuiCPU`, `TileCPU`).
//!
//! Provides DIP switch peripheral bus configuration (Disk ID, Console ID, Bus ID),
//! hardware control switches (START, HALT, RESET), running status LED indicators,
//! and 6502 hardware debugger register displays:
//! Accumulator (A), Index Registers (X, Y), Stack Pointer (SP), Program Counter (PC),
//! and Status Flags (NV-BDIZC). Supports single-step execution, run/stop toggle,
//! and reset buttons via packet events (`Packet212GuiEvent`).

use crate::core::foundation::Packet212GuiEvent;
use crate::Cpu6502;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Packet 212 GUI Event IDs & Menu Button Constants
// ============================================================================

/// Packet 212 GUI Event 1: update `byte0` (Disk ID, 8-bit DIP switch).
pub const EVENT_SET_DISK_ID: u32 = 1;
/// Packet 212 GUI Event 2: update `byte1` (Console ID, 8-bit DIP switch).
pub const EVENT_SET_CONSOLE_ID: u32 = 2;
/// Packet 212 GUI Event 3: update `rbaddr` (CPU Redbus ID, 8-bit DIP switch).
pub const EVENT_SET_BUS_ID: u32 = 3;
/// Packet 212 GUI Event 4: START button / Warm Boot (`TileCPU.warmBootCPU`).
pub const EVENT_WARM_BOOT: u32 = 4;
/// Packet 212 GUI Event 5: HALT button / Stop (`TileCPU.haltCPU`).
pub const EVENT_HALT: u32 = 5;
/// Packet 212 GUI Event 6: RESET button / Cold Boot (`TileCPU.coldBootCPU`).
pub const EVENT_COLD_BOOT: u32 = 6;
/// Packet 212 GUI Event 7: Single-step instruction debugger execution.
pub const EVENT_STEP: u32 = 7;
/// Packet 212 GUI Event 8: Run/Stop toggle debugger control.
pub const EVENT_TOGGLE_RUN: u32 = 8;

/// Horizon menu button ID: START button.
pub const BTN_START: i32 = 4;
/// Horizon menu button ID: HALT button.
pub const BTN_HALT: i32 = 5;
/// Horizon menu button ID: RESET button.
pub const BTN_RESET: i32 = 6;
/// Horizon menu button ID: Single-step button.
pub const BTN_STEP: i32 = 7;
/// Horizon menu button ID: Run/Stop toggle button.
pub const BTN_TOGGLE_RUN: i32 = 8;

/// Base button ID for Disk ID DIP switches: `BTN_DISK_BIT_BASE + bit` (bit in 0..8).
pub const BTN_DISK_BIT_BASE: i32 = 100;
/// Base button ID for Console ID DIP switches: `BTN_CONSOLE_BIT_BASE + bit` (bit in 0..8).
pub const BTN_CONSOLE_BIT_BASE: i32 = 200;
/// Base button ID for Redbus ID DIP switches: `BTN_BUS_BIT_BASE + bit` (bit in 0..8).
pub const BTN_BUS_BIT_BASE: i32 = 300;

// ============================================================================
// Layout Constants
// ============================================================================

/// GuiCPU window width in pixels (`b = 227`).
pub const GUI_WIDTH: f32 = 227.0;
/// GuiCPU window height in pixels (`c = 145`).
pub const GUI_HEIGHT: f32 = 145.0;
/// Texture path for GuiCPU background.
pub const TEXTURE_PATH: &str = "redpower_2:textures/gui/cpugui.png";

// ============================================================================
// 6502 Hardware Debugger Registers
// ============================================================================

/// Hardware debugger registers and status flags for MOS 6502 / 65C816.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuRegisters {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: u8,
    pub halted: bool,
}

impl Default for CpuRegisters {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0x0400,
            status: 0x20, // Bit 5 (unused) is always 1 in 6502
            halted: true,
        }
    }
}

impl CpuRegisters {
    /// Negative flag (N, bit 7).
    pub fn flag_n(&self) -> bool {
        (self.status & 0x80) != 0
    }

    /// Overflow flag (V, bit 6).
    pub fn flag_v(&self) -> bool {
        (self.status & 0x40) != 0
    }

    /// Unused flag (bit 5, always 1 on 6502).
    pub fn flag_unused(&self) -> bool {
        (self.status & 0x20) != 0
    }

    /// Break flag (B, bit 4).
    pub fn flag_b(&self) -> bool {
        (self.status & 0x10) != 0
    }

    /// Decimal mode flag (D, bit 3).
    pub fn flag_d(&self) -> bool {
        (self.status & 0x08) != 0
    }

    /// Interrupt disable flag (I, bit 2).
    pub fn flag_i(&self) -> bool {
        (self.status & 0x04) != 0
    }

    /// Zero flag (Z, bit 1).
    pub fn flag_z(&self) -> bool {
        (self.status & 0x02) != 0
    }

    /// Carry flag (C, bit 0).
    pub fn flag_c(&self) -> bool {
        (self.status & 0x01) != 0
    }

    /// Formats the status flags as `NV-BDIZC` where inactive flags are `-`.
    pub fn format_flags(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            if self.flag_n() { 'N' } else { '-' },
            if self.flag_v() { 'V' } else { '-' },
            if self.flag_unused() { '-' } else { ' ' },
            if self.flag_b() { 'B' } else { '-' },
            if self.flag_d() { 'D' } else { '-' },
            if self.flag_i() { 'I' } else { '-' },
            if self.flag_z() { 'Z' } else { '-' },
            if self.flag_c() { 'C' } else { '-' },
        )
    }

    pub fn format_a(&self) -> String {
        format!("A: ${:02X}", self.a)
    }

    pub fn format_x(&self) -> String {
        format!("X: ${:02X}", self.x)
    }

    pub fn format_y(&self) -> String {
        format!("Y: ${:02X}", self.y)
    }

    pub fn format_sp(&self) -> String {
        format!("SP: ${:02X}", self.sp)
    }

    pub fn format_pc(&self) -> String {
        format!("PC: ${:04X}", self.pc)
    }

    pub fn format_status(&self) -> String {
        format!("P: ${:02X} [{}]", self.status, self.format_flags())
    }
}

// ============================================================================
// ContainerCPU
// ============================================================================

/// Progress bar field update sent over Minecraft container network (`ContainerCPU.a()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressBarUpdate {
    pub id: i32,
    pub val: i32,
}

/// Port of `com.eloraam.redpower.control.ContainerCPU`.
///
/// Contains peripheral DIP switch settings (Disk ID, Console ID, Bus ID),
/// execution state, and debugger register mirror. Has 0 inventory slots (custom GUI only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerCPU {
    /// Disk ID configured by DIP switch 0 (default: 2, `TileCPU.byte0`).
    pub byte0: u8,
    /// Console/Terminal ID configured by DIP switch 1 (default: 1, `TileCPU.byte1`).
    pub byte1: u8,
    /// CPU Redbus address configured by DIP switch 2 (default: 0, `TileCPU.rbaddr`).
    pub rbaddr: u8,
    /// Whether the CPU is currently executing instructions (`TileCPU.isRunning()`).
    pub is_running: bool,
    /// Hardware debugger register states.
    pub registers: CpuRegisters,
}

impl Default for ContainerCPU {
    fn default() -> Self {
        Self {
            byte0: 2,
            byte1: 1,
            rbaddr: 0,
            is_running: false,
            registers: CpuRegisters::default(),
        }
    }
}

impl ContainerCPU {
    pub fn new(
        byte0: u8,
        byte1: u8,
        rbaddr: u8,
        is_running: bool,
        registers: CpuRegisters,
    ) -> Self {
        Self {
            byte0,
            byte1,
            rbaddr,
            is_running,
            registers,
        }
    }

    /// Java: `ContainerCPU.b(yw player)` -> `TileCPU.isUseableByPlayer(player)`.
    /// Check whether the player is within interaction distance (default 8 blocks / 64 dist_sq).
    pub fn is_useable_by_player(&self, dist_sq: f64) -> bool {
        dist_sq <= 64.0
    }

    /// Java: `ContainerCPU.a(int i)` -> `null`.
    /// No inventory transfer slots in ContainerCPU.
    pub fn transfer_stack_in_slot(&self, _slot: usize) -> Option<()> {
        None
    }

    /// Sync internal container state from a live `Cpu6502` instance.
    pub fn sync_from_cpu(&mut self, cpu: &Cpu6502) {
        self.registers.a = cpu.a;
        self.registers.x = cpu.x;
        self.registers.y = cpu.y;
        self.registers.sp = cpu.sp;
        self.registers.pc = cpu.pc;
        self.registers.status = cpu.status;
        self.registers.halted = cpu.halted;
        self.is_running = !cpu.halted;
    }

    /// Sync settings and execution state to a live `Cpu6502` instance.
    pub fn sync_to_cpu(&self, cpu: &mut Cpu6502) {
        cpu.a = self.registers.a;
        cpu.x = self.registers.x;
        cpu.y = self.registers.y;
        cpu.sp = self.registers.sp;
        cpu.pc = self.registers.pc;
        cpu.status = self.registers.status;
        cpu.halted = self.registers.halted;
        if !cpu.ram.is_empty() {
            cpu.ram[0] = self.byte0;
        }
        if cpu.ram.len() > 1 {
            cpu.ram[1] = self.byte1;
        }
    }

    /// Java: `ContainerCPU.a()`: detect field changes and emit container updates.
    pub fn detect_changes(
        &mut self,
        current_byte0: u8,
        current_byte1: u8,
        current_rbaddr: u8,
        current_isrun: bool,
    ) -> Vec<ProgressBarUpdate> {
        let mut updates = Vec::new();
        if current_byte0 != self.byte0 {
            updates.push(ProgressBarUpdate {
                id: 0,
                val: current_byte0 as i32,
            });
            self.byte0 = current_byte0;
        }
        if current_byte1 != self.byte1 {
            updates.push(ProgressBarUpdate {
                id: 1,
                val: current_byte1 as i32,
            });
            self.byte1 = current_byte1;
        }
        if current_rbaddr != self.rbaddr {
            updates.push(ProgressBarUpdate {
                id: 2,
                val: current_rbaddr as i32,
            });
            self.rbaddr = current_rbaddr;
        }
        if current_isrun != self.is_running {
            updates.push(ProgressBarUpdate {
                id: 3,
                val: if current_isrun { 1 } else { 0 },
            });
            self.is_running = current_isrun;
            self.registers.halted = !current_isrun;
        }
        updates
    }

    /// Java: `ContainerCPU.a(int i, int j)`: update progress bar field on client.
    pub fn update_progress_bar(&mut self, i: i32, j: i32) {
        match i {
            0 => self.byte0 = j as u8,
            1 => self.byte1 = j as u8,
            2 => self.rbaddr = j as u8,
            3 => {
                self.is_running = j > 0;
                self.registers.halted = !self.is_running;
            }
            4 => self.registers.a = j as u8,
            5 => self.registers.x = j as u8,
            6 => self.registers.y = j as u8,
            7 => self.registers.sp = j as u8,
            8 => self.registers.pc = j as u16,
            9 => self.registers.status = j as u8,
            _ => {}
        }
    }

    /// Java: `ContainerCPU.handleGuiEvent(Packet212GuiEvent packet)`.
    ///
    /// Handles packet 212 events:
    /// - 1: set Disk ID (`byte0`)
    /// - 2: set Console ID (`byte1`)
    /// - 3: set Bus ID (`rbaddr`)
    /// - 4: START button / warm boot
    /// - 5: HALT button / halt
    /// - 6: RESET button / cold boot
    /// - 7: single-step execution
    /// - 8: run/stop toggle
    pub fn handle_gui_event(&mut self, pkt: &Packet212GuiEvent) {
        match pkt.event_id {
            EVENT_SET_DISK_ID => {
                self.byte0 = pkt.body.first().copied().unwrap_or(self.byte0);
            }
            EVENT_SET_CONSOLE_ID => {
                self.byte1 = pkt.body.first().copied().unwrap_or(self.byte1);
            }
            EVENT_SET_BUS_ID => {
                self.rbaddr = pkt.body.first().copied().unwrap_or(self.rbaddr);
            }
            EVENT_WARM_BOOT => {
                self.is_running = true;
                self.registers.halted = false;
            }
            EVENT_HALT => {
                self.is_running = false;
                self.registers.halted = true;
            }
            EVENT_COLD_BOOT => {
                self.is_running = false;
                self.registers.halted = true;
                self.registers.a = 0;
                self.registers.x = 0;
                self.registers.y = 0;
                self.registers.sp = 0xFD;
                self.registers.pc = 0x0400;
                self.registers.status = 0x20;
            }
            EVENT_STEP => {
                // Stepping occurs on the CPU instance
            }
            EVENT_TOGGLE_RUN => {
                self.is_running = !self.is_running;
                self.registers.halted = !self.is_running;
            }
            _ => {}
        }
    }

    /// Handles a GUI event directly modifying both container state and live `Cpu6502`.
    pub fn handle_gui_event_on_cpu(&mut self, cpu: &mut Cpu6502, pkt: &Packet212GuiEvent) {
        self.handle_gui_event(pkt);
        match pkt.event_id {
            EVENT_SET_DISK_ID => {
                if !cpu.ram.is_empty() {
                    cpu.ram[0] = self.byte0;
                }
            }
            EVENT_SET_CONSOLE_ID => {
                if cpu.ram.len() > 1 {
                    cpu.ram[1] = self.byte1;
                }
            }
            EVENT_WARM_BOOT => {
                cpu.warm_boot();
                self.sync_from_cpu(cpu);
            }
            EVENT_HALT => {
                cpu.halt();
                self.sync_from_cpu(cpu);
            }
            EVENT_COLD_BOOT => {
                cpu.cold_boot();
                if !cpu.ram.is_empty() {
                    cpu.ram[0] = self.byte0;
                }
                if cpu.ram.len() > 1 {
                    cpu.ram[1] = self.byte1;
                }
                self.sync_from_cpu(cpu);
            }
            EVENT_STEP => {
                cpu.step_single();
                self.sync_from_cpu(cpu);
            }
            EVENT_TOGGLE_RUN => {
                if cpu.is_running() {
                    cpu.halt();
                } else {
                    cpu.warm_boot();
                }
                self.sync_from_cpu(cpu);
            }
            _ => {}
        }
    }

    /// Apply an incoming Horizon menu button click ID (`native_menu_button`).
    pub fn apply_menu_button(&mut self, button_id: i32, mut cpu: Option<&mut Cpu6502>) {
        match button_id {
            BTN_START => {
                let pkt = GuiCPU::send_event(0, EVENT_WARM_BOOT);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            BTN_HALT => {
                let pkt = GuiCPU::send_event(0, EVENT_HALT);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            BTN_RESET => {
                let pkt = GuiCPU::send_event(0, EVENT_COLD_BOOT);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            BTN_STEP => {
                let pkt = GuiCPU::send_event(0, EVENT_STEP);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            BTN_TOGGLE_RUN => {
                let pkt = GuiCPU::send_event(0, EVENT_TOGGLE_RUN);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            b if (BTN_DISK_BIT_BASE..BTN_DISK_BIT_BASE + 8).contains(&b) => {
                let bit = (b - BTN_DISK_BIT_BASE) as u8;
                let new_val = self.byte0 ^ (1 << bit);
                let pkt = GuiCPU::send_simple(0, EVENT_SET_DISK_ID, new_val);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            b if (BTN_CONSOLE_BIT_BASE..BTN_CONSOLE_BIT_BASE + 8).contains(&b) => {
                let bit = (b - BTN_CONSOLE_BIT_BASE) as u8;
                let new_val = self.byte1 ^ (1 << bit);
                let pkt = GuiCPU::send_simple(0, EVENT_SET_CONSOLE_ID, new_val);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            b if (BTN_BUS_BIT_BASE..BTN_BUS_BIT_BASE + 8).contains(&b) => {
                let bit = (b - BTN_BUS_BIT_BASE) as u8;
                let new_val = self.rbaddr ^ (1 << bit);
                let pkt = GuiCPU::send_simple(0, EVENT_SET_BUS_ID, new_val);
                if let Some(c) = cpu.as_mut() {
                    self.handle_gui_event_on_cpu(c, &pkt);
                } else {
                    self.handle_gui_event(&pkt);
                }
            }
            _ => {}
        }
    }

    /// Encode state into `entity.data` for client sync.
    /// Format: `[byte0, byte1, rbaddr, is_running, a, x, y, sp, pc, status, halted]`
    pub fn to_data(&self) -> Vec<i32> {
        vec![
            self.byte0 as i32,
            self.byte1 as i32,
            self.rbaddr as i32,
            if self.is_running { 1 } else { 0 },
            self.registers.a as i32,
            self.registers.x as i32,
            self.registers.y as i32,
            self.registers.sp as i32,
            self.registers.pc as i32,
            self.registers.status as i32,
            if self.registers.halted { 1 } else { 0 },
        ]
    }

    /// Decode state from `entity.data`.
    pub fn from_data(data: &[i32]) -> Self {
        let g = |idx: usize, def: i32| data.get(idx).copied().unwrap_or(def);
        let byte0 = g(0, 2) as u8;
        let byte1 = g(1, 1) as u8;
        let rbaddr = g(2, 0) as u8;
        let is_running = g(3, 0) > 0;
        let registers = CpuRegisters {
            a: g(4, 0) as u8,
            x: g(5, 0) as u8,
            y: g(6, 0) as u8,
            sp: g(7, 0xFD) as u8,
            pc: g(8, 0x0400) as u16,
            status: g(9, 0x20) as u8,
            halted: g(10, if is_running { 0 } else { 1 }) > 0,
        };
        Self {
            byte0,
            byte1,
            rbaddr,
            is_running,
            registers,
        }
    }

    /// Serialize persistent settings into TileEntity NBT.
    pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("byte0".into(), self.byte0.to_string());
        nbt.insert("byte1".into(), self.byte1.to_string());
        nbt.insert("rbaddr".into(), self.rbaddr.to_string());
        nbt.insert("is_running".into(), self.is_running.to_string());
        nbt.insert("reg_a".into(), self.registers.a.to_string());
        nbt.insert("reg_x".into(), self.registers.x.to_string());
        nbt.insert("reg_y".into(), self.registers.y.to_string());
        nbt.insert("reg_sp".into(), self.registers.sp.to_string());
        nbt.insert("reg_pc".into(), self.registers.pc.to_string());
        nbt.insert("status".into(), self.registers.status.to_string());
        nbt.insert("halted".into(), self.registers.halted.to_string());
    }

    /// Restore settings from TileEntity NBT.
    pub fn from_nbt(nbt: &HashMap<String, String>) -> Self {
        let g = |key: &str, def: i32| {
            nbt.get(key)
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(def)
        };
        let byte0 = g("byte0", 2) as u8;
        let byte1 = g("byte1", 1) as u8;
        let rbaddr = g("rbaddr", 0) as u8;
        let is_running = nbt
            .get("is_running")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let registers = CpuRegisters {
            a: g("reg_a", 0) as u8,
            x: g("reg_x", 0) as u8,
            y: g("reg_y", 0) as u8,
            sp: g("reg_sp", 0xFD) as u8,
            pc: g("reg_pc", 0x0400) as u16,
            status: g("status", 0x20) as u8,
            halted: nbt
                .get("halted")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(!is_running),
        };
        Self {
            byte0,
            byte1,
            rbaddr,
            is_running,
            registers,
        }
    }
}

// ============================================================================
// GuiCPU Actions and Click Handling
// ============================================================================

/// Action triggered by interacting with the GuiCPU screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiCpuAction {
    SetDiskId {
        new_val: u8,
        bit: u8,
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
    SetConsoleId {
        new_val: u8,
        bit: u8,
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
    SetBusId {
        new_val: u8,
        bit: u8,
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
    Start {
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
    Halt {
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
    Reset {
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
    Step {
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
    ToggleRun {
        pkt: Packet212GuiEvent,
        button_id: i32,
    },
}

/// Port of `com.eloraam.redpower.control.GuiCPU`.
pub struct GuiCPU;

impl GuiCPU {
    /// Java: `GuiCPU.sendSimple(int n, int m)`: creates a Packet 212 with a single byte payload.
    pub fn send_simple(window_id: u8, event_id: u32, val: u8) -> Packet212GuiEvent {
        Packet212GuiEvent {
            window_id,
            event_id,
            body: vec![val],
        }
    }

    /// Java: `GuiCPU.sendEvent(int n)`: creates a Packet 212 with a zero payload.
    pub fn send_event(window_id: u8, event_id: u32) -> Packet212GuiEvent {
        Packet212GuiEvent {
            window_id,
            event_id,
            body: vec![0],
        }
    }

    /// Java: `GuiCPU.a(int i, int j, int k)`: evaluate mouse click at `(x, y)` relative to GUI top-left.
    pub fn click(at: [f32; 2], byte0: u8, byte1: u8, rbaddr: u8) -> Option<GuiCpuAction> {
        let (x, y) = (at[0], at[1]);

        // DIP Switches row 1 (Disk ID & Console ID, y: 57..=89)
        if (57.0..=89.0).contains(&y) {
            // Disk ID: 8 switches at x: 14 + n * 12 ..= 26 + n * 12
            for n in 0..8u8 {
                let x0 = 14.0 + (n as f32) * 12.0;
                let x1 = 26.0 + (n as f32) * 12.0;
                if x >= x0 && x <= x1 {
                    let new_val = byte0 ^ (1 << n);
                    return Some(GuiCpuAction::SetDiskId {
                        new_val,
                        bit: n,
                        pkt: Self::send_simple(0, EVENT_SET_DISK_ID, new_val),
                        button_id: BTN_DISK_BIT_BASE + (n as i32),
                    });
                }
            }

            // Console ID: 8 switches at x: 118 + n * 12 ..= 130 + n * 12
            for n in 0..8u8 {
                let x0 = 118.0 + (n as f32) * 12.0;
                let x1 = 130.0 + (n as f32) * 12.0;
                if x >= x0 && x <= x1 {
                    let new_val = byte1 ^ (1 << n);
                    return Some(GuiCpuAction::SetConsoleId {
                        new_val,
                        bit: n,
                        pkt: Self::send_simple(0, EVENT_SET_CONSOLE_ID, new_val),
                        button_id: BTN_CONSOLE_BIT_BASE + (n as i32),
                    });
                }
            }
        }

        // DIP Switches row 2 (Redbus ID, y: 101..=133)
        if (101.0..=133.0).contains(&y) {
            for n in 0..8u8 {
                let x0 = 118.0 + (n as f32) * 12.0;
                let x1 = 130.0 + (n as f32) * 12.0;
                if x >= x0 && x <= x1 {
                    let new_val = rbaddr ^ (1 << n);
                    return Some(GuiCpuAction::SetBusId {
                        new_val,
                        bit: n,
                        pkt: Self::send_simple(0, EVENT_SET_BUS_ID, new_val),
                        button_id: BTN_BUS_BIT_BASE + (n as i32),
                    });
                }
            }
        }

        // Push Buttons (START, HALT, RESET at x: 87..=96)
        if (87.0..=96.0).contains(&x) {
            if (98.0..=107.0).contains(&y) {
                return Some(GuiCpuAction::Start {
                    pkt: Self::send_event(0, EVENT_WARM_BOOT),
                    button_id: BTN_START,
                });
            }
            if (111.0..=120.0).contains(&y) {
                return Some(GuiCpuAction::Halt {
                    pkt: Self::send_event(0, EVENT_HALT),
                    button_id: BTN_HALT,
                });
            }
            if (124.0..=133.0).contains(&y) {
                return Some(GuiCpuAction::Reset {
                    pkt: Self::send_event(0, EVENT_COLD_BOOT),
                    button_id: BTN_RESET,
                });
            }
        }

        // Hardware Debugger Extra Controls: STEP and RUN/STOP TOGGLE at x: 14..=44
        if (14.0..=44.0).contains(&x) {
            if (98.0..=110.0).contains(&y) {
                return Some(GuiCpuAction::Step {
                    pkt: Self::send_event(0, EVENT_STEP),
                    button_id: BTN_STEP,
                });
            }
            if (112.0..=124.0).contains(&y) {
                return Some(GuiCpuAction::ToggleRun {
                    pkt: Self::send_event(0, EVENT_TOGGLE_RUN),
                    button_id: BTN_TOGGLE_RUN,
                });
            }
        }

        None
    }

    /// Evaluates `native_screen_click` for the CPU screen directly from synced entity data.
    pub fn screen_click_button(at: [f32; 2], data: &[i32]) -> Option<i32> {
        let byte0 = data.first().copied().unwrap_or(2) as u8;
        let byte1 = data.get(1).copied().unwrap_or(1) as u8;
        let rbaddr = data.get(2).copied().unwrap_or(0) as u8;
        Self::click(at, byte0, byte1, rbaddr).map(|action| match action {
            GuiCpuAction::SetDiskId { button_id, .. }
            | GuiCpuAction::SetConsoleId { button_id, .. }
            | GuiCpuAction::SetBusId { button_id, .. }
            | GuiCpuAction::Start { button_id, .. }
            | GuiCpuAction::Halt { button_id, .. }
            | GuiCpuAction::Reset { button_id, .. }
            | GuiCpuAction::Step { button_id, .. }
            | GuiCpuAction::ToggleRun { button_id, .. } => button_id,
        })
    }

    /// Positions of status LED in GuiCPU (running: green, halted: red).
    pub fn led_position(is_running: bool) -> (f32, f32) {
        if is_running {
            (102.0, 99.0) // Green LED
        } else {
            (102.0, 112.0) // Red LED
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_registers_default_and_flags() {
        let regs = CpuRegisters::default();
        assert_eq!(regs.a, 0);
        assert_eq!(regs.x, 0);
        assert_eq!(regs.y, 0);
        assert_eq!(regs.sp, 0xFD);
        assert_eq!(regs.pc, 0x0400);
        assert_eq!(regs.status, 0x20);
        assert!(regs.halted);

        // Flags
        assert!(!regs.flag_n());
        assert!(!regs.flag_v());
        assert!(regs.flag_unused());
        assert!(!regs.flag_b());
        assert!(!regs.flag_d());
        assert!(!regs.flag_i());
        assert!(!regs.flag_z());
        assert!(!regs.flag_c());
        assert_eq!(regs.format_flags(), "--------");

        let mut custom = CpuRegisters {
            status: 0b11111111,
            ..Default::default()
        };
        assert!(custom.flag_n());
        assert!(custom.flag_v());
        assert!(custom.flag_b());
        assert!(custom.flag_d());
        assert!(custom.flag_i());
        assert!(custom.flag_z());
        assert!(custom.flag_c());
        assert_eq!(custom.format_flags(), "NV-BDIZC");

        custom.status = 0b10000001; // N and C
        assert_eq!(custom.format_flags(), "N- ----C");
        assert_eq!(custom.format_a(), "A: $00");
        assert_eq!(custom.format_sp(), "SP: $FD");
        assert_eq!(custom.format_pc(), "PC: $0400");
    }

    #[test]
    fn test_container_cpu_defaults_and_useability() {
        let container = ContainerCPU::default();
        assert_eq!(container.byte0, 2);
        assert_eq!(container.byte1, 1);
        assert_eq!(container.rbaddr, 0);
        assert!(!container.is_running);
        assert!(container.is_useable_by_player(16.0));
        assert!(!container.is_useable_by_player(65.0));
        assert_eq!(container.transfer_stack_in_slot(0), None);
    }

    #[test]
    fn test_container_cpu_progress_bar_sync() {
        let mut container = ContainerCPU::default();

        let updates = container.detect_changes(5, 1, 0, false);
        assert_eq!(updates, vec![ProgressBarUpdate { id: 0, val: 5 }]);
        assert_eq!(container.byte0, 5);

        let updates2 = container.detect_changes(5, 7, 3, true);
        assert_eq!(
            updates2,
            vec![
                ProgressBarUpdate { id: 1, val: 7 },
                ProgressBarUpdate { id: 2, val: 3 },
                ProgressBarUpdate { id: 3, val: 1 },
            ]
        );
        assert_eq!(container.byte1, 7);
        assert_eq!(container.rbaddr, 3);
        assert!(container.is_running);

        // Update progress bar
        container.update_progress_bar(0, 10);
        assert_eq!(container.byte0, 10);
        container.update_progress_bar(1, 12);
        assert_eq!(container.byte1, 12);
        container.update_progress_bar(2, 4);
        assert_eq!(container.rbaddr, 4);
        container.update_progress_bar(3, 0);
        assert!(!container.is_running);
        assert!(container.registers.halted);
    }

    #[test]
    fn test_container_cpu_packet_events() {
        let mut container = ContainerCPU::default();

        // Event 1: Disk ID
        let p1 = GuiCPU::send_simple(0, EVENT_SET_DISK_ID, 42);
        container.handle_gui_event(&p1);
        assert_eq!(container.byte0, 42);

        // Event 2: Console ID
        let p2 = GuiCPU::send_simple(0, EVENT_SET_CONSOLE_ID, 17);
        container.handle_gui_event(&p2);
        assert_eq!(container.byte1, 17);

        // Event 3: Bus ID
        let p3 = GuiCPU::send_simple(0, EVENT_SET_BUS_ID, 8);
        container.handle_gui_event(&p3);
        assert_eq!(container.rbaddr, 8);

        // Event 4: Warm boot / START
        let p4 = GuiCPU::send_event(0, EVENT_WARM_BOOT);
        container.handle_gui_event(&p4);
        assert!(container.is_running);
        assert!(!container.registers.halted);

        // Event 5: Halt
        let p5 = GuiCPU::send_event(0, EVENT_HALT);
        container.handle_gui_event(&p5);
        assert!(!container.is_running);
        assert!(container.registers.halted);

        // Event 6: Reset / Cold boot
        let p6 = GuiCPU::send_event(0, EVENT_COLD_BOOT);
        container.handle_gui_event(&p6);
        assert!(!container.is_running);
        assert_eq!(container.registers.pc, 0x0400);
        assert_eq!(container.registers.sp, 0xFD);
    }

    #[test]
    fn test_cpu_sync_and_single_step_events() {
        let mut cpu = Cpu6502::new();
        // Load simple program: LDA #$42, NOP, BRK
        cpu.load_program(0x0400, &[0xA9, 0x42, 0xEA, 0x00]);
        let mut container = ContainerCPU::default();
        container.sync_from_cpu(&cpu);
        assert_eq!(container.registers.pc, 0x0400);
        assert_eq!(container.registers.a, 0x00);

        // Single step 1: executes LDA #$42
        let p_step = GuiCPU::send_event(0, EVENT_STEP);
        container.handle_gui_event_on_cpu(&mut cpu, &p_step);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(container.registers.a, 0x42);
        assert_eq!(cpu.pc, 0x0402);
        assert_eq!(container.registers.pc, 0x0402);

        // Single step 2: executes NOP
        container.handle_gui_event_on_cpu(&mut cpu, &p_step);
        assert_eq!(cpu.pc, 0x0403);
        assert_eq!(container.registers.pc, 0x0403);

        // Single step 3: executes BRK
        container.handle_gui_event_on_cpu(&mut cpu, &p_step);
        assert!(cpu.halted);
        assert!(container.registers.halted);
    }

    #[test]
    fn test_gui_cpu_click_dispatch() {
        // Disk ID bit 0 click (x: 14..=26, y: 57..=89)
        let click1 = GuiCPU::click([20.0, 60.0], 2, 1, 0);
        match click1 {
            Some(GuiCpuAction::SetDiskId {
                new_val,
                bit,
                button_id,
                ..
            }) => {
                assert_eq!(bit, 0);
                assert_eq!(new_val, 3); // 2 ^ 1 = 3
                assert_eq!(button_id, BTN_DISK_BIT_BASE);
            }
            _ => panic!("Expected SetDiskId"),
        }

        // Console ID bit 2 click (x: 118 + 2*12 = 142..=154, y: 57..=89)
        let click2 = GuiCPU::click([145.0, 70.0], 2, 1, 0);
        match click2 {
            Some(GuiCpuAction::SetConsoleId {
                new_val,
                bit,
                button_id,
                ..
            }) => {
                assert_eq!(bit, 2);
                assert_eq!(new_val, 5); // 1 ^ 4 = 5
                assert_eq!(button_id, BTN_CONSOLE_BIT_BASE + 2);
            }
            _ => panic!("Expected SetConsoleId"),
        }

        // Bus ID bit 1 click (x: 118 + 1*12 = 130..=142, y: 101..=133)
        let click3 = GuiCPU::click([135.0, 110.0], 2, 1, 0);
        match click3 {
            Some(GuiCpuAction::SetBusId {
                new_val,
                bit,
                button_id,
                ..
            }) => {
                assert_eq!(bit, 1);
                assert_eq!(new_val, 2); // 0 ^ 2 = 2
                assert_eq!(button_id, BTN_BUS_BIT_BASE + 1);
            }
            _ => panic!("Expected SetBusId"),
        }

        // START button (x: 87..=96, y: 98..=107)
        let click_start = GuiCPU::click([90.0, 100.0], 2, 1, 0);
        match click_start {
            Some(GuiCpuAction::Start { button_id, .. }) => {
                assert_eq!(button_id, BTN_START);
            }
            _ => panic!("Expected Start"),
        }

        // HALT button (x: 87..=96, y: 111..=120)
        let click_halt = GuiCPU::click([90.0, 115.0], 2, 1, 0);
        match click_halt {
            Some(GuiCpuAction::Halt { button_id, .. }) => {
                assert_eq!(button_id, BTN_HALT);
            }
            _ => panic!("Expected Halt"),
        }

        // RESET button (x: 87..=96, y: 124..=133)
        let click_reset = GuiCPU::click([90.0, 128.0], 2, 1, 0);
        match click_reset {
            Some(GuiCpuAction::Reset { button_id, .. }) => {
                assert_eq!(button_id, BTN_RESET);
            }
            _ => panic!("Expected Reset"),
        }

        // STEP button (x: 14..=44, y: 98..=110)
        let click_step = GuiCPU::click([25.0, 105.0], 2, 1, 0);
        match click_step {
            Some(GuiCpuAction::Step { button_id, .. }) => {
                assert_eq!(button_id, BTN_STEP);
            }
            _ => panic!("Expected Step"),
        }

        // Outside clicks
        assert_eq!(GuiCPU::click([0.0, 0.0], 2, 1, 0), None);
    }

    #[test]
    fn test_screen_click_button_and_apply_menu_button() {
        let data = vec![2, 1, 0, 0, 0, 0, 0, 0xFD, 0x0400, 0x20, 1];
        let btn = GuiCPU::screen_click_button([90.0, 100.0], &data);
        assert_eq!(btn, Some(BTN_START));

        let mut container = ContainerCPU::default();
        let mut cpu = Cpu6502::new();
        cpu.halt();
        assert!(cpu.halted);

        // Apply START button
        container.apply_menu_button(BTN_START, Some(&mut cpu));
        assert!(container.is_running);
        assert!(!cpu.halted);

        // Apply HALT button
        container.apply_menu_button(BTN_HALT, Some(&mut cpu));
        assert!(!container.is_running);
        assert!(cpu.halted);

        // Apply Disk bit 0 button
        container.apply_menu_button(BTN_DISK_BIT_BASE, Some(&mut cpu));
        assert_eq!(container.byte0, 3);
        assert_eq!(cpu.ram[0], 3);
    }

    #[test]
    fn test_nbt_and_data_round_trip() {
        let container = ContainerCPU {
            byte0: 4,
            byte1: 5,
            rbaddr: 6,
            is_running: true,
            registers: CpuRegisters {
                a: 0x12,
                x: 0x34,
                y: 0x56,
                sp: 0xEE,
                pc: 0x0800,
                status: 0x65,
                halted: false,
            },
        };

        let mut nbt = HashMap::new();
        container.write_nbt(&mut nbt);
        let from_nbt = ContainerCPU::from_nbt(&nbt);
        assert_eq!(container, from_nbt);

        let data = container.to_data();
        let from_data = ContainerCPU::from_data(&data);
        assert_eq!(container, from_data);
    }
}
