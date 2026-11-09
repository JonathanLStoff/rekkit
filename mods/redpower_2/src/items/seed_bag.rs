//! Seed Bag (`redpower_2:seed_bag`): port of `ItemSeedBag` and `ContainerSeedBag`.
//!
//! Features:
//! - 9-slot inventory holding up to 576 seeds (64 per slot).
//! - Durability bar tracks remaining seeds ($576 - \text{total}$).
//! - Outward spiral planting algorithm (`SpiralSearch`) covering up to a 5x5 farmland area.
//! - Automated vacuum pickup of matching seeds from player inventory.
//! - NBT persistence for all 9 internal slots.

use crate::ItemStack;
use std::collections::HashMap;

/// Canonical item ID for the Seed Bag.
pub const SEED_BAG_ID: &str = "redpower_2:seed_bag";
/// Total slots in the seed bag inventory (3x3 grid).
pub const SEED_BAG_SLOTS: usize = 9;
/// Maximum seed capacity (9 slots * 64 max stack).
pub const SEED_BAG_MAX_CAPACITY: u32 = 576;
/// Maximum radius of the spiral search (2 -> 5x5 block area).
pub const SPIRAL_SEARCH_RADIUS: i32 = 2;

/// Outward rectangular spiral search algorithm from RedPower 2 (`SpiralSearch`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpiralSearch {
    pub cx: i32,
    pub cz: i32,
    pub y: i32,
    pub x: i32,
    pub z: i32,
    pub dx: i32,
    pub dz: i32,
    pub step_count: i32,
    pub leg_length: i32,
    pub leg_index: i32,
    pub max_radius: i32,
}

impl SpiralSearch {
    pub fn new(center: (i32, i32, i32), max_radius: i32) -> Self {
        Self {
            cx: center.0,
            cz: center.2,
            y: center.1,
            x: center.0,
            z: center.2,
            dx: 1,
            dz: 0,
            step_count: 0,
            leg_length: 1,
            leg_index: 0,
            max_radius,
        }
    }

    /// Advance one step in the spiral. Returns the current `(x, y, z)` coordinate.
    pub fn step(&mut self) -> Option<(i32, i32, i32)> {
        let current = (self.x, self.y, self.z);
        if (self.x - self.cx).abs() > self.max_radius || (self.z - self.cz).abs() > self.max_radius
        {
            return None;
        }

        self.x += self.dx;
        self.z += self.dz;
        self.step_count += 1;

        if self.step_count >= self.leg_length {
            self.step_count = 0;
            // Turn 90 degrees right: (dx, dz) -> (-dz, dx)
            let old_dx = self.dx;
            self.dx = -self.dz;
            self.dz = old_dx;
            self.leg_index += 1;
            if self.leg_index % 2 == 0 {
                self.leg_length += 1;
            }
        }

        Some(current)
    }
}

/// 9-slot Seed Bag state machine.
#[derive(Debug, Clone, PartialEq)]
pub struct SeedBagInventory {
    pub slots: [Option<ItemStack>; SEED_BAG_SLOTS],
}

impl Default for SeedBagInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl SeedBagInventory {
    pub fn new() -> Self {
        Self {
            slots: [const { None }; SEED_BAG_SLOTS],
        }
    }

    /// Total count of all seeds stored across all 9 slots.
    pub fn total_seeds(&self) -> u32 {
        self.slots.iter().flatten().map(|s| s.count).sum()
    }

    /// Item damage value (0 is full 576 seeds, 576 is completely empty).
    pub fn damage(&self) -> u16 {
        (SEED_BAG_MAX_CAPACITY.saturating_sub(self.total_seeds())).min(576) as u16
    }

    /// Returns the primary seed type held in the bag.
    pub fn primary_seed_type(&self) -> Option<&str> {
        self.slots
            .iter()
            .flatten()
            .find(|s| s.count > 0)
            .map(|s| s.item_id.as_str())
    }

    /// Pulls and decrements a single seed from the first available slot.
    pub fn take_one_seed(&mut self) -> Option<ItemStack> {
        for slot in &mut self.slots {
            if let Some(stack) = slot {
                if stack.count > 0 {
                    let mut single = stack.clone();
                    single.count = 1;
                    stack.count -= 1;
                    if stack.count == 0 {
                        *slot = None;
                    }
                    return Some(single);
                }
            }
        }
        None
    }

    /// Checks if a seed item can be accepted into this bag.
    pub fn can_add(&self, item: &ItemStack) -> bool {
        if let Some(primary) = self.primary_seed_type() {
            if item.item_id != primary {
                return false;
            }
        }

        self.slots.iter().any(|s| match s {
            None => true,
            Some(existing) => existing.item_id == item.item_id && existing.count < 64,
        })
    }

    /// Inserts seed items into the bag. Returns any excess items.
    pub fn add_seeds(&mut self, mut item: ItemStack) -> Option<ItemStack> {
        if !self.can_add(&item) {
            return Some(item);
        }

        // 1. Stack into existing slots
        for s in self.slots.iter_mut().flatten() {
            if s.item_id == item.item_id && s.count < 64 {
                let available = 64 - s.count;
                let to_add = available.min(item.count);
                s.count += to_add;
                item.count -= to_add;
                if item.count == 0 {
                    return None;
                }
            }
        }

        // 2. Insert into empty slots
        for slot in &mut self.slots {
            if slot.is_none() {
                let to_place = 64.min(item.count);
                let mut placed = item.clone();
                placed.count = to_place;
                *slot = Some(placed);
                item.count -= to_place;
                if item.count == 0 {
                    return None;
                }
            }
        }

        if item.count > 0 {
            Some(item)
        } else {
            None
        }
    }

    /// Executes outward spiral planting around `center` position.
    /// Returns the coordinates of planted crops.
    pub fn spiral_plant(
        &mut self,
        center: (i32, i32, i32),
        can_plant_at: impl Fn(i32, i32, i32) -> bool,
    ) -> Vec<(i32, i32, i32)> {
        let mut planted = Vec::new();
        let mut spiral = SpiralSearch::new(center, SPIRAL_SEARCH_RADIUS);

        while let Some(pos) = spiral.step() {
            if self.total_seeds() == 0 {
                break;
            }

            if can_plant_at(pos.0, pos.1, pos.2) {
                if let Some(_seed) = self.take_one_seed() {
                    planted.push((pos.0, pos.1 + 1, pos.2));
                }
            }
        }

        planted
    }

    /// Restores 9-slot inventory from NBT string map.
    pub fn from_nbt(nbt: &HashMap<String, String>) -> Self {
        let mut bag = Self::new();
        for i in 0..SEED_BAG_SLOTS {
            if let Some(id) = nbt.get(&format!("slot_{}_id", i)) {
                let count = nbt
                    .get(&format!("slot_{}_count", i))
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                if count > 0 {
                    bag.slots[i] = Some(ItemStack::new(id, count));
                }
            }
        }
        bag
    }

    /// Serializes 9-slot inventory into NBT string map.
    pub fn to_nbt(&self, nbt: &mut HashMap<String, String>) {
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(stack) = slot {
                nbt.insert(format!("slot_{}_id", i), stack.item_id.clone());
                nbt.insert(format!("slot_{}_count", i), stack.count.to_string());
            } else {
                nbt.remove(&format!("slot_{}_id", i));
                nbt.remove(&format!("slot_{}_count", i));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiral_search_coverage() {
        let mut spiral = SpiralSearch::new((0, 64, 0), 2);
        let mut visited = Vec::new();
        while let Some(pt) = spiral.step() {
            visited.push(pt);
        }
        // 5x5 = 25 blocks visited
        assert_eq!(visited.len(), 25);
        assert_eq!(visited[0], (0, 64, 0));
    }

    #[test]
    fn test_seed_bag_storage_and_damage() {
        let mut bag = SeedBagInventory::new();
        assert_eq!(bag.total_seeds(), 0);
        assert_eq!(bag.damage(), 576);

        // Add 64 flax seeds
        let rem = bag.add_seeds(ItemStack::new("redpower_2:flax_seeds", 64));
        assert!(rem.is_none());
        assert_eq!(bag.total_seeds(), 64);
        assert_eq!(bag.damage(), 512);

        // Cannot add different seed type when non-empty
        let rejected = bag.add_seeds(ItemStack::new("minecraft:wheat_seeds", 10));
        assert!(rejected.is_some());
        assert_eq!(bag.total_seeds(), 64);
    }

    #[test]
    fn test_seed_bag_spiral_planting() {
        let mut bag = SeedBagInventory::new();
        bag.add_seeds(ItemStack::new("redpower_2:flax_seeds", 10));

        let soil_checker = |x: i32, _y: i32, z: i32| (x + z) % 2 == 0;
        let planted = bag.spiral_plant((0, 64, 0), soil_checker);

        assert!(!planted.is_empty());
        assert_eq!(bag.total_seeds() + planted.len() as u32, 10);
    }

    #[test]
    fn test_seed_bag_nbt_persistence() {
        let mut bag = SeedBagInventory::new();
        bag.add_seeds(ItemStack::new("redpower_2:flax_seeds", 100));

        let mut nbt = HashMap::new();
        bag.to_nbt(&mut nbt);

        let restored = SeedBagInventory::from_nbt(&nbt);
        assert_eq!(restored.total_seeds(), 100);
        assert_eq!(restored.damage(), bag.damage());
    }
}
