//! Item Detector (`redpower_2:item_detector`): port of `TileItemDetect`.
//!
//! Pure logic. Items enter at the front-facing tube end (`Rotation ^ 1`, state 1), are matched
//! against a nine-slot filter template and counted; the count is turned into redstone pulses
//! by `on_game_tick` (`g()`). The item then continues out of the rear (`Rotation`), through
//! the engine-supplied `handleItem` callback. Bounced items come back in at the rear (state 2).

use super::base::MachineFlags;
use super::breaker::{BufferedItem, TubeBuffer, TUBE_STATE_BOUNCE};
use crate::ItemStack;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};

/// Number of filter slots (`k_()`).
pub const DETECTOR_SLOTS: usize = 9;
/// Per-slot stack limit (`c()`).
pub const DETECTOR_STACK_LIMIT: u32 = 64;
/// Tube state of an item entering from the input end.
pub const TUBE_STATE_ENTER: i32 = 1;
/// Ticks before the active flag drops after an item passed or bounced.
pub const DETECTOR_ACTIVE_DELAY: u32 = 5;
/// Ticks before retrying when the buffer could not be emptied.
pub const DETECTOR_RETRY_DELAY: u32 = 10;
/// Extended id of `TileItemDetect`.
pub const DETECTOR_EXTENDED_ID: u8 = 4;
/// GUI event id carrying the mode byte (`ContainerItemDetect.handleGuiEvent`).
pub const GUI_EVENT_MODE: u8 = 1;
/// Mode button area in the GUI: x, y, width, height (`GuiItemDetect`).
pub const MODE_BUTTON: [i32; 4] = [117, 54, 15, 15];

/// What the detector counts (`mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectMode {
    /// Count every item of a matching stack.
    Items = 0,
    /// Count one per matching stack.
    Stacks = 1,
    /// Signal while any item is held in the buffer.
    Continuous = 2,
}

impl DetectMode {
    pub const fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Stacks,
            2 => Self::Continuous,
            _ => Self::Items,
        }
    }
    /// Left click on the GUI button.
    pub const fn next(self) -> Self {
        Self::from_u8((self as u8 + 1) % 3)
    }
    /// Right click on the GUI button.
    pub const fn prev(self) -> Self {
        Self::from_u8((self as u8 + 2) % 3)
    }
}

/// `TileItemDetect`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileItemDetector {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub buffer: TubeBuffer,
    /// Pulses still owed (`count`).
    pub count: i32,
    pub mode: DetectMode,
    pub contents: [Option<ItemStack>; DETECTOR_SLOTS],
    pub scheduled_tick: Option<u32>,
    /// Set whenever the redstone output changed; the engine notifies neighbours and clears it.
    #[serde(skip)]
    pub signal_changed: bool,
}

impl TileItemDetector {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            buffer: TubeBuffer::default(),
            count: 0,
            mode: DetectMode::Items,
            contents: Default::default(),
            scheduled_tick: None,
            signal_changed: false,
        }
    }

    /// `Rotation ^ 1`: where items come in.
    pub fn input_side(&self) -> Direction {
        self.rotation.opposite()
    }

    /// `getTubeConnectableSides`: both ends of the axis.
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation || side == self.rotation.opposite()
    }

    fn schedule(&mut self, ticks: u32) {
        self.scheduled_tick = Some(ticks);
    }

    /// `FilterMap.containsKey`: id and damage equal. Empty slots do not count.
    pub fn filter_matches(&self, item: &ItemStack) -> bool {
        self.contents
            .iter()
            .flatten()
            .filter(|f| f.count != 0)
            .any(|f| f.item_id == item.item_id && f.damage == item.damage)
    }

    fn filter_is_empty(&self) -> bool {
        !self.contents.iter().flatten().any(|f| f.count != 0)
    }

    /// `tubeItemCanEnter`.
    pub fn tube_item_can_enter(&self, side: Direction, state: i32) -> bool {
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            true
        } else {
            side == self.input_side() && state == TUBE_STATE_ENTER && self.buffer.is_empty()
        }
    }

    /// `tubeItemEnter`.
    pub fn tube_item_enter(
        &mut self,
        side: Direction,
        state: i32,
        item: BufferedItem,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            self.buffer.add_bounce(item);
            self.flags.active = true;
            self.schedule(DETECTOR_ACTIVE_DELAY);
            return true;
        }
        if side != self.input_side() || state != TUBE_STATE_ENTER || !self.buffer.is_empty() {
            return false;
        }
        let matched = self.filter_is_empty() || self.filter_matches(&item.item);
        let n = item.item.count as i32;
        self.buffer.add(item);
        if matched {
            match self.mode {
                DetectMode::Items => self.count += n,
                DetectMode::Stacks => self.count += 1,
                DetectMode::Continuous => {}
            }
        }
        self.flags.active = true;
        self.schedule(DETECTOR_ACTIVE_DELAY);
        self.drain_buffer(handle_item);
        true
    }

    fn start_continuous(&mut self) {
        if self.mode == DetectMode::Continuous && !self.flags.powered {
            self.flags.delay = false;
            self.flags.powered = true;
            self.count = 0;
            self.signal_changed = true;
        }
    }

    /// `drainBuffer`.
    pub fn drain_buffer(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) {
        while let Some(last) = self.buffer.last() {
            if !handle_item(last) {
                self.buffer.plugged = true;
                self.start_continuous();
                return;
            }
            self.buffer.pop();
            if self.buffer.plugged {
                self.start_continuous();
                return;
            }
        }
        if self.mode == DetectMode::Continuous && self.flags.powered {
            self.flags.powered = false;
            self.signal_changed = true;
        }
    }

    /// `g()`: the per-tick pulse generator for the counting modes.
    pub fn on_game_tick(&mut self) {
        if self.mode == DetectMode::Continuous {
            return;
        }
        if self.flags.powered {
            if self.flags.delay {
                self.flags.delay = false;
            } else {
                self.flags.powered = false;
                if self.count > 0 {
                    self.flags.delay = true;
                }
                self.signal_changed = true;
            }
        } else if self.count != 0 {
            if self.flags.delay {
                self.flags.delay = false;
            } else {
                self.count -= 1;
                self.flags.powered = true;
                self.flags.delay = true;
                self.signal_changed = true;
            }
        }
    }

    /// `onTileTick`; returns true when the active flag changed (client update needed).
    pub fn on_tile_tick(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        self.scheduled_tick = None;
        if !self.buffer.is_empty() {
            self.drain_buffer(handle_item);
            if !self.buffer.is_empty() {
                self.schedule(DETECTOR_RETRY_DELAY);
            } else {
                self.schedule(DETECTOR_ACTIVE_DELAY);
            }
            false
        } else {
            self.flags.active = false;
            true
        }
    }

    /// `isPoweringTo`: every face but the input end.
    pub fn is_powering_to(&self, side: Direction) -> bool {
        side != self.input_side() && self.flags.powered
    }

    /// `handleGuiEvent` (event 1): set the mode from the client's byte.
    pub fn set_mode_from_gui(&mut self, byte: u8) {
        self.mode = DetectMode::from_u8(byte);
    }

    /// `getSizeInventorySide`: the filter is reachable from every side.
    pub const fn inventory_size_for_side(&self) -> usize {
        DETECTOR_SLOTS
    }

    /// `a(int, ItemStack)`: set a slot, clamping the stack to the limit.
    pub fn set(&mut self, slot: usize, stack: Option<ItemStack>) {
        if let Some(cell) = self.contents.get_mut(slot) {
            *cell = stack.map(|mut s| {
                s.count = s.count.min(DETECTOR_STACK_LIMIT);
                s
            });
        }
    }

    /// `onBlockRemoval`: filter template and buffered items drop.
    pub fn drain_for_drop(&mut self) -> Vec<ItemStack> {
        let mut out: Vec<ItemStack> = self
            .contents
            .iter_mut()
            .filter_map(|s| s.take())
            .filter(|s| s.count > 0)
            .collect();
        out.extend(self.buffer.take_all_for_drop());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: &str, n: u32) -> ItemStack {
        ItemStack {
            item_id: id.to_string(),
            count: n,
            damage: 0,
            max_stack_size: 64,
        }
    }
    fn bi(id: &str, n: u32) -> BufferedItem {
        BufferedItem {
            item: stack(id, n),
            color: None,
        }
    }

    #[test]
    fn counts_and_pulses() {
        let mut d = TileItemDetector::new(Direction::North);
        let mut out = |_: &BufferedItem| true;
        assert!(d.tube_item_enter(Direction::South, 1, bi("a", 3), &mut out));
        assert_eq!(d.count, 3);
        assert!(d.buffer.is_empty());
        d.on_game_tick();
        assert!(d.flags.powered && d.signal_changed);
        assert!(d.is_powering_to(Direction::North));
        assert!(!d.is_powering_to(Direction::South));
        assert_eq!(d.count, 2);
    }

    #[test]
    fn filter_and_stack_mode() {
        let mut d = TileItemDetector::new(Direction::North);
        d.set(0, Some(stack("a", 1)));
        d.mode = DetectMode::Stacks;
        let mut out = |_: &BufferedItem| true;
        d.tube_item_enter(Direction::South, 1, bi("b", 5), &mut out);
        assert_eq!(d.count, 0);
        d.tube_item_enter(Direction::South, 1, bi("a", 5), &mut out);
        assert_eq!(d.count, 1);
    }

    #[test]
    fn continuous_while_plugged() {
        let mut d = TileItemDetector::new(Direction::North);
        d.mode = DetectMode::Continuous;
        let mut refuse = |_: &BufferedItem| false;
        d.tube_item_enter(Direction::South, 1, bi("a", 1), &mut refuse);
        assert!(d.flags.powered);
        assert!(!d.tube_item_can_enter(Direction::South, 1));
        let mut ok = |_: &BufferedItem| true;
        d.on_tile_tick(&mut ok);
        assert!(!d.flags.powered);
    }

    #[test]
    fn rejects_wrong_side_and_mode_cycle() {
        let mut d = TileItemDetector::new(Direction::North);
        let mut out = |_: &BufferedItem| true;
        assert!(!d.tube_item_enter(Direction::North, 1, bi("a", 1), &mut out));
        assert_eq!(DetectMode::Items.prev(), DetectMode::Continuous);
        assert_eq!(DetectMode::Continuous.next(), DetectMode::Items);
    }
}
