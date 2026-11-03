//! Retriever and Regulator GUI layout and button logic
//! (`ContainerRetriever`/`GuiRetriever`, `ContainerRegulator`/`GuiRegulator`).

/// Menu button IDs sent by `native_screen_click`.
pub const BTN_COLOR_NEXT: i32 = 1;
pub const BTN_COLOR_PREV: i32 = 2;
pub const BTN_MODE_NEXT: i32 = 3;
pub const BTN_MODE_PREV: i32 = 4;

/// Screen layout: size, slot positions, player inventory origin.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub size: [f32; 2],
    pub slots: Vec<[f32; 2]>,
    pub inventory: [f32; 2],
}

/// `ContainerRetriever`: 3x3 filter grid at (62,17), inventory at (8,84).
pub fn retriever_layout() -> Layout {
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

/// `ContainerRegulator`: three 3x3 blocks (template, buffer, excess) 72px apart; inventory at (26,86).
pub fn regulator_layout() -> Layout {
    let mut slots = Vec::new();
    for k in 0..3 {
        for i in 0..3 {
            for j in 0..3 {
                slots.push([
                    8.0 + j as f32 * 18.0 + k as f32 * 72.0,
                    18.0 + i as f32 * 18.0,
                ]);
            }
        }
    }
    Layout {
        size: [211.0, 167.0],
        slots,
        inventory: [26.0, 86.0],
    }
}

/// `GuiRetriever.a(int,int,int)`: colour square x118..129, mode x45..58, both y55..66.
pub fn retriever_click(at: [f32; 2], right: bool) -> Option<i32> {
    let (x, y) = (at[0], at[1]);
    if !(55.0..=66.0).contains(&y) {
        return None;
    }
    if (118.0..=129.0).contains(&x) {
        return Some(if right {
            BTN_COLOR_PREV
        } else {
            BTN_COLOR_NEXT
        });
    }
    if (45.0..=58.0).contains(&x) {
        return Some(if right { BTN_MODE_PREV } else { BTN_MODE_NEXT });
    }
    None
}

/// `GuiRegulator.a(int,int,int)`: x136..147; colour y56..67, mode y19..32.
pub fn regulator_click(at: [f32; 2], right: bool) -> Option<i32> {
    let (x, y) = (at[0], at[1]);
    if !(136.0..=147.0).contains(&x) {
        return None;
    }
    if (56.0..=67.0).contains(&y) {
        return Some(if right {
            BTN_COLOR_PREV
        } else {
            BTN_COLOR_NEXT
        });
    }
    if (19.0..=32.0).contains(&y) {
        return Some(if right { BTN_MODE_PREV } else { BTN_MODE_NEXT });
    }
    None
}

/// Apply a button to `(color, mode)`: colour wraps 0..=16, mode wraps 0..=1.
pub fn apply_button(color: i32, mode: i32, id: i32) -> (i32, i32) {
    match id {
        BTN_COLOR_NEXT => ((color + 1).rem_euclid(17), mode),
        BTN_COLOR_PREV => ((color - 1).rem_euclid(17), mode),
        BTN_MODE_NEXT => (color, (mode + 1).rem_euclid(2)),
        BTN_MODE_PREV => (color, (mode - 1).rem_euclid(2)),
        _ => (color, mode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_match_java() {
        assert_eq!(retriever_layout().slots.len(), 9);
        assert_eq!(retriever_layout().slots[4], [80.0, 35.0]);
        let r = regulator_layout();
        assert_eq!(r.slots.len(), 27);
        assert_eq!(r.slots[9], [80.0, 18.0]);
        assert_eq!(r.slots[26], [188.0, 54.0]);
    }

    #[test]
    fn clicks_and_wrap() {
        assert_eq!(retriever_click([120.0, 60.0], false), Some(BTN_COLOR_NEXT));
        assert_eq!(retriever_click([50.0, 60.0], true), Some(BTN_MODE_PREV));
        assert_eq!(regulator_click([140.0, 25.0], false), Some(BTN_MODE_NEXT));
        assert_eq!(regulator_click([10.0, 25.0], false), None);
        assert_eq!(apply_button(16, 0, BTN_COLOR_NEXT), (0, 0));
        assert_eq!(apply_button(0, 0, BTN_COLOR_PREV), (16, 0));
        assert_eq!(apply_button(0, 1, BTN_MODE_NEXT), (0, 0));
    }
}
