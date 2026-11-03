//! Sorting Machine GUI layout and button logic (`ContainerSorter`/`GuiSorter`).
//! 40 filter slots (8 columns x 5 rows), 8 column colours, mode 0..=6,
//! auto mode 0..=2, default colour 0..=16.

use super::logistics_gui::Layout;

pub const SLOT_COUNT: usize = 40;
pub const COLUMNS: usize = 8;
pub const MODE_MAX: i32 = 6;
pub const AUTOMODE_MAX: i32 = 2;
pub const COLOR_MAX: i32 = 16;

pub const BTN_MODE_NEXT: i32 = 1;
pub const BTN_MODE_PREV: i32 = 2;
pub const BTN_AUTO_NEXT: i32 = 3;
pub const BTN_AUTO_PREV: i32 = 4;
pub const BTN_DEFCOLOR_NEXT: i32 = 5;
pub const BTN_DEFCOLOR_PREV: i32 = 6;
/// Column colour button: `BTN_COLUMN_NEXT + n` / `BTN_COLUMN_PREV + n`, n in 0..8.
pub const BTN_COLUMN_NEXT: i32 = 10;
pub const BTN_COLUMN_PREV: i32 = 20;

/// Tile settings synced to the client in `entity.data` as
/// `[mode, automode, defcolor, colors[0..8]]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SorterSettings {
    pub mode: i32,
    pub automode: i32,
    pub defcolor: i32,
    pub colors: [i32; COLUMNS],
}

impl SorterSettings {
    pub fn to_data(&self) -> Vec<i32> {
        let mut v = vec![self.mode, self.automode, self.defcolor];
        v.extend_from_slice(&self.colors);
        v
    }

    pub fn from_data(d: &[i32]) -> Self {
        let g = |i: usize| d.get(i).copied().unwrap_or(0);
        let mut s = Self {
            mode: g(0),
            automode: g(1),
            defcolor: g(2),
            colors: [0; COLUMNS],
        };
        for n in 0..COLUMNS {
            s.colors[n] = g(3 + n);
        }
        s
    }

    /// Apply a menu button (wraps like `GuiSorter`).
    pub fn apply(&mut self, id: i32) {
        let wrap = |v: i32, d: i32, max: i32| (v + d).rem_euclid(max + 1);
        match id {
            BTN_MODE_NEXT => self.mode = wrap(self.mode, 1, MODE_MAX),
            BTN_MODE_PREV => self.mode = wrap(self.mode, -1, MODE_MAX),
            BTN_AUTO_NEXT => self.automode = wrap(self.automode, 1, AUTOMODE_MAX),
            BTN_AUTO_PREV => self.automode = wrap(self.automode, -1, AUTOMODE_MAX),
            BTN_DEFCOLOR_NEXT => self.defcolor = wrap(self.defcolor, 1, COLOR_MAX),
            BTN_DEFCOLOR_PREV => self.defcolor = wrap(self.defcolor, -1, COLOR_MAX),
            n if (BTN_COLUMN_NEXT..BTN_COLUMN_NEXT + COLUMNS as i32).contains(&n) => {
                let c = (n - BTN_COLUMN_NEXT) as usize;
                self.colors[c] = wrap(self.colors[c], 1, COLOR_MAX);
            }
            n if (BTN_COLUMN_PREV..BTN_COLUMN_PREV + COLUMNS as i32).contains(&n) => {
                let c = (n - BTN_COLUMN_PREV) as usize;
                self.colors[c] = wrap(self.colors[c], -1, COLOR_MAX);
            }
            _ => {}
        }
    }

    pub fn from_nbt(nbt: &std::collections::HashMap<String, String>) -> Self {
        let g = |k: &str| nbt.get(k).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
        let mut s = Self {
            mode: g("mode"),
            automode: g("automode"),
            defcolor: g("defcolor"),
            colors: [0; COLUMNS],
        };
        for n in 0..COLUMNS {
            s.colors[n] = g(&format!("color{n}"));
        }
        s
    }

    pub fn write_nbt(&self, nbt: &mut std::collections::HashMap<String, String>) {
        nbt.insert("mode".into(), self.mode.to_string());
        nbt.insert("automode".into(), self.automode.to_string());
        nbt.insert("defcolor".into(), self.defcolor.to_string());
        for (n, c) in self.colors.iter().enumerate() {
            nbt.insert(format!("color{n}"), c.to_string());
        }
    }
}

/// `ContainerSorter`: 8x5 slots at (26,18) pitch 18, inventory at (8,140), 176x222.
pub fn layout() -> Layout {
    let mut slots = Vec::with_capacity(SLOT_COUNT);
    for i in 0..5 {
        for j in 0..COLUMNS {
            slots.push([26.0 + 18.0 * j as f32, 18.0 + 18.0 * i as f32]);
        }
    }
    Layout {
        size: [176.0, 222.0],
        slots,
        inventory: [8.0, 140.0],
    }
}

/// `GuiSorter.a(int,int,int)`: left click increments, right decrements.
pub fn click(at: [f32; 2], right: bool, mode: i32) -> Option<i32> {
    let (x, y) = (at[0], at[1]);
    if (7.0..=21.0).contains(&x) {
        if (91.0..=105.0).contains(&y) {
            return Some(if right { BTN_MODE_PREV } else { BTN_MODE_NEXT });
        }
        if (73.0..=87.0).contains(&y) {
            return Some(if right { BTN_AUTO_PREV } else { BTN_AUTO_NEXT });
        }
    }
    if (110.0..=121.0).contains(&y) {
        for n in 0..COLUMNS as i32 {
            let x0 = 28.0 + 18.0 * n as f32;
            if (x0..=x0 + 11.0).contains(&x) {
                return Some(
                    if right {
                        BTN_COLUMN_PREV
                    } else {
                        BTN_COLUMN_NEXT
                    } + n,
                );
            }
        }
        if (mode == 4 || mode == 6) && (7.0..=21.0).contains(&x) {
            return Some(if right {
                BTN_DEFCOLOR_PREV
            } else {
                BTN_DEFCOLOR_NEXT
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_matches_java() {
        let l = layout();
        assert_eq!(l.slots.len(), 40);
        assert_eq!(l.slots[0], [26.0, 18.0]);
        assert_eq!(l.slots[39], [152.0, 90.0]);
        assert_eq!(l.inventory, [8.0, 140.0]);
    }

    #[test]
    fn buttons_wrap() {
        let mut s = SorterSettings::default();
        s.apply(BTN_MODE_PREV);
        assert_eq!(s.mode, 6);
        s.apply(BTN_MODE_NEXT);
        assert_eq!(s.mode, 0);
        s.apply(BTN_AUTO_PREV);
        assert_eq!(s.automode, 2);
        s.apply(BTN_COLUMN_PREV + 3);
        assert_eq!(s.colors[3], 16);
        s.apply(BTN_COLUMN_NEXT + 3);
        assert_eq!(s.colors[3], 0);
        assert_eq!(SorterSettings::from_data(&s.to_data()), s);
    }

    #[test]
    fn clicks() {
        assert_eq!(click([10.0, 95.0], false, 0), Some(BTN_MODE_NEXT));
        assert_eq!(click([10.0, 80.0], true, 0), Some(BTN_AUTO_PREV));
        assert_eq!(
            click([30.0 + 18.0 * 2.0, 115.0], false, 0),
            Some(BTN_COLUMN_NEXT + 2)
        );
        assert_eq!(click([10.0, 115.0], false, 0), None);
        assert_eq!(click([10.0, 115.0], false, 4), Some(BTN_DEFCOLOR_NEXT));
    }
}
