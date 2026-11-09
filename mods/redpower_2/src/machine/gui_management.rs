//! Management GUIs & Containers: Regulator, Manager, Assembler.
//!
//! Provides Level 3 GUI layouts, container definitions, and networking event synchronization for:
//! - Regulator (`redpower_2:regulator`): 27 slots (9 stock template, 9 buffer, 9 excess export filter).
//! - Manager (`redpower_2:manager`): 24 slots (6 columns x 4 rows) for inventory synchronization.
//! - Assembler (`redpower_2:assembler`): 34 slots (16 template selector slots, 18 component storage slots).

pub use super::logistics_gui::{
    regulator_layout, Layout, BTN_COLOR_NEXT, BTN_COLOR_PREV, BTN_MODE_NEXT, BTN_MODE_PREV,
};
use serde::{Deserialize, Serialize};

/// Button IDs for management GUIs.
pub const BTN_CHANNEL_NEXT: i32 = 10;
pub const BTN_CHANNEL_PREV: i32 = 11;
pub const BTN_THRESHOLD_INC: i32 = 12;
pub const BTN_THRESHOLD_DEC: i32 = 13;

/// Full GUI descriptor for management screens matching contract specifications.
#[derive(Debug, Clone, PartialEq)]
pub struct ManagementGuiLayout {
    pub title: &'static str,
    pub size: [f32; 2],
    pub slots: Vec<[f32; 2]>,
    pub inventory: [f32; 2],
}

/// GUI layout for Regulator (`redpower_2:regulator`).
/// 27 slots: 3x3 template, 3x3 buffer, 3x3 excess filter.
pub fn regulator_gui_layout() -> ManagementGuiLayout {
    let base = regulator_layout();
    ManagementGuiLayout {
        title: "Regulator",
        size: base.size,
        slots: base.slots,
        inventory: base.inventory,
    }
}

/// GUI layout for Manager (`redpower_2:manager`).
/// 24 slots: 6 columns x 4 rows from [44, 18] with 18px pitch; inventory at [8, 104].
pub fn manager_layout() -> Layout {
    let mut slots = Vec::with_capacity(24);
    for row in 0..4 {
        for col in 0..6 {
            slots.push([44.0 + col as f32 * 18.0, 18.0 + row as f32 * 18.0]);
        }
    }
    Layout {
        size: [176.0, 186.0],
        slots,
        inventory: [8.0, 104.0],
    }
}

/// GUI layout descriptor for Manager.
pub fn manager_gui_layout() -> ManagementGuiLayout {
    let base = manager_layout();
    ManagementGuiLayout {
        title: "Manager",
        size: base.size,
        slots: base.slots,
        inventory: base.inventory,
    }
}

/// GUI layout for Assembler (`redpower_2:assembler`).
/// 34 slots:
/// - 16 template selectors (8 columns x 2 rows) at [8, 18] with 18px pitch.
/// - 18 storage slots (9 columns x 2 rows) at [8, 63] with 18px pitch.
///
/// Inventory at [8, 113].
pub fn assembler_layout() -> Layout {
    let mut slots = Vec::with_capacity(34);
    // 16 template selector slots
    for row in 0..2 {
        for col in 0..8 {
            slots.push([8.0 + col as f32 * 18.0, 18.0 + row as f32 * 18.0]);
        }
    }
    // 18 storage slots
    for row in 0..2 {
        for col in 0..9 {
            slots.push([8.0 + col as f32 * 18.0, 63.0 + row as f32 * 18.0]);
        }
    }
    Layout {
        size: [176.0, 195.0],
        slots,
        inventory: [8.0, 113.0],
    }
}

/// GUI layout descriptor for Assembler.
pub fn assembler_gui_layout() -> ManagementGuiLayout {
    let base = assembler_layout();
    ManagementGuiLayout {
        title: "Assembler",
        size: base.size,
        slots: base.slots,
        inventory: base.inventory,
    }
}

/// Sync settings for Regulator inventory maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RegulatorSettings {
    pub mode: u8,
    pub color_channel: u8,
    pub threshold: u32,
}

impl RegulatorSettings {
    pub fn apply_button(&mut self, id: i32) {
        match id {
            BTN_MODE_NEXT => self.mode = (self.mode + 1).rem_euclid(3),
            BTN_MODE_PREV => self.mode = (self.mode + 2).rem_euclid(3),
            BTN_COLOR_NEXT => self.color_channel = (self.color_channel + 1).rem_euclid(17),
            BTN_COLOR_PREV => self.color_channel = (self.color_channel + 16).rem_euclid(17),
            BTN_THRESHOLD_INC => self.threshold = self.threshold.saturating_add(1),
            BTN_THRESHOLD_DEC => self.threshold = self.threshold.saturating_sub(1),
            _ => {}
        }
    }
}

/// Sync settings for Manager stock balancing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ManagerSettings {
    pub channel: u8,
    pub priority: i32,
}

impl ManagerSettings {
    pub fn apply_button(&mut self, id: i32) {
        match id {
            BTN_COLOR_NEXT => self.channel = (self.channel + 1).rem_euclid(17),
            BTN_COLOR_PREV => self.channel = (self.channel + 16).rem_euclid(17),
            BTN_THRESHOLD_INC => self.priority = self.priority.saturating_add(1),
            BTN_THRESHOLD_DEC => self.priority = self.priority.saturating_sub(1),
            _ => {}
        }
    }
}

/// Sync settings for Assembler bundled cable deployment and plan item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AssemblerSettings {
    pub active_channel_mask: u16,
    pub plan_slot_active: bool,
}

impl AssemblerSettings {
    pub fn is_channel_active(&self, channel: usize) -> bool {
        if channel < 16 {
            (self.active_channel_mask & (1 << channel)) != 0
        } else {
            false
        }
    }

    pub fn set_channel_active(&mut self, channel: usize, active: bool) {
        if channel < 16 {
            if active {
                self.active_channel_mask |= 1 << channel;
            } else {
                self.active_channel_mask &= !(1 << channel);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulator_gui_contract_slots() {
        let layout = regulator_gui_layout();
        assert_eq!(layout.title, "Regulator");
        assert_eq!(layout.size, [211.0, 167.0]);
        assert_eq!(layout.slots.len(), 27);

        // Template slot 0
        assert_eq!(layout.slots[0], [8.0, 18.0]);
        // Template slot 8
        assert_eq!(layout.slots[8], [44.0, 54.0]);

        // Buffer slot 0 (index 9)
        assert_eq!(layout.slots[9], [80.0, 18.0]);
        // Buffer slot 8 (index 17)
        assert_eq!(layout.slots[17], [116.0, 54.0]);

        // Export filter slot 0 (index 18)
        assert_eq!(layout.slots[18], [152.0, 18.0]);
        // Export filter slot 8 (index 26)
        assert_eq!(layout.slots[26], [188.0, 54.0]);
    }

    #[test]
    fn test_manager_gui_contract_slots() {
        let layout = manager_gui_layout();
        assert_eq!(layout.title, "Manager");
        assert_eq!(layout.size, [176.0, 186.0]);
        assert_eq!(layout.slots.len(), 24);

        assert_eq!(layout.slots[0], [44.0, 18.0]);
        assert_eq!(layout.slots[5], [134.0, 18.0]);
        assert_eq!(layout.slots[6], [44.0, 36.0]);
        assert_eq!(layout.slots[23], [134.0, 72.0]);
        assert_eq!(layout.inventory, [8.0, 104.0]);
    }

    #[test]
    fn test_assembler_gui_contract_slots() {
        let layout = assembler_gui_layout();
        assert_eq!(layout.title, "Assembler");
        assert_eq!(layout.size, [176.0, 195.0]);
        assert_eq!(layout.slots.len(), 34);

        // Template selector slots (0..16)
        assert_eq!(layout.slots[0], [8.0, 18.0]);
        assert_eq!(layout.slots[7], [134.0, 18.0]);
        assert_eq!(layout.slots[8], [8.0, 36.0]);
        assert_eq!(layout.slots[15], [134.0, 36.0]);

        // Storage slots (16..34)
        assert_eq!(layout.slots[16], [8.0, 63.0]);
        assert_eq!(layout.slots[24], [152.0, 63.0]);
        assert_eq!(layout.slots[25], [8.0, 81.0]);
        assert_eq!(layout.slots[33], [152.0, 81.0]);
    }

    #[test]
    fn test_management_button_sync() {
        let mut reg = RegulatorSettings::default();
        reg.apply_button(BTN_MODE_NEXT);
        assert_eq!(reg.mode, 1);
        reg.apply_button(BTN_COLOR_NEXT);
        assert_eq!(reg.color_channel, 1);
        reg.apply_button(BTN_THRESHOLD_INC);
        assert_eq!(reg.threshold, 1);

        let mut mgr = ManagerSettings::default();
        mgr.apply_button(BTN_COLOR_PREV);
        assert_eq!(mgr.channel, 16);
        mgr.apply_button(BTN_THRESHOLD_DEC);
        assert_eq!(mgr.priority, -1);

        let mut asm = AssemblerSettings::default();
        asm.set_channel_active(0, true);
        asm.set_channel_active(15, true);
        assert!(asm.is_channel_active(0));
        assert!(asm.is_channel_active(15));
        assert!(!asm.is_channel_active(1));
    }
}
