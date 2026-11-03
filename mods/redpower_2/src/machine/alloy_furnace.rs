//! Alloy Furnace (`redpower_2:alloy_furnace`): port of `TileAlloyFurnace`, `ContainerAlloyFurnace`
//! and `SlotAlloyFurnace`.
//!
//! Pure logic. Slots 0..9 are the 3x3 input matrix, slot 9 the fuel, slot 10 the output.
//! Recipes come from `CraftLib::get_alloy_result`; a smelt takes 200 ticks of burning fuel.

use crate::core::foundation::CraftLib;
use crate::ItemStack;
use serde::{Deserialize, Serialize};

pub const FURNACE_SLOTS: usize = 11;
pub const INPUT_COUNT: usize = 9;
pub const FUEL_SLOT: usize = 9;
pub const OUTPUT_SLOT: usize = 10;
/// Ticks one smelt takes (`cooktime == 200`).
pub const COOK_TICKS: u32 = 200;
/// Per-slot stack limit (`c()`).
pub const STACK_LIMIT: u32 = 64;
/// Screen size of `GuiAlloyFurnace` (`afurnacegui.png`).
pub const GUI_SIZE: [f32; 2] = [176.0, 166.0];
/// Player inventory origin of `ContainerAlloyFurnace`.
pub const GUI_INVENTORY: [f32; 2] = [8.0, 84.0];

/// Slot positions of `ContainerAlloyFurnace`: 3x3 at (48,17), fuel (17,42), output (141,35).
pub fn gui_slots() -> Vec<[f32; 2]> {
    let mut slots = Vec::with_capacity(FURNACE_SLOTS);
    for i in 0..3 {
        for j in 0..3 {
            slots.push([48.0 + j as f32 * 18.0, 17.0 + i as f32 * 18.0]);
        }
    }
    slots.push([17.0, 42.0]);
    slots.push([141.0, 35.0]);
    slots
}

/// `CoreLib.getBurnTime` (vanilla `TileEntityFurnace.getItemBurnTime`).
pub fn burn_time(stack: &ItemStack) -> u32 {
    let id = stack
        .item_id
        .strip_prefix("minecraft:")
        .unwrap_or(&stack.item_id);
    match id {
        "coal" | "charcoal" => 1600,
        "coal_block" => 16000,
        "lava_bucket" => 20000,
        "blaze_rod" => 2400,
        "stick" | "sapling" | "oak_sapling" | "spruce_sapling" | "birch_sapling"
        | "jungle_sapling" => 100,
        "wooden_sword" | "wooden_pickaxe" | "wooden_axe" | "wooden_shovel" | "wooden_hoe" => 200,
        "bowl" | "wooden_door" | "oak_door" | "sign" | "oak_sign" => 200,
        _ if id.ends_with("_planks")
            || id.ends_with("_log")
            || id.ends_with("_wood")
            || id == "planks"
            || id == "log"
            || id == "crafting_table"
            || id == "chest"
            || id == "bookshelf"
            || id == "jukebox"
            || id == "note_block" =>
        {
            300
        }
        _ if id.ends_with("_slab") && id.starts_with("oak") => 150,
        _ => 0,
    }
}

/// Item left behind when a fuel is consumed (`getContainerItem`).
fn fuel_container(stack: &ItemStack) -> Option<ItemStack> {
    (stack.item_id == "minecraft:lava_bucket").then(|| ItemStack::new("minecraft:bucket", 1))
}

/// `CoreLib.rotToSide`: side (ForgeDirection ordinal) a rotation 0..3 faces.
pub fn rot_to_side(r: i32) -> i32 {
    match r {
        0 => 5,
        1 => 3,
        2 => 4,
        _ => 2,
    }
}

/// `TileAlloyFurnace`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileAlloyFurnace {
    /// `Rotation` 0..3.
    pub rotation: i32,
    pub contents: [Option<ItemStack>; FURNACE_SLOTS],
    pub total_burn: u32,
    pub burn_time: u32,
    pub cook_time: u32,
    pub active: bool,
}

impl Default for TileAlloyFurnace {
    fn default() -> Self {
        Self::new(0)
    }
}

/// What the engine must do after a tick.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TickResult {
    /// Lit state changed: update the block and its light.
    pub lit_changed: bool,
    /// Contents changed: mark the entity dirty.
    pub dirty: bool,
}

impl TileAlloyFurnace {
    pub fn new(rotation: i32) -> Self {
        Self {
            rotation: rotation & 3,
            contents: Default::default(),
            total_burn: 0,
            burn_time: 0,
            cook_time: 0,
            active: false,
        }
    }

    /// `onBlockPlaced`: rotation from the placer's yaw in degrees.
    pub fn rotation_from_yaw(yaw: f32) -> i32 {
        ((yaw * 4.0 / 360.0 + 0.5).floor() as i32) & 3
    }

    /// `canSmelt`.
    pub fn can_smelt(&self, lib: &CraftLib) -> bool {
        let mut grid: Vec<Option<ItemStack>> = self.contents[..INPUT_COUNT].to_vec();
        let Some(res) = lib.get_alloy_result(&mut grid, 0, INPUT_COUNT, false) else {
            return false;
        };
        match &self.contents[OUTPUT_SLOT] {
            None => true,
            Some(out) => {
                if out.item_id != res.item_id || out.damage != res.damage {
                    return false;
                }
                let total = out.count + res.count;
                total <= STACK_LIMIT && total <= res.max_stack_size
            }
        }
    }

    /// `smeltItem`.
    pub fn smelt_item(&mut self, lib: &CraftLib) {
        if !self.can_smelt(lib) {
            return;
        }
        let Some(res) = lib.get_alloy_result(&mut self.contents, 0, INPUT_COUNT, true) else {
            return;
        };
        match &mut self.contents[OUTPUT_SLOT] {
            Some(out) => out.count += res.count,
            slot @ None => *slot = Some(res),
        }
    }

    /// `n_` / `g`: one server tick.
    pub fn tick(&mut self, lib: &CraftLib) -> TickResult {
        let mut r = TickResult::default();
        let mut burned_out = false;
        if self.burn_time > 0 {
            self.burn_time -= 1;
            if self.burn_time == 0 {
                burned_out = true;
                self.active = false;
            }
        }
        let can = self.can_smelt(lib);
        if self.burn_time == 0 && can {
            if let Some(fuel) = self.contents[FUEL_SLOT].as_ref() {
                let t = burn_time(fuel);
                if t > 0 {
                    self.burn_time = t;
                    self.total_burn = t;
                    self.active = true;
                    let left = fuel_container(fuel);
                    if let Some(left) = left {
                        self.contents[FUEL_SLOT] = Some(left);
                    } else {
                        let f = self.contents[FUEL_SLOT].as_mut().unwrap();
                        f.count -= 1;
                        if f.count == 0 {
                            self.contents[FUEL_SLOT] = None;
                        }
                    }
                    r.dirty = true;
                    if !burned_out {
                        r.lit_changed = true;
                    }
                }
            }
        }
        if self.burn_time > 0 && can {
            self.cook_time += 1;
            if self.cook_time == COOK_TICKS {
                self.cook_time = 0;
                self.smelt_item(lib);
                r.dirty = true;
            }
        } else {
            self.cook_time = 0;
        }
        if burned_out {
            r.lit_changed = true;
        }
        r
    }

    /// `getCookScaled`.
    pub fn cook_scaled(&self, n: u32) -> u32 {
        self.cook_time * n / COOK_TICKS
    }

    /// `getBurnScaled`.
    pub fn burn_scaled(&self, n: u32) -> u32 {
        (self.burn_time * n)
            .checked_div(self.total_burn)
            .unwrap_or(0)
    }

    /// `getStartInventorySide(side)`; `side` is the ForgeDirection ordinal (0 down, 1 up, 2..5).
    pub fn start_for_side(&self, side: i32) -> usize {
        if side == 1 {
            return 0;
        }
        let s = rot_to_side(self.rotation);
        if side == s {
            FUEL_SLOT
        } else if side == (s ^ 1) {
            OUTPUT_SLOT
        } else {
            0
        }
    }

    /// `getSizeInventorySide(side)`.
    pub fn size_for_side(&self, side: i32) -> usize {
        if side == 1 {
            return INPUT_COUNT;
        }
        let s = rot_to_side(self.rotation);
        usize::from(side == s || side == (s ^ 1))
    }

    /// Slots reachable from a side.
    pub fn slots_for_side(&self, side: i32) -> Vec<usize> {
        let start = self.start_for_side(side);
        (start..start + self.size_for_side(side)).collect()
    }

    /// `a(int, int)`: take up to `n` items from a slot.
    pub fn take(&mut self, slot: usize, n: u32) -> Option<ItemStack> {
        let stack = self.contents.get_mut(slot)?.as_mut()?;
        if stack.count <= n {
            return self.contents[slot].take();
        }
        stack.count -= n;
        let mut out = stack.clone();
        out.count = n;
        Some(out)
    }

    /// `a(int, ItemStack)`: set a slot, clamping to the stack limit.
    pub fn set(&mut self, slot: usize, stack: Option<ItemStack>) {
        if let Some(cell) = self.contents.get_mut(slot) {
            *cell = stack.map(|mut s| {
                s.count = s.count.min(STACK_LIMIT);
                s
            });
        }
    }

    /// `onBlockRemoval`: everything drops.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        self.contents
            .iter_mut()
            .filter_map(|c| c.take())
            .filter(|s| s.count > 0)
            .collect()
    }

    /// `SlotAlloyFurnace.a(ist)`: the output slot accepts nothing from the player.
    pub fn slot_accepts(slot: usize, stack: &ItemStack) -> bool {
        if slot == OUTPUT_SLOT {
            return false;
        }
        let _ = stack;
        true
    }

    /// Menu data slots synchronised to the open screen (`ContainerAlloyFurnace`):
    /// cook time, burn time, total burn.
    pub fn menu_data(&self) -> Vec<i32> {
        vec![
            self.cook_time as i32,
            self.burn_time as i32,
            self.total_burn as i32,
        ]
    }

    /// Load from block entity parts (`items`, `data`, `facing` side 2..=5).
    pub fn from_parts(items: &[Option<ItemStack>], data: &[i32], facing_side: i32) -> Self {
        let rotation = (0..4).find(|&r| rot_to_side(r) == facing_side).unwrap_or(0);
        let mut t = Self::new(rotation);
        for (i, s) in items.iter().take(FURNACE_SLOTS).enumerate() {
            t.contents[i] = s.clone();
        }
        let g = |i: usize| data.get(i).copied().unwrap_or(0).max(0) as u32;
        t.cook_time = g(0);
        t.burn_time = g(1);
        t.total_burn = g(2);
        t.active = t.burn_time > 0;
        t
    }

    /// Write the contents back into a block entity's item list.
    pub fn write_items(&self, items: &mut Vec<Option<ItemStack>>) {
        items.resize(FURNACE_SLOTS.max(items.len()), None);
        for (i, s) in self.contents.iter().enumerate() {
            items[i] = s.clone();
        }
    }
}

/// `ContainerAlloyFurnace.b(player, slot)`: shift-click destination range
/// (inventory 11..47 reversed from the furnace; furnace inputs 0..9 from the inventory).
pub fn quick_move_range(slot: usize) -> (usize, usize, bool) {
    if slot < FURNACE_SLOTS {
        (11, 47, true)
    } else {
        (0, INPUT_COUNT, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib() -> CraftLib {
        let mut c = CraftLib::default();
        c.register_default_alloy_recipes();
        c
    }

    fn st(id: &str, n: u32) -> Option<ItemStack> {
        Some(ItemStack::new(id, n))
    }

    #[test]
    fn gui_layout_matches_container() {
        let s = gui_slots();
        assert_eq!(s.len(), 11);
        assert_eq!(s[0], [48.0, 17.0]);
        assert_eq!(s[8], [84.0, 53.0]);
        assert_eq!(s[9], [17.0, 42.0]);
        assert_eq!(s[10], [141.0, 35.0]);
    }

    #[test]
    fn smelts_red_alloy_in_200_ticks() {
        let l = lib();
        let mut t = TileAlloyFurnace::new(0);
        t.contents[0] = st("minecraft:redstone", 4);
        t.contents[1] = st("minecraft:iron_ingot", 1);
        t.contents[FUEL_SLOT] = st("minecraft:coal", 2);
        for _ in 0..199 {
            t.tick(&l);
        }
        assert!(t.contents[OUTPUT_SLOT].is_none());
        assert!(t.active);
        t.tick(&l);
        let out = t.contents[OUTPUT_SLOT].as_ref().unwrap();
        assert_eq!(out.item_id, "redpower_2:red_alloy_ingot");
        assert!(t.contents[0].is_none() && t.contents[1].is_none());
        assert_eq!(t.contents[FUEL_SLOT].as_ref().unwrap().count, 1);
    }

    #[test]
    fn no_recipe_no_fuel_use() {
        let l = lib();
        let mut t = TileAlloyFurnace::new(0);
        t.contents[FUEL_SLOT] = st("minecraft:coal", 1);
        t.tick(&l);
        assert_eq!(t.contents[FUEL_SLOT].as_ref().unwrap().count, 1);
        assert_eq!(t.burn_time, 0);
    }

    #[test]
    fn lava_bucket_leaves_bucket() {
        let l = lib();
        let mut t = TileAlloyFurnace::new(0);
        t.contents[0] = st("minecraft:redstone", 4);
        t.contents[1] = st("minecraft:iron_ingot", 1);
        t.contents[FUEL_SLOT] = st("minecraft:lava_bucket", 1);
        t.tick(&l);
        assert_eq!(
            t.contents[FUEL_SLOT].as_ref().unwrap().item_id,
            "minecraft:bucket"
        );
        assert_eq!(t.total_burn, 20000);
    }

    #[test]
    fn output_blocked_by_other_item_or_full() {
        let l = lib();
        let mut t = TileAlloyFurnace::new(0);
        t.contents[0] = st("minecraft:redstone", 4);
        t.contents[1] = st("minecraft:iron_ingot", 1);
        t.contents[OUTPUT_SLOT] = st("minecraft:stick", 1);
        assert!(!t.can_smelt(&l));
        t.contents[OUTPUT_SLOT] = st("redpower_2:red_alloy_ingot", 64);
        assert!(!t.can_smelt(&l));
        t.contents[OUTPUT_SLOT] = st("redpower_2:red_alloy_ingot", 10);
        assert!(t.can_smelt(&l));
    }

    #[test]
    fn sided_access() {
        // rotation 0 faces side 5 (east)
        let t = TileAlloyFurnace::new(0);
        assert_eq!(t.slots_for_side(1), (0..9).collect::<Vec<_>>());
        assert_eq!(t.slots_for_side(5), vec![9]);
        assert_eq!(t.slots_for_side(4), vec![10]);
        assert!(t.slots_for_side(2).is_empty());
        assert!(t.slots_for_side(0).is_empty());
    }

    #[test]
    fn output_slot_rejects_insertion_and_scaling() {
        assert!(!TileAlloyFurnace::slot_accepts(
            OUTPUT_SLOT,
            &ItemStack::new("a", 1)
        ));
        assert!(TileAlloyFurnace::slot_accepts(0, &ItemStack::new("a", 1)));
        let mut t = TileAlloyFurnace::new(0);
        t.total_burn = 1600;
        t.burn_time = 800;
        t.cook_time = 100;
        assert_eq!(t.burn_scaled(12), 6);
        assert_eq!(t.cook_scaled(24), 12);
    }

    #[test]
    fn rotation_and_roundtrip() {
        assert_eq!(TileAlloyFurnace::rotation_from_yaw(0.0), 0);
        assert_eq!(TileAlloyFurnace::rotation_from_yaw(90.0), 1);
        assert_eq!(TileAlloyFurnace::rotation_from_yaw(-90.0), 3);
        let mut t = TileAlloyFurnace::new(2);
        t.contents[3] = st("x", 5);
        t.cook_time = 7;
        let mut items = Vec::new();
        t.write_items(&mut items);
        let u = TileAlloyFurnace::from_parts(&items, &t.menu_data(), rot_to_side(2));
        assert_eq!(u.rotation, 2);
        assert_eq!(u.contents[3], t.contents[3]);
        assert_eq!(u.cook_time, 7);
        assert_eq!(quick_move_range(3), (11, 47, true));
        assert_eq!(quick_move_range(20), (0, 9, false));
    }
}
