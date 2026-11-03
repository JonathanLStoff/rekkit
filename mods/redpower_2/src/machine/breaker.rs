//! Block Breaker (`redpower_2:block_breaker`): port of `TileBreaker` and `TubeBuffer`.
//!
//! Pure logic. The engine layer supplies the block in front (its drops, or `None` when it is
//! air, bedrock or has negative hardness) and the `handleItem` ejection into the rear tube/chest.

use super::base::MachineFlags;
use crate::{DyeColor, ItemStack};
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Ticks before the first drain after a break or a bounced item.
pub const BREAK_DRAIN_DELAY: u32 = 5;
/// Ticks before retrying when the buffer could not be emptied.
pub const BLOCKED_RETRY_DELAY: u32 = 10;
/// Tube state in which an item may be pushed back into the breaker (`state == 2`).
pub const TUBE_STATE_BOUNCE: i32 = 2;

/// One buffered item (`TubeItem`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BufferedItem {
    pub item: ItemStack,
    pub color: Option<DyeColor>,
}

/// `TubeBuffer`: `add*` push at the front, `getLast`/`pop` work at the back.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TubeBuffer {
    items: VecDeque<BufferedItem>,
    pub plugged: bool,
}

impl TubeBuffer {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn size(&self) -> usize {
        self.items.len()
    }
    pub fn last(&self) -> Option<&BufferedItem> {
        self.items.back()
    }
    pub fn add(&mut self, item: BufferedItem) {
        self.items.push_front(item);
    }
    pub fn add_new(&mut self, item: ItemStack) {
        self.add(BufferedItem { item, color: None });
    }
    pub fn add_new_color(&mut self, item: ItemStack, color: Option<DyeColor>) {
        self.add(BufferedItem { item, color });
    }
    /// `addAll`: appended at the back (unlike `add`).
    pub fn add_all(&mut self, drops: impl IntoIterator<Item = ItemStack>) {
        for item in drops {
            self.items.push_back(BufferedItem { item, color: None });
        }
    }
    /// `addBounce`: an item returned by a tube goes to the back and plugs the buffer.
    pub fn add_bounce(&mut self, item: BufferedItem) {
        self.items.push_back(item);
        self.plugged = true;
    }
    pub fn pop(&mut self) {
        self.items.pop_back();
        if self.items.is_empty() {
            self.plugged = false;
        }
    }
    /// `onRemove`: everything still buffered drops into the world.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        self.items
            .drain(..)
            .filter(|b| b.item.count > 0)
            .map(|b| b.item)
            .collect()
    }
}

/// What the engine must do after a neighbour change.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BreakOutcome {
    /// Remove the block in front (`setBlock(.., 0)`).
    pub remove_front_block: bool,
    /// Block entity changed and needs saving/syncing.
    pub dirty: bool,
}

/// `TileBreaker`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileBreaker {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub buffer: TubeBuffer,
    /// Ticks until `on_tile_tick`, if scheduled.
    pub scheduled_tick: Option<u32>,
}

impl TileBreaker {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            buffer: TubeBuffer::default(),
            scheduled_tick: None,
        }
    }

    /// The side the breaker harvests (`Rotation ^ 1`).
    pub fn front(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Tubes connect on the rear face (`getTubeConnectableSides`).
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation
    }

    fn schedule(&mut self, ticks: u32) {
        self.scheduled_tick = Some(ticks);
    }

    /// `tubeItemEnter`: only through the rear, in the bounce state.
    pub fn tube_item_enter(&mut self, side: Direction, state: i32, item: BufferedItem) -> bool {
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            self.buffer.add_bounce(item);
            self.flags.active = true;
            self.schedule(BREAK_DRAIN_DELAY);
            true
        } else {
            false
        }
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

    /// `onBlockNeighborChange`. `powered` is the redstone state now; `front_drops` is the
    /// harvest of the block in front (`None` for air, bedrock or hardness < 0).
    pub fn on_neighbor_change(
        &mut self,
        powered: bool,
        front_drops: Option<Vec<ItemStack>>,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> BreakOutcome {
        let mut out = BreakOutcome::default();
        if powered {
            if !self.flags.powered {
                self.flags.powered = true;
                out.dirty = true;
                if !self.flags.active {
                    if let Some(drops) = front_drops {
                        self.flags.active = true;
                        self.buffer.add_all(drops);
                        out.remove_front_block = true;
                        self.drain_buffer(handle_item);
                        if !self.buffer.is_empty() {
                            self.schedule(BREAK_DRAIN_DELAY);
                        }
                    }
                }
            }
        } else {
            if self.flags.active && self.scheduled_tick.is_none() {
                self.schedule(BREAK_DRAIN_DELAY);
            }
            self.flags.powered = false;
        }
        out
    }

    /// `onTileTick`; returns true when the active flag changed (client update needed).
    pub fn on_tile_tick(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        self.scheduled_tick = None;
        if !self.buffer.is_empty() {
            self.drain_buffer(handle_item);
            if !self.buffer.is_empty() {
                self.schedule(BLOCKED_RETRY_DELAY);
            } else {
                self.schedule(BREAK_DRAIN_DELAY);
            }
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
    fn pulse_breaks_and_ejects() {
        let mut b = TileBreaker::new(Direction::North);
        let mut got = Vec::new();
        let out = b.on_neighbor_change(true, Some(vec![stack("a", 1), stack("b", 2)]), &mut |i| {
            got.push(i.item.item_id.clone());
            true
        });
        assert!(out.remove_front_block);
        assert_eq!(got, vec!["b", "a"]); // drains from the back
        assert!(b.buffer.is_empty() && b.flags.active && b.flags.powered);
        // second call while powered does nothing
        let out = b.on_neighbor_change(true, Some(vec![stack("c", 1)]), &mut |_| true);
        assert!(!out.remove_front_block);
    }

    #[test]
    fn blocked_output_plugs_and_retries() {
        let mut b = TileBreaker::new(Direction::North);
        b.on_neighbor_change(true, Some(vec![stack("a", 1)]), &mut |_| false);
        assert!(b.buffer.plugged && b.buffer.size() == 1);
        assert_eq!(b.scheduled_tick, Some(5));
        b.on_tile_tick(&mut |_| false);
        assert_eq!(b.scheduled_tick, Some(10));
        b.on_tile_tick(&mut |_| true);
        assert!(b.buffer.is_empty() && !b.buffer.plugged);
        assert_eq!(b.scheduled_tick, Some(5));
    }

    #[test]
    fn unbreakable_front_does_nothing() {
        let mut b = TileBreaker::new(Direction::North);
        let out = b.on_neighbor_change(true, None, &mut |_| true);
        assert!(!out.remove_front_block && !b.flags.active && b.flags.powered);
    }

    #[test]
    fn rear_tube_bounce_only() {
        let mut b = TileBreaker::new(Direction::North);
        let it = BufferedItem {
            item: stack("a", 1),
            color: None,
        };
        assert!(!b.tube_item_enter(Direction::South, 2, it.clone()));
        assert!(!b.tube_item_enter(Direction::North, 1, it.clone()));
        assert!(b.tube_item_enter(Direction::North, 2, it));
        assert!(b.buffer.plugged && b.flags.active);
    }

    #[test]
    fn idle_tick_clears_active_when_unpowered() {
        let mut b = TileBreaker::new(Direction::North);
        b.flags.active = true;
        assert!(b.on_tile_tick(&mut |_| true));
        assert!(!b.flags.active);
    }
}
