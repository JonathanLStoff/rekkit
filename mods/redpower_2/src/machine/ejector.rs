//! Ejector (`redpower_2:ejector`): port of `TileEject` and `TileEjectBase`.
//!
//! Pure logic. On a redstone rising edge the ejector takes one item from its first non-empty
//! slot into a tube buffer and pushes it out of its rear face; a refused item plugs the buffer
//! and is retried on a tile tick. The engine layer supplies `handle_item` (offering the item
//! to the tube or inventory behind).

use super::base::MachineFlags;
use super::breaker::{BufferedItem, TubeBuffer, BLOCKED_RETRY_DELAY, TUBE_STATE_BOUNCE};
use crate::ItemStack;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};

/// Number of inventory slots (`k_()`).
pub const EJECTOR_SLOTS: usize = 9;
/// Per-slot stack limit (`c()`).
pub const EJECTOR_STACK_LIMIT: u32 = 64;
/// Ticks before the next drain/reset tick.
pub const EJECT_TICK_DELAY: u32 = 5;
/// `ContainerEject`: 3x3 grid origin and the player inventory origin; screen 176x166.
pub const EJECTOR_GRID_ORIGIN: [f32; 2] = [62.0, 17.0];
pub const EJECTOR_INVENTORY_ORIGIN: [f32; 2] = [8.0, 84.0];

/// Slot positions of `ContainerEject` (index = `j + i * 3`).
pub fn slot_positions() -> Vec<[f32; 2]> {
    let mut v = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            v.push([
                EJECTOR_GRID_ORIGIN[0] + j as f32 * 18.0,
                EJECTOR_GRID_ORIGIN[1] + i as f32 * 18.0,
            ]);
        }
    }
    v
}

/// `TileEject`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileEjector {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub buffer: TubeBuffer,
    pub contents: [Option<ItemStack>; EJECTOR_SLOTS],
    pub scheduled_tick: Option<u32>,
}

impl TileEjector {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            buffer: TubeBuffer::default(),
            contents: Default::default(),
            scheduled_tick: None,
        }
    }

    /// Tubes connect on the rear face (`getTubeConnectableSides`).
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation
    }

    /// `tubeItemCanEnter`.
    pub fn tube_item_can_enter(&self, side: Direction, state: i32) -> bool {
        side == self.rotation && state == TUBE_STATE_BOUNCE
    }

    /// `tubeWeight`.
    pub fn tube_weight(&self, side: Direction, state: i32) -> usize {
        if self.tube_item_can_enter(side, state) {
            self.buffer.size()
        } else {
            0
        }
    }

    /// `tubeItemEnter`: a bounced item re-enters through the rear.
    pub fn tube_item_enter(&mut self, side: Direction, state: i32, item: BufferedItem) -> bool {
        if self.tube_item_can_enter(side, state) {
            self.buffer.add_bounce(item);
            self.flags.active = true;
            self.scheduled_tick = Some(EJECT_TICK_DELAY);
            true
        } else {
            false
        }
    }

    /// `getSizeInventorySide`: nothing on the rear face, nine slots elsewhere.
    pub fn inventory_size_for_side(&self, side: Direction) -> usize {
        if side == self.rotation {
            0
        } else {
            EJECTOR_SLOTS
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
                s.count = s.count.min(EJECTOR_STACK_LIMIT);
                s
            });
        }
    }

    /// `onBlockRemoval`: slot contents and the buffer drop into the world.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        let mut out: Vec<ItemStack> = self
            .contents
            .iter_mut()
            .filter_map(|c| c.take())
            .filter(|s| s.count > 0)
            .collect();
        out.extend(self.buffer.take_all_for_drop());
        out
    }

    /// `drainBuffer`: eject from the back; a refusal plugs the buffer.
    pub fn drain_buffer(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) {
        while let Some(last) = self.buffer.last() {
            if !handle_item(last) {
                self.buffer.plugged = true;
                return;
            }
            self.buffer.pop();
            if self.buffer.plugged {
                return;
            }
        }
    }

    /// `handleExtract`: one item from the first non-empty slot into the buffer, then drain.
    pub fn handle_extract(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        for n in 0..EJECTOR_SLOTS {
            if self.contents[n].as_ref().is_some_and(|s| s.count > 0) {
                if let Some(one) = self.take(n, 1) {
                    self.buffer.add_new(one);
                }
                self.drain_buffer(handle_item);
                return true;
            }
        }
        false
    }

    /// `onBlockNeighborChange`. Returns true when the engine must update the block.
    pub fn on_neighbor_change(
        &mut self,
        powered: bool,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        if powered {
            if !self.flags.powered {
                self.flags.powered = true;
                if !self.flags.active {
                    self.flags.active = true;
                    return self.handle_extract(handle_item);
                }
            }
        } else {
            if self.flags.active && self.scheduled_tick.is_none() {
                self.scheduled_tick = Some(EJECT_TICK_DELAY);
            }
            self.flags.powered = false;
        }
        false
    }

    /// `onTileTick`; returns true when the active flag changed (client update needed).
    pub fn on_tile_tick(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        self.scheduled_tick = None;
        if !self.buffer.is_empty() {
            self.drain_buffer(handle_item);
            self.scheduled_tick = Some(if self.buffer.is_empty() {
                EJECT_TICK_DELAY
            } else {
                BLOCKED_RETRY_DELAY
            });
            false
        } else if !self.flags.powered {
            self.flags.active = false;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: &str, n: u32) -> ItemStack {
        ItemStack::new(id, n)
    }

    #[test]
    fn pulse_ejects_one_item_from_first_slot() {
        let mut e = TileEjector::new(Direction::North);
        e.set(2, Some(stack("a", 3)));
        e.set(4, Some(stack("b", 1)));
        let mut got = Vec::new();
        let upd = e.on_neighbor_change(true, &mut |i| {
            got.push((i.item.item_id.clone(), i.item.count));
            true
        });
        assert!(upd && e.flags.active && e.flags.powered);
        assert_eq!(got, vec![("a".to_string(), 1)]);
        assert_eq!(e.contents[2].as_ref().unwrap().count, 2);
        assert!(!e.on_neighbor_change(true, &mut |_| panic!("no retrigger")));
    }

    #[test]
    fn empty_ejector_does_nothing() {
        let mut e = TileEjector::new(Direction::North);
        assert!(!e.on_neighbor_change(true, &mut |_| true));
        assert!(e.flags.powered);
    }

    #[test]
    fn blocked_output_plugs_and_retries() {
        let mut e = TileEjector::new(Direction::East);
        e.set(0, Some(stack("a", 1)));
        e.on_neighbor_change(true, &mut |_| false);
        assert!(e.buffer.plugged && e.buffer.size() == 1 && e.contents[0].is_none());
        e.on_tile_tick(&mut |_| false);
        assert_eq!(e.scheduled_tick, Some(10));
        e.on_tile_tick(&mut |_| true);
        assert!(e.buffer.is_empty());
        assert_eq!(e.scheduled_tick, Some(5));
    }

    #[test]
    fn unpower_resets_active() {
        let mut e = TileEjector::new(Direction::East);
        e.on_neighbor_change(true, &mut |_| true);
        e.on_neighbor_change(false, &mut |_| true);
        assert_eq!(e.scheduled_tick, Some(5));
        assert!(e.on_tile_tick(&mut |_| true));
        assert!(!e.flags.active);
    }

    #[test]
    fn rear_bounce_and_sides() {
        let mut e = TileEjector::new(Direction::North);
        let it = BufferedItem {
            item: stack("a", 1),
            color: None,
        };
        assert!(!e.tube_item_enter(Direction::South, 2, it.clone()));
        assert!(e.tube_item_enter(Direction::North, 2, it));
        assert_eq!(e.tube_weight(Direction::North, 2), 1);
        assert_eq!(e.tube_weight(Direction::South, 2), 0);
        assert_eq!(e.inventory_size_for_side(Direction::North), 0);
        assert_eq!(e.inventory_size_for_side(Direction::Up), 9);
    }

    #[test]
    fn drops_and_layout() {
        let mut e = TileEjector::new(Direction::North);
        e.set(0, Some(stack("a", 200)));
        assert_eq!(e.contents[0].as_ref().unwrap().count, 64);
        e.buffer.add_new(stack("b", 1));
        assert_eq!(e.take_all_for_drop().len(), 2);
        let p = slot_positions();
        assert_eq!(p.len(), 9);
        assert_eq!(p[8], [98.0, 53.0]);
    }
}
