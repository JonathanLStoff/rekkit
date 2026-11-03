//! Deployer (`redpower_2:deployer`): port of `TileDeploy` and `TileDeployBase`.
//!
//! Pure logic. On a redstone rising edge the deployer tries each of its nine slots in order,
//! right-clicking the held stack on the block in front with a fake player; the first slot
//! whose use succeeds ends the pulse. The engine layer supplies that use as a callback.

use super::base::MachineFlags;
use crate::ItemStack;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};

/// Number of inventory slots (`k_()`).
pub const DEPLOYER_SLOTS: usize = 9;
/// Per-slot stack limit (`c()`).
pub const DEPLOYER_STACK_LIMIT: u32 = 64;
/// Ticks the active flag stays up after an unpowered neighbour update.
pub const DEPLOY_RESET_DELAY: u32 = 5;
/// Distance the fake player's look vector is scaled by when hitting entities.
pub const ENTITY_REACH: f64 = 2.5;

/// `TileDeploy`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileDeployer {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub contents: [Option<ItemStack>; DEPLOYER_SLOTS],
    pub scheduled_tick: Option<u32>,
}

impl TileDeployer {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            contents: Default::default(),
            scheduled_tick: None,
        }
    }

    /// The side the deployer acts on (`Rotation ^ 1`).
    pub fn front(&self) -> Direction {
        self.rotation.opposite()
    }

    /// `getSizeInventorySide`: nothing is exposed on the front face, nine slots elsewhere.
    pub fn inventory_size_for_side(&self, side: Direction) -> usize {
        if side.opposite() == self.rotation {
            0
        } else {
            DEPLOYER_SLOTS
        }
    }

    /// `a(int, int)`: take up to `n` items out of a slot.
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

    /// `a(int, ItemStack)`: set a slot, clamping the stack to the limit.
    pub fn set(&mut self, slot: usize, stack: Option<ItemStack>) {
        if let Some(cell) = self.contents.get_mut(slot) {
            *cell = stack.map(|mut s| {
                s.count = s.count.min(DEPLOYER_STACK_LIMIT);
                s
            });
        }
    }

    /// `onBlockRemoval`: the contents that drop into the world.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        self.contents
            .iter_mut()
            .filter_map(|c| c.take())
            .filter(|s| s.count > 0)
            .collect()
    }

    /// `enableTowards`: try each non-empty slot in order; `use_item(slot, stack)` performs the
    /// fake-player use (and may change the stack) and returns whether it did something.
    /// Returns true if a slot was used.
    pub fn enable_towards(
        &mut self,
        use_item: &mut dyn FnMut(usize, &mut ItemStack) -> bool,
    ) -> bool {
        for i in 0..DEPLOYER_SLOTS {
            let Some(stack) = self.contents[i].as_mut() else {
                continue;
            };
            if stack.count == 0 {
                continue;
            }
            if use_item(i, stack) {
                if self.contents[i].as_ref().is_some_and(|s| s.count == 0) {
                    self.contents[i] = None;
                }
                return true;
            }
        }
        false
    }

    /// `onBlockNeighborChange`. Returns true when the engine must update the block (active
    /// flag raised) after `use_item` ran.
    pub fn on_neighbor_change(
        &mut self,
        powered: bool,
        use_item: &mut dyn FnMut(usize, &mut ItemStack) -> bool,
    ) -> bool {
        if !powered {
            if self.flags.active {
                self.scheduled_tick = Some(DEPLOY_RESET_DELAY);
            }
            false
        } else if !self.flags.active {
            self.flags.active = true;
            self.enable_towards(use_item);
            true
        } else {
            false
        }
    }

    /// `onTileTick`: the pulse ends.
    pub fn on_tile_tick(&mut self) {
        self.scheduled_tick = None;
        self.flags.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: &str, n: u32) -> ItemStack {
        ItemStack::new(id, n)
    }

    #[test]
    fn first_usable_slot_only() {
        let mut d = TileDeployer::new(Direction::North);
        d.set(1, Some(stack("a", 2)));
        d.set(3, Some(stack("b", 2)));
        let mut tried = Vec::new();
        let used = d.on_neighbor_change(true, &mut |slot, s| {
            tried.push(slot);
            s.count -= 1;
            true
        });
        assert!(used && d.flags.active);
        assert_eq!(tried, vec![1]);
        assert_eq!(d.contents[1].as_ref().unwrap().count, 1);
        assert_eq!(d.contents[3].as_ref().unwrap().count, 2);
    }

    #[test]
    fn skips_failed_slots_and_empties_used_up() {
        let mut d = TileDeployer::new(Direction::North);
        d.set(0, Some(stack("a", 1)));
        d.set(2, Some(stack("b", 1)));
        d.enable_towards(&mut |slot, s| {
            if slot == 2 {
                s.count = 0;
                true
            } else {
                false
            }
        });
        assert!(d.contents[0].is_some() && d.contents[2].is_none());
    }

    #[test]
    fn pulse_resets_after_unpower() {
        let mut d = TileDeployer::new(Direction::East);
        d.on_neighbor_change(true, &mut |_, _| false);
        assert!(d.flags.active);
        assert!(!d.on_neighbor_change(true, &mut |_, _| panic!("no retrigger")));
        d.on_neighbor_change(false, &mut |_, _| false);
        assert_eq!(d.scheduled_tick, Some(5));
        d.on_tile_tick();
        assert!(!d.flags.active && d.scheduled_tick.is_none());
    }

    #[test]
    fn inventory_and_sides() {
        let mut d = TileDeployer::new(Direction::North);
        assert_eq!(d.inventory_size_for_side(Direction::South), 0);
        assert_eq!(d.inventory_size_for_side(Direction::North), 9);
        d.set(0, Some(stack("a", 200)));
        assert_eq!(d.contents[0].as_ref().unwrap().count, 64);
        assert_eq!(d.take(0, 10).unwrap().count, 10);
        assert_eq!(d.take(0, 99).unwrap().count, 54);
        assert!(d.take(0, 1).is_none());
    }
}
