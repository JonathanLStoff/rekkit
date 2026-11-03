//! Project Table (`TileAdvBench`, `ContainerAdvBench`, `GuiAdvBench`, `SlotCraftRefill`).
//!
//! Tile layout (28 slots): 0..9 persistent 3x3 grid, 9 plan, 10..28 storage (18).
//! Container layout (29 slots, the order of the screen): 0..9 grid, 9 plan,
//! 10 craft result (computed, never stored), 11..29 storage.
//! Pure logic; the recipe lookup itself is the engine's and is passed in.
#![allow(clippy::needless_range_loop)]

use crate::{Inventory, ItemStack};

pub const BLOCK_ID: &str = "redpower_2:project_table";
pub const BLANK_PLAN: &str = "redpower_2:blank_plan";
pub const FULL_PLAN: &str = "redpower_2:plan";

pub const TILE_SLOTS: usize = 28;
pub const GRID: usize = 9;
pub const PLAN_SLOT: usize = 9;
pub const STORAGE_START: usize = 10;
pub const STORAGE_SIZE: usize = 18;
pub const CONTAINER_SLOTS: usize = 29;
pub const CONTAINER_RESULT: usize = 10;
pub const CONTAINER_STORAGE_START: usize = 11;
/// `GuiAdvBench` "write plan" button area (x 18..=32, y 55..=69) and its event id.
pub const PLAN_BUTTON: [f32; 4] = [18.0, 55.0, 32.0, 69.0];
pub const EVENT_WRITE_PLAN: i32 = 1;
pub const SIZE: [f32; 2] = [176.0, 222.0];
pub const INVENTORY_ORIGIN: [f32; 2] = [8.0, 140.0];
pub const FULL_MASK: u32 = 511;

/// Slot positions in container order (grid, plan, result, storage).
pub fn slot_positions() -> Vec<[f32; 2]> {
    let mut v = Vec::with_capacity(CONTAINER_SLOTS);
    for i in 0..3 {
        for j in 0..3 {
            v.push([48.0 + j as f32 * 18.0, 18.0 + i as f32 * 18.0]);
        }
    }
    v.push([17.0, 36.0]);
    v.push([143.0, 36.0]);
    for i in 0..2 {
        for j in 0..9 {
            v.push([8.0 + j as f32 * 18.0, 90.0 + i as f32 * 18.0]);
        }
    }
    v
}

/// Tile slot -> container slot.
pub fn tile_to_container(tile: usize) -> usize {
    if tile < STORAGE_START {
        tile
    } else {
        tile + 1
    }
}

/// Container slot -> tile slot (`None` for the result slot).
pub fn container_to_tile(slot: usize) -> Option<usize> {
    match slot {
        CONTAINER_RESULT => None,
        s if s < CONTAINER_RESULT => Some(s),
        s => Some(s - 1),
    }
}

/// Container slots sided automation reaches: the storage rows only.
pub fn sided_container_slots() -> Vec<usize> {
    (CONTAINER_STORAGE_START..CONTAINER_SLOTS).collect()
}

/// Ore-dictionary-free item match (id and damage).
pub fn matches(a: &ItemStack, b: &ItemStack) -> bool {
    a.item_id == b.item_id && a.damage == b.damage
}

/// A written plan: the 9 ingredients (`requires`) and the crafted result.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    pub requires: [Option<ItemStack>; 9],
    pub result: Option<ItemStack>,
}

fn enc(s: &ItemStack) -> String {
    format!("{}@{}@{}", s.item_id, s.count, s.damage)
}

fn dec(s: &str) -> Option<ItemStack> {
    let mut p = s.rsplitn(3, '@');
    let damage = p.next()?.parse().ok()?;
    let count = p.next()?.parse().ok()?;
    let id = p.next()?;
    let mut st = ItemStack::new(id, count);
    st.damage = damage;
    Some(st)
}

impl Plan {
    /// `result|slot=stack;slot=stack` for the entity's string map.
    pub fn encode(&self) -> String {
        let r = self.result.as_ref().map(enc).unwrap_or_default();
        let req: Vec<String> = self
            .requires
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|s| format!("{i}={}", enc(s))))
            .collect();
        format!("{r}|{}", req.join(";"))
    }

    pub fn decode(text: &str) -> Option<Plan> {
        let (r, req) = text.split_once('|')?;
        let mut plan = Plan {
            result: if r.is_empty() { None } else { dec(r) },
            ..Plan::default()
        };
        for part in req.split(';').filter(|p| !p.is_empty()) {
            let (i, s) = part.split_once('=')?;
            let i: usize = i.parse().ok()?;
            if i < 9 {
                plan.requires[i] = dec(s);
            }
        }
        Some(plan)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectTable {
    /// 28 tile slots.
    pub inv: Inventory,
    /// Data of the full plan in slot 9 (the item stack carries no NBT here).
    pub plan: Option<Plan>,
    /// Facing from the placer's yaw (`TileAdvBench.onBlockPlaced`).
    pub rotation: u8,
}

impl Default for ProjectTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectTable {
    pub fn new() -> Self {
        Self {
            inv: Inventory::new(TILE_SLOTS),
            plan: None,
            rotation: 0,
        }
    }

    /// `floor(yaw * 4 / 360 + 0.5) & 3`.
    pub fn rotation_from_yaw(yaw: f32) -> u8 {
        ((yaw * 4.0 / 360.0 + 0.5).floor() as i32 & 3) as u8
    }

    pub fn place(&mut self, yaw: f32) {
        self.rotation = Self::rotation_from_yaw(yaw);
    }

    fn at(&self, i: usize) -> Option<&ItemStack> {
        self.inv.slots[i].as_ref()
    }

    fn plan_items(&self) -> Option<&[Option<ItemStack>; 9]> {
        match self.at(PLAN_SLOT) {
            Some(p) if p.item_id == FULL_PLAN => self.plan.as_ref().map(|p| &p.requires),
            _ => None,
        }
    }

    /// First storage slot (tile index) holding a stack matching `a`.
    pub fn find_match(&self, a: &ItemStack) -> Option<usize> {
        (STORAGE_START..STORAGE_START + STORAGE_SIZE)
            .find(|&i| self.at(i).is_some_and(|t| t.count > 0 && matches(a, t)))
    }

    /// Bit i set when grid cell i is filled, not demanded by the plan, or
    /// coverable from storage (`getSatisfyMask`).
    pub fn satisfy_mask(&self) -> u32 {
        let items = self.plan_items();
        let mut bits = 0u32;
        for i in 0..GRID {
            if self.at(i).is_some() || items.is_none_or(|p| p[i].is_none()) {
                bits |= 1 << i;
            }
        }
        if bits == FULL_MASK {
            return FULL_MASK;
        }
        let Some(items) = items else { return bits };
        for i in 0..STORAGE_SIZE {
            let Some(test) = self.at(STORAGE_START + i) else {
                continue;
            };
            if test.count == 0 {
                continue;
            }
            let mut sc = test.count;
            for j in 0..GRID {
                if bits & (1 << j) == 0 && self.at(j).is_none() {
                    if let Some(want) = &items[j] {
                        if matches(want, test) {
                            bits |= 1 << j;
                            sc -= 1;
                            if sc == 0 {
                                break;
                            }
                        }
                    }
                }
            }
        }
        bits
    }

    /// The 3x3 handed to the recipe lookup: the grid, with plan cells that
    /// are empty filled from a matching storage stack (`ContainerAdvBench.a(la)`).
    /// `Java` only did this when the match index was above 0, always true here.
    pub fn crafting_grid(&self) -> [Option<ItemStack>; 9] {
        let items = self.plan_items();
        std::array::from_fn(|i| {
            let mut s = self.at(i).cloned();
            if s.is_none() {
                if let Some(want) = items.and_then(|p| p[i].as_ref()) {
                    if let Some(j) = self.find_match(want) {
                        s = self.at(j).cloned();
                    }
                }
            }
            s
        })
    }

    /// The result slot content: only when every cell is satisfied.
    pub fn craft_result(
        &self,
        lookup: impl Fn(&[Option<ItemStack>; 9]) -> Option<ItemStack>,
    ) -> Option<ItemStack> {
        if self.satisfy_mask() == FULL_MASK {
            lookup(&self.crafting_grid())
        } else {
            None
        }
    }

    /// `SlotCraftRefill.isLastUse`: true when taking the result once more
    /// could not be repeated from the grid and storage.
    pub fn is_last_use(&self) -> bool {
        let mut bits = 0u32;
        for i in 0..GRID {
            match self.at(i) {
                None => bits |= 1 << i,
                Some(s) if s.max_stack_size <= 1 || s.count > 1 => bits |= 1 << i,
                _ => {}
            }
        }
        if bits == FULL_MASK {
            return false;
        }
        for i in 0..STORAGE_SIZE {
            let Some(test) = self.at(STORAGE_START + i) else {
                continue;
            };
            if test.count == 0 {
                continue;
            }
            let mut sc = test.count;
            for j in 0..GRID {
                if bits & (1 << j) == 0 {
                    if let Some(st) = self.at(j) {
                        if matches(st, test) {
                            bits |= 1 << j;
                            sc -= 1;
                            if sc == 0 {
                                break;
                            }
                        }
                    }
                }
            }
        }
        bits != FULL_MASK
    }

    fn take_one(&mut self, slot: usize) -> Option<ItemStack> {
        let s = self.inv.slots[slot].as_mut()?;
        let mut one = s.clone();
        one.count = 1;
        s.count -= 1;
        if s.count == 0 {
            self.inv.slots[slot] = None;
        }
        Some(one)
    }

    /// Called when the player takes the result (`SlotCraftRefill.a`): plan
    /// cells that were empty are paid from storage, every filled cell loses
    /// one, then cells without a plan entry refill from storage.
    pub fn on_craft_taken(&mut self) {
        let plan: Option<[Option<ItemStack>; 9]> = self.plan_items().cloned();
        let cur: Vec<Option<ItemStack>> = (0..GRID).map(|i| self.at(i).cloned()).collect();
        if let Some(plan) = &plan {
            for i in 0..GRID {
                if cur[i].is_none() {
                    if let Some(want) = &plan[i] {
                        if let Some(p) = self.find_match(want) {
                            self.take_one(p);
                        }
                    }
                }
            }
        }
        for i in 0..GRID {
            if self.at(i).is_some() {
                self.take_one(i);
            }
        }
        for i in 0..GRID {
            let Some(c) = &cur[i] else { continue };
            if plan.as_ref().is_some_and(|p| p[i].is_some()) || self.at(i).is_some() {
                continue;
            }
            if let Some(idx) = self.find_match(c) {
                self.inv.slots[i] = self.take_one(idx);
            }
        }
    }

    /// `GuiEvent 1`: writes the plan when a blank plan sits in the plan slot.
    /// `result` is the current craft result; nothing happens without one.
    pub fn write_plan(&mut self, result: Option<ItemStack>) -> bool {
        let blank = self.at(PLAN_SLOT).is_some_and(|p| p.item_id == BLANK_PLAN);
        if !blank || result.is_none() {
            return false;
        }
        let requires = std::array::from_fn(|i| {
            self.at(i).map(|s| {
                let mut one = s.clone();
                one.count = 1;
                one
            })
        });
        self.plan = Some(Plan { requires, result });
        self.inv.slots[PLAN_SLOT] = Some(ItemStack::new(FULL_PLAN, 1).with_max_stack(1));
        true
    }

    /// Items dropped when broken: slots 0..27 (the original skips slot 27).
    pub fn removal_drops(&self) -> Vec<ItemStack> {
        self.inv.slots[..27]
            .iter()
            .flatten()
            .filter(|s| s.count > 0)
            .cloned()
            .collect()
    }

    /// Stack limit of the tile.
    pub fn set_slot(&mut self, slot: usize, stack: Option<ItemStack>) {
        self.inv.slots[slot] = stack.map(|mut s| {
            s.count = s.count.min(64);
            s
        });
        if self.plan.is_some() && self.at(PLAN_SLOT).is_none_or(|p| p.item_id != FULL_PLAN) {
            self.plan = None;
        }
    }

    /// Entity items in container order (result slot left empty).
    pub fn to_container_items(&self) -> Vec<Option<ItemStack>> {
        let mut v = vec![None; CONTAINER_SLOTS];
        for (t, s) in self.inv.slots.iter().enumerate() {
            v[tile_to_container(t)] = s.clone();
        }
        v
    }

    pub fn from_container_items(items: &[Option<ItemStack>], plan: Option<Plan>) -> Self {
        let mut t = Self::new();
        for (c, s) in items.iter().enumerate().take(CONTAINER_SLOTS) {
            if let Some(ti) = container_to_tile(c) {
                t.inv.slots[ti] = s.clone();
            }
        }
        t.plan = plan;
        t
    }

    pub fn save(&self, nbt: &mut std::collections::HashMap<String, String>) {
        nbt.insert("rotation".into(), self.rotation.to_string());
        match &self.plan {
            Some(p) => nbt.insert("plan".into(), p.encode()),
            None => nbt.remove("plan"),
        };
    }

    pub fn load_meta(&mut self, nbt: &std::collections::HashMap<String, String>) {
        self.rotation = nbt
            .get("rotation")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        self.plan = nbt.get("plan").and_then(|p| Plan::decode(p));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(id: &str, n: u32) -> Option<ItemStack> {
        Some(ItemStack::new(id, n))
    }

    #[test]
    fn layout_matches_contract() {
        let p = slot_positions();
        assert_eq!(p.len(), 29);
        assert_eq!(p[0], [48.0, 18.0]);
        assert_eq!(p[9], [17.0, 36.0]);
        assert_eq!(p[10], [143.0, 36.0]);
        assert_eq!(p[11], [8.0, 90.0]);
        assert_eq!(p[28], [152.0, 108.0]);
    }

    #[test]
    fn index_mapping_roundtrips() {
        for t in 0..TILE_SLOTS {
            assert_eq!(container_to_tile(tile_to_container(t)), Some(t));
        }
        assert_eq!(container_to_tile(CONTAINER_RESULT), None);
    }

    #[test]
    fn rotation_from_yaw() {
        assert_eq!(ProjectTable::rotation_from_yaw(0.0), 0);
        assert_eq!(ProjectTable::rotation_from_yaw(90.0), 1);
        assert_eq!(ProjectTable::rotation_from_yaw(-90.0), 3);
    }

    #[test]
    fn plan_roundtrip_and_write() {
        let mut t = ProjectTable::new();
        t.inv.slots[0] = st("minecraft:stick", 4);
        t.inv.slots[PLAN_SLOT] = st(BLANK_PLAN, 1);
        assert!(!t.write_plan(None));
        assert!(t.write_plan(st("minecraft:torch", 4)));
        assert_eq!(t.at(PLAN_SLOT).unwrap().item_id, FULL_PLAN);
        let plan = t.plan.clone().unwrap();
        assert_eq!(plan.requires[0].as_ref().unwrap().count, 1);
        assert_eq!(Plan::decode(&plan.encode()), Some(plan));
        let mut nbt = Default::default();
        t.save(&mut nbt);
        let mut u = ProjectTable::from_container_items(&t.to_container_items(), None);
        u.load_meta(&nbt);
        assert_eq!(u, ProjectTable { rotation: 0, ..t });
    }

    #[test]
    fn plan_satisfied_from_storage() {
        let mut t = ProjectTable::new();
        t.inv.slots[PLAN_SLOT] = st(FULL_PLAN, 1);
        let mut req: [Option<ItemStack>; 9] = Default::default();
        req[4] = st("minecraft:stick", 1);
        t.plan = Some(Plan {
            requires: req,
            result: st("minecraft:torch", 4),
        });
        assert_ne!(t.satisfy_mask(), FULL_MASK);
        t.inv.slots[STORAGE_START + 3] = st("minecraft:stick", 2);
        assert_eq!(t.satisfy_mask(), FULL_MASK);
        let grid = t.crafting_grid();
        assert_eq!(grid[4].as_ref().unwrap().item_id, "minecraft:stick");
        let r = t.craft_result(|g| g[4].as_ref().map(|_| ItemStack::new("minecraft:torch", 4)));
        assert_eq!(r.unwrap().count, 4);
        t.on_craft_taken();
        assert_eq!(t.at(STORAGE_START + 3).unwrap().count, 1);
    }

    #[test]
    fn craft_refills_grid_from_storage() {
        let mut t = ProjectTable::new();
        t.inv.slots[0] = st("minecraft:stick", 1);
        t.inv.slots[STORAGE_START] = st("minecraft:stick", 3);
        assert!(!t.is_last_use());
        t.on_craft_taken();
        assert_eq!(t.at(0).unwrap().count, 1);
        assert_eq!(t.at(STORAGE_START).unwrap().count, 2);
        t.inv.slots[STORAGE_START] = None;
        assert!(t.is_last_use());
    }

    #[test]
    fn removal_skips_slot_27() {
        let mut t = ProjectTable::new();
        t.inv.slots[26] = st("minecraft:dirt", 1);
        t.inv.slots[27] = st("minecraft:dirt", 1);
        assert_eq!(t.removal_drops().len(), 1);
    }
}
