//! GUI layouts of the Buffer Chest, Item Detector, Filter and Battery Box
//! (`ContainerBufferChest`, `ContainerItemDetect`, `ContainerFilter`, `ContainerBatteryBox`
//! and their `Gui*` classes).

use super::logistics_gui::{Layout, BTN_COLOR_NEXT, BTN_COLOR_PREV, BTN_MODE_NEXT, BTN_MODE_PREV};

/// Buffer Chest: 5 columns x 4 rows from (44,18), slot index `row + col * 4`; 176x186.
pub fn buffer_layout() -> Layout {
    let mut slots = Vec::new();
    for i in 0..5 {
        for j in 0..4 {
            slots.push([44.0 + i as f32 * 18.0, 18.0 + j as f32 * 18.0]);
        }
    }
    Layout {
        size: [176.0, 186.0],
        slots,
        inventory: [8.0, 104.0],
    }
}

/// Item Detector and Filter: 3x3 template grid at (62,17); 176x166, inventory at (8,84).
pub fn grid9_layout() -> Layout {
    let mut slots = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            slots.push([62.0 + j as f32 * 18.0, 17.0 + i as f32 * 18.0]);
        }
    }
    Layout {
        size: [176.0, 166.0],
        slots,
        inventory: [8.0, 84.0],
    }
}

/// Battery Box: charge slot (120,27), drain slot (120,55); 176x170, inventory at (8,88).
pub fn battery_layout() -> Layout {
    Layout {
        size: [176.0, 170.0],
        slots: vec![[120.0, 27.0], [120.0, 55.0]],
        inventory: [8.0, 88.0],
    }
}

/// Wind Turbine: rotor slot (80,35); 176x167, inventory at (8,84).
pub fn turbine_layout() -> Layout {
    Layout {
        size: [176.0, 167.0],
        slots: vec![[80.0, 35.0]],
        inventory: [8.0, 84.0],
    }
}

/// `GuiFilter.a(int,int,int)`: colour square x118..129, y55..66.
pub fn filter_click(at: [f32; 2], right: bool) -> Option<i32> {
    if (118.0..=129.0).contains(&at[0]) && (55.0..=66.0).contains(&at[1]) {
        Some(if right {
            BTN_COLOR_PREV
        } else {
            BTN_COLOR_NEXT
        })
    } else {
        None
    }
}

/// `GuiItemDetect.a(int,int,int)`: mode button x117..131, y54..68.
pub fn detector_click(at: [f32; 2], right: bool) -> Option<i32> {
    if (117.0..=131.0).contains(&at[0]) && (54.0..=68.0).contains(&at[1]) {
        Some(if right { BTN_MODE_PREV } else { BTN_MODE_NEXT })
    } else {
        None
    }
}

/// Detector mode after a button: wraps over 0..=2.
pub fn detector_apply(mode: i32, id: i32) -> i32 {
    match id {
        BTN_MODE_NEXT => (mode + 1).rem_euclid(3),
        BTN_MODE_PREV => (mode - 1).rem_euclid(3),
        _ => mode,
    }
}

/// Filter colour after a button: wraps over 0..=16.
pub fn filter_apply(color: i32, id: i32) -> i32 {
    match id {
        BTN_COLOR_NEXT => (color + 1).rem_euclid(17),
        BTN_COLOR_PREV => (color - 1).rem_euclid(17),
        _ => color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_match_java() {
        let b = buffer_layout();
        assert_eq!(b.slots.len(), 20);
        assert_eq!(b.slots[1], [44.0, 36.0]);
        assert_eq!(b.slots[19], [116.0, 72.0]);
        assert_eq!(grid9_layout().slots[8], [98.0, 53.0]);
        assert_eq!(battery_layout().slots[1], [120.0, 55.0]);
    }

    #[test]
    fn buttons_wrap() {
        assert_eq!(filter_click([120.0, 60.0], true), Some(BTN_COLOR_PREV));
        assert_eq!(filter_apply(0, BTN_COLOR_PREV), 16);
        assert_eq!(detector_click([120.0, 60.0], false), Some(BTN_MODE_NEXT));
        assert_eq!(detector_apply(2, BTN_MODE_NEXT), 0);
        assert_eq!(detector_apply(0, BTN_MODE_PREV), 2);
        assert_eq!(detector_click([10.0, 10.0], false), None);
    }
}
