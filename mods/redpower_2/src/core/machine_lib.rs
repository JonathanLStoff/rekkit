//! `MachineLib` port: sided inventory ranges, stack insertion, item comparison and the
//! `FilterMap` matching algorithms used by the logistics machines (filter, sorter, retriever...).
//!
//! Pure logic over slices of `Option<ItemStack>`; the engine glue hands in the slots.

use rekkit_core::inventory::ItemStack;

/// Per-item facts `compareItem` needs from the item registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemMeta {
    /// `Item.getHasSubtypes()`: damage selects a distinct item.
    pub has_subtypes: bool,
    /// `ItemStack.getMaxDamage()` (0 for undamageable items).
    pub max_damage: u32,
}

/// Looks item facts up by id.
pub trait ItemCatalog {
    fn meta(&self, item_id: &str) -> ItemMeta;
}

/// Catalog where no item has subtypes or durability.
pub struct PlainCatalog;
impl ItemCatalog for PlainCatalog {
    fn meta(&self, _: &str) -> ItemMeta {
        ItemMeta {
            has_subtypes: false,
            max_damage: 0,
        }
    }
}

/// `MachineLib.compareItem`: 0 means "the same item" for filtering purposes.
pub fn compare_item(a: &ItemStack, b: &ItemStack, cat: &dyn ItemCatalog) -> i32 {
    if a.item_id != b.item_id {
        return if a.item_id < b.item_id { -1 } else { 1 };
    }
    if a.damage == b.damage {
        return 0;
    }
    let (da, db) = (a.damage as i32, b.damage as i32);
    if cat.meta(&a.item_id).has_subtypes {
        return da - db;
    }
    let max = cat.meta(&a.item_id).max_damage as i32;
    let bucket = |d: i32| {
        if d <= 1 {
            -1
        } else if d == max - 1 {
            1
        } else {
            0
        }
    };
    bucket(da) - bucket(db)
}

pub type Slots = [Option<ItemStack>];

/// Slot window of a sided inventory (`getStartInventorySide` / `getSizeInventorySide`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotRange {
    pub start: usize,
    pub len: usize,
}

impl SlotRange {
    pub fn all(total: usize) -> Self {
        Self {
            start: 0,
            len: total,
        }
    }

    /// `getSideInventory`/`addToInventoryCore`: the side window when the inventory is sided,
    /// otherwise the whole inventory. The window is clamped to the inventory.
    pub fn for_side(total: usize, sided: Option<(usize, usize)>) -> Self {
        match sided {
            Some((start, len)) => {
                let start = start.min(total);
                Self {
                    start,
                    len: len.min(total - start),
                }
            }
            None => Self::all(total),
        }
    }

    fn indices(self) -> std::ops::Range<usize> {
        self.start..self.start + self.len
    }
}

fn occupied(s: &Option<ItemStack>) -> Option<&ItemStack> {
    s.as_ref().filter(|i| i.count != 0)
}

/// `addToInventoryCore(inv, ist, start, len, act)`. With `act == false` only reports whether
/// something would fit. With `act` the stack is merged then placed; `ist` keeps what did not fit.
/// `limit` is the inventory stack limit (`getInventoryStackLimit`).
pub fn add_to_inventory(
    inv: &mut Slots,
    ist: &mut ItemStack,
    range: SlotRange,
    limit: u32,
    act: bool,
) -> bool {
    let end = (range.start + range.len).min(inv.len());
    #[allow(clippy::needless_range_loop)]
    for n in range.start..end {
        match &mut inv[n] {
            None => {
                if !act {
                    return true;
                }
            }
            Some(cur) => {
                if cur.can_merge(ist) {
                    let room = cur.max_stack_size.min(limit).saturating_sub(cur.count);
                    if room > 0 {
                        let si = room.min(ist.count);
                        if !act {
                            return true;
                        }
                        cur.count += si;
                        ist.count -= si;
                        if ist.count == 0 {
                            return true;
                        }
                    }
                }
            }
        }
    }
    if !act {
        return false;
    }
    for slot in inv.iter_mut().take(end).skip(range.start) {
        if slot.is_none() {
            if limit >= ist.count {
                *slot = Some(ist.clone());
                ist.count = 0;
                return true;
            }
            let mut part = ist.clone();
            part.count = limit;
            ist.count -= limit;
            *slot = Some(part);
        }
    }
    false
}

/// Pattern table: every non-empty template slot, grouped by "same item" (per `compare_item`),
/// each group listing its slot indices in ascending order.
pub struct FilterMap<'a> {
    pub filter: Vec<Option<ItemStack>>,
    groups: Vec<(ItemStack, Vec<usize>)>,
    cat: &'a dyn ItemCatalog,
}

impl<'a> FilterMap<'a> {
    pub fn new(filter: &Slots, cat: &'a dyn ItemCatalog) -> Self {
        let mut groups: Vec<(ItemStack, Vec<usize>)> = Vec::new();
        for (i, f) in filter.iter().enumerate() {
            if let Some(f) = occupied(f) {
                match groups
                    .iter_mut()
                    .find(|(k, _)| compare_item(k, f, cat) == 0)
                {
                    Some((_, v)) => v.push(i),
                    None => groups.push((f.clone(), vec![i])),
                }
            }
        }
        Self {
            filter: filter.to_vec(),
            groups,
            cat,
        }
    }

    /// `makeFilterMap(ist, st, ln)`.
    pub fn from_range(filter: &Slots, start: usize, len: usize, cat: &'a dyn ItemCatalog) -> Self {
        Self::new(&filter[start..start + len], cat)
    }

    pub fn size(&self) -> usize {
        self.groups.len()
    }

    fn slots_for(&self, ist: &ItemStack) -> Option<&Vec<usize>> {
        self.groups
            .iter()
            .find(|(k, _)| compare_item(k, ist, self.cat) == 0)
            .map(|(_, v)| v)
    }

    pub fn contains_key(&self, ist: &ItemStack) -> bool {
        self.slots_for(ist).is_some()
    }

    /// First template slot matching `ist`, or -1.
    pub fn first_match(&self, ist: &ItemStack) -> i32 {
        self.slots_for(ist).map_or(-1, |v| v[0] as i32)
    }

    fn count_at(&self, i: usize) -> u32 {
        self.filter[i].as_ref().map_or(0, |s| s.count)
    }

    /// `genMatchCounts`: wanted total per group, stored at the group's first slot.
    pub fn gen_match_counts(&self) -> Vec<u32> {
        let mut tr = vec![0; self.filter.len()];
        for (_, slots) in &self.groups {
            tr[slots[0]] = slots.iter().map(|&m| self.count_at(m)).sum();
        }
        tr
    }

    /// `decMatchCount`: takes up to `ist.count` from the remaining wanted count.
    pub fn dec_match_count(&self, mc: &mut [u32], ist: &ItemStack) -> u32 {
        let Some(v) = self.slots_for(ist) else {
            return 0;
        };
        let n = v[0];
        let tr = mc[n].min(ist.count);
        mc[n] -= tr;
        tr
    }

    /// `decMatchCounts` over an inventory window.
    pub fn dec_match_counts(&self, mc: &mut [u32], inv: &Slots, range: SlotRange) {
        for n in range.indices().take_while(|&n| n < inv.len()) {
            if let Some(s) = occupied(&inv[n]) {
                self.dec_match_count(mc, s);
            }
        }
    }

    /// `matchOneStack`: does the window hold template slot `pos` (any item when it is empty)?
    pub fn match_one_stack(&self, inv: &Slots, range: SlotRange, pos: usize) -> bool {
        let matchst = self.filter[pos].as_ref();
        let mut fc = matchst.map_or(1, |m| m.count) as i64;
        for n in range.indices().take_while(|&n| n < inv.len()) {
            if let Some(s) = occupied(&inv[n]) {
                let Some(m) = matchst else { return true };
                if compare_item(m, s, self.cat) == 0 {
                    fc -= s.count.min(fc.max(0) as u32) as i64;
                    if fc <= 0 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// `matchAnyStack`: first template slot whose wanted count the window satisfies, or -1.
    pub fn match_any_stack(&self, inv: &Slots, range: SlotRange) -> i32 {
        let mut mc = vec![0u32; self.filter.len()];
        for n in range.indices().take_while(|&n| n < inv.len()) {
            if let Some(s) = occupied(&inv[n]) {
                if let Some(v) = self.slots_for(s) {
                    for &m in v {
                        mc[m] += s.count;
                        if mc[m] >= self.count_at(m) {
                            return m as i32;
                        }
                    }
                }
            }
        }
        -1
    }

    /// `matchAnyStackCol`: like `match_any_stack` restricted to sorter column `col`
    /// (slot `m` is column `m & 7`, row `m >> 3`).
    pub fn match_any_stack_col(&self, inv: &Slots, range: SlotRange, col: usize) -> i32 {
        let mut mc = [0u32; 5];
        for n in range.indices().take_while(|&n| n < inv.len()) {
            if let Some(s) = occupied(&inv[n]) {
                if let Some(v) = self.slots_for(s) {
                    for &m in v.iter().filter(|&&m| m & 7 == col) {
                        let row = m >> 3;
                        if row >= 5 {
                            continue;
                        }
                        mc[row] += s.count;
                        if mc[row] >= self.count_at(m) {
                            return m as i32;
                        }
                    }
                }
            }
        }
        -1
    }

    /// `matchAllCol`: every non-empty template of column `col` is satisfied (and one exists).
    pub fn match_all_col(&self, inv: &Slots, range: SlotRange, col: usize) -> bool {
        let mut mc = [0u32; 5];
        for n in range.indices().take_while(|&n| n < inv.len()) {
            if let Some(s) = occupied(&inv[n]) {
                if let Some(v) = self.slots_for(s) {
                    let mut ss = s.count;
                    for &m in v.iter().filter(|&&m| m & 7 == col) {
                        let row = m >> 3;
                        if row >= 5 {
                            continue;
                        }
                        let s1 = ss.min(self.count_at(m).saturating_sub(mc[row]));
                        mc[row] += s1;
                        ss -= s1;
                        if ss == 0 {
                            break;
                        }
                    }
                }
            }
        }
        let mut any = false;
        for (row, have) in mc.iter().enumerate() {
            let idx = row * 8 + col;
            if idx >= self.filter.len() {
                break;
            }
            let ct = self.count_at(idx);
            if ct != 0 {
                any = true;
                if ct > *have {
                    return false;
                }
            }
        }
        any
    }
}

/// `emptyInventory`.
pub fn empty_inventory(inv: &Slots, range: SlotRange) -> bool {
    range
        .indices()
        .take_while(|&n| n < inv.len())
        .all(|n| occupied(&inv[n]).is_none())
}

/// `collectOneStack`: removes and returns the wanted stack (first stack when `matchst` is None).
pub fn collect_one_stack(
    inv: &mut Slots,
    range: SlotRange,
    matchst: Option<&ItemStack>,
    cat: &dyn ItemCatalog,
) -> Option<ItemStack> {
    let mut tr: Option<ItemStack> = None;
    let mut fc = matchst.map_or(1, |m| m.count);
    let end = (range.start + range.len).min(inv.len());
    #[allow(clippy::needless_range_loop)]
    for n in range.start..end {
        let Some(s) = occupied(&inv[n]) else { continue };
        let Some(m) = matchst else {
            return inv[n].take();
        };
        if compare_item(m, s, cat) == 0 {
            let take = s.count.min(fc);
            let slot = inv[n].as_mut().unwrap();
            let mut part = slot.clone();
            part.count = take;
            slot.count -= take;
            if slot.count == 0 {
                inv[n] = None;
            }
            match &mut tr {
                None => tr = Some(part),
                Some(t) => t.count += take,
            }
            fc -= take;
            if fc == 0 {
                break;
            }
        }
    }
    tr
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(id: &str, n: u32) -> Option<ItemStack> {
        Some(ItemStack::new(id, n))
    }
    fn all(len: usize) -> SlotRange {
        SlotRange::all(len)
    }

    #[test]
    fn side_window_clamped() {
        assert_eq!(SlotRange::for_side(9, None), SlotRange { start: 0, len: 9 });
        assert_eq!(
            SlotRange::for_side(9, Some((7, 5))),
            SlotRange { start: 7, len: 2 }
        );
    }

    #[test]
    fn compare_damage_rules() {
        let mut a = ItemStack::new("x", 1);
        let mut b = ItemStack::new("x", 1);
        a.damage = 5;
        b.damage = 9;
        assert_eq!(compare_item(&a, &b, &PlainCatalog), 0);
        a.damage = 0;
        assert_eq!(compare_item(&a, &b, &PlainCatalog), -1);
        struct Sub;
        impl ItemCatalog for Sub {
            fn meta(&self, _: &str) -> ItemMeta {
                ItemMeta {
                    has_subtypes: true,
                    max_damage: 0,
                }
            }
        }
        assert_eq!(compare_item(&a, &b, &Sub), -9);
        assert!(compare_item(&ItemStack::new("a", 1), &ItemStack::new("b", 1), &Sub) < 0);
    }

    #[test]
    fn add_merges_then_fills_empty() {
        let mut inv = vec![st("a", 60), None, st("b", 1)];
        let mut s = ItemStack::new("a", 10);
        assert!(add_to_inventory(&mut inv, &mut s, all(3), 64, false));
        assert!(add_to_inventory(&mut inv, &mut s, all(3), 64, true));
        assert_eq!(inv[0].as_ref().unwrap().count, 64);
        assert_eq!(inv[1].as_ref().unwrap().count, 6);
    }

    #[test]
    fn add_respects_window_and_overflow() {
        let mut inv = vec![None, None, st("b", 1)];
        let mut s = ItemStack::new("a", 100);
        assert!(add_to_inventory(
            &mut inv,
            &mut s,
            SlotRange { start: 0, len: 2 },
            64,
            true
        ));
        assert_eq!(inv[1].as_ref().unwrap().count, 36);
        assert_eq!(s.count, 0);
        let mut inv = vec![None, st("b", 1)];
        let mut s = ItemStack::new("a", 100);
        assert!(!add_to_inventory(&mut inv, &mut s, all(2), 64, true));
        assert_eq!(inv[0].as_ref().unwrap().count, 64);
        assert_eq!(s.count, 36);
        assert!(!add_to_inventory(
            &mut vec![st("b", 1)],
            &mut ItemStack::new("a", 1),
            all(1),
            64,
            false
        ));
    }

    #[test]
    fn filter_counts_and_dec() {
        let f = vec![st("a", 2), st("b", 1), st("a", 3), None];
        let fm = FilterMap::new(&f, &PlainCatalog);
        assert_eq!(fm.size(), 2);
        assert_eq!(fm.first_match(&ItemStack::new("a", 1)), 0);
        assert_eq!(fm.first_match(&ItemStack::new("z", 1)), -1);
        let mut mc = fm.gen_match_counts();
        assert_eq!(mc, vec![5, 1, 0, 0]);
        assert_eq!(fm.dec_match_count(&mut mc, &ItemStack::new("a", 4)), 4);
        assert_eq!(fm.dec_match_count(&mut mc, &ItemStack::new("a", 4)), 1);
        assert_eq!(mc[0], 0);
    }

    #[test]
    fn match_one_and_any() {
        let f = vec![st("a", 3), None];
        let fm = FilterMap::new(&f, &PlainCatalog);
        let inv = vec![st("a", 2), st("a", 1), st("c", 1)];
        assert!(fm.match_one_stack(&inv, all(3), 0));
        assert!(fm.match_one_stack(&inv, all(3), 1));
        assert!(!fm.match_one_stack(&inv, SlotRange { start: 0, len: 1 }, 0));
        assert_eq!(fm.match_any_stack(&inv, all(3)), 0);
        assert_eq!(fm.match_any_stack(&inv, SlotRange { start: 2, len: 1 }), -1);
    }

    #[test]
    fn column_matching() {
        let mut f: Vec<Option<ItemStack>> = vec![None; 40];
        f[2] = st("a", 2); // row 0 col 2
        f[10] = st("b", 1); // row 1 col 2
        let fm = FilterMap::new(&f, &PlainCatalog);
        let inv = vec![st("a", 2), st("b", 1)];
        assert_eq!(fm.match_any_stack_col(&inv, all(2), 2), 2);
        assert_eq!(fm.match_any_stack_col(&inv, all(2), 3), -1);
        assert!(fm.match_all_col(&inv, all(2), 2));
        assert!(!fm.match_all_col(&inv[..1], all(1), 2));
        assert!(!fm.match_all_col(&inv, all(2), 5));
    }

    #[test]
    fn collect_and_empty() {
        let mut inv = vec![st("a", 2), st("b", 4), st("a", 5)];
        let m = ItemStack::new("a", 4);
        let got = collect_one_stack(&mut inv, all(3), Some(&m), &PlainCatalog).unwrap();
        assert_eq!(got.count, 4);
        assert!(inv[0].is_none());
        assert_eq!(inv[2].as_ref().unwrap().count, 3);
        let first = collect_one_stack(&mut inv, all(3), None, &PlainCatalog).unwrap();
        assert_eq!(first.item_id, "b");
        assert!(!empty_inventory(&inv, all(3)));
        assert!(empty_inventory(&inv, SlotRange { start: 0, len: 2 }));
    }
}
