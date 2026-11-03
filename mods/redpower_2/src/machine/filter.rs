//! Filter (`redpower_2:filter`): port of `TileFilter` (over `TileTranspose`) and `ContainerFilter`.
//!
//! Pure logic. Nine ghost template slots gate what may enter from the input end and pick which
//! stack a redstone pulse extracts from the inventory behind. Every item leaving the filter is
//! tagged with the configured dye tag (0 = none, 1..=16 = a colour).

use super::base::MachineFlags;
use super::breaker::{BufferedItem, TubeBuffer, TUBE_STATE_BOUNCE};
use crate::core::machine_lib::{collect_one_stack, FilterMap, ItemCatalog, SlotRange};
use crate::{DyeColor, ItemStack};
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};

/// Number of ghost filter slots.
pub const FILTER_SLOTS: usize = 9;
/// Per-slot stack limit (`c()`).
pub const FILTER_STACK_LIMIT: u32 = 64;
/// Highest dye tag value (0 = untagged).
pub const FILTER_MAX_COLOR: u8 = 16;
/// Extended id of `TileFilter` (inherited from `TileTranspose`).
pub const FILTER_EXTENDED_ID: u8 = 2;
/// GUI event id carrying the colour byte (`ContainerFilter.handleGuiEvent`).
pub const GUI_EVENT_COLOR: u8 = 1;
/// Colour button hit area (`GuiFilter.a`): inclusive x range, y range, GUI-relative.
pub const COLOR_BUTTON_X: (i32, i32) = (118, 129);
pub const COLOR_BUTTON_Y: (i32, i32) = (55, 66);
/// Filter grid origin and inventory origins (`ContainerFilter`).
pub const FILTER_GRID: (i32, i32) = (62, 17);
pub const INVENTORY_ORIGIN: (i32, i32) = (8, 84);
pub const HOTBAR_Y: i32 = 142;
/// Tube state of an item entering from the input end.
pub const TUBE_STATE_ENTER: i32 = 1;
/// Ticks before the active flag drops / the buffer is retried.
pub const FILTER_ACTIVE_DELAY: u32 = 5;
pub const FILTER_RETRY_DELAY: u32 = 10;
/// Paint colours drawn in the colour swatch (`GuiFilter.paintColors`), indexed by `color - 1`.
pub const PAINT_COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF8000, 0xFF00FF, 0x6C8CFF, 0xFFFF00, 0x00FF00, 0xFF6000, 0x535353, 0x939393,
    0x00FFFF, 0x7F00FF, 0x0000FF, 0x4F2F00, 0x008000, 0xFF0000, 0x1F1F1F,
];

/// Slot position of GUI slot `index` (0..9 filter, 9..36 inventory, 36..45 hotbar).
pub const fn slot_position(index: usize) -> (i32, i32) {
    if index < 9 {
        (
            FILTER_GRID.0 + (index % 3) as i32 * 18,
            FILTER_GRID.1 + (index / 3) as i32 * 18,
        )
    } else if index < 36 {
        let i = index - 9;
        (
            INVENTORY_ORIGIN.0 + (i % 9) as i32 * 18,
            INVENTORY_ORIGIN.1 + (i / 9) as i32 * 18,
        )
    } else {
        (INVENTORY_ORIGIN.0 + (index - 36) as i32 * 18, HOTBAR_Y)
    }
}

/// `TileFilter`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileFilter {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub buffer: TubeBuffer,
    /// Dye tag: 0 none, 1..=16 colour `n - 1`.
    pub color: u8,
    pub contents: [Option<ItemStack>; FILTER_SLOTS],
    pub scheduled_tick: Option<u32>,
}

impl TileFilter {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            buffer: TubeBuffer::default(),
            color: 0,
            contents: Default::default(),
            scheduled_tick: None,
        }
    }

    /// `Rotation ^ 1`: where items come in and the inventory is pulled from.
    pub fn input_side(&self) -> Direction {
        self.rotation.opposite()
    }

    /// `getTubeConnectableSides`: both ends of the axis.
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation || side == self.input_side()
    }

    fn schedule(&mut self, ticks: u32) {
        self.scheduled_tick = Some(ticks);
    }

    /// The tube colour applied to outgoing items.
    pub fn tag(&self) -> Option<DyeColor> {
        (self.color > 0).then(|| DyeColor::from_u8(self.color - 1))
    }

    /// `GuiFilter.changeColor` / `handleGuiEvent`: wraps over 0..=16.
    pub fn step_color(&mut self, increment: bool) {
        self.color = if increment {
            if self.color >= FILTER_MAX_COLOR {
                0
            } else {
                self.color + 1
            }
        } else if self.color == 0 {
            FILTER_MAX_COLOR
        } else {
            self.color - 1
        };
    }

    /// `ContainerFilter.handleGuiEvent`: event 1 sets the colour from the client byte.
    pub fn handle_gui_event(&mut self, event: u8, byte: u8) {
        if event == GUI_EVENT_COLOR {
            self.color = byte.min(FILTER_MAX_COLOR);
        }
    }

    /// Whether a click at GUI-relative (x, y) hits the colour button.
    pub fn is_color_button(x: i32, y: i32) -> bool {
        (COLOR_BUTTON_X.0..=COLOR_BUTTON_X.1).contains(&x)
            && (COLOR_BUTTON_Y.0..=COLOR_BUTTON_Y.1).contains(&y)
    }

    fn filter_map<'a>(&self, cat: &'a dyn ItemCatalog) -> FilterMap<'a> {
        FilterMap::new(&self.contents, cat)
    }

    /// `suckFilter` / the check in `tubeItemEnter`: an empty template accepts everything.
    pub fn accepts(&self, item: &ItemStack, cat: &dyn ItemCatalog) -> bool {
        let fm = self.filter_map(cat);
        fm.size() == 0 || fm.contains_key(item)
    }

    /// `tubeItemCanEnter`.
    pub fn tube_item_can_enter(
        &self,
        side: Direction,
        state: i32,
        item: &ItemStack,
        cat: &dyn ItemCatalog,
    ) -> bool {
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            return true;
        }
        if side == self.input_side() && state == TUBE_STATE_ENTER {
            return self.accepts(item, cat) && self.buffer.is_empty() && !self.flags.powered;
        }
        false
    }

    /// `tubeItemEnter`. The item is re-tagged with the filter's colour.
    pub fn tube_item_enter(
        &mut self,
        side: Direction,
        state: i32,
        item: BufferedItem,
        cat: &dyn ItemCatalog,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            self.buffer.add_bounce(item);
            self.flags.active = true;
            self.schedule(FILTER_ACTIVE_DELAY);
            return true;
        }
        if side != self.input_side() || state != TUBE_STATE_ENTER {
            return false;
        }
        if !self.accepts(&item.item, cat) || self.flags.powered || !self.buffer.is_empty() {
            return false;
        }
        let tag = self.tag();
        self.buffer.add_new_color(item.item, tag);
        self.flags.active = true;
        self.schedule(FILTER_ACTIVE_DELAY);
        self.drain_buffer(handle_item);
        true
    }

    /// `drainBuffer` (without the minecart branch, which the engine layer handles).
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

    /// `handleExtract(inv, start, len)`: on a pulse, pull one stack (or the template's wanted
    /// count) from `inv` into the tube. Returns whether something was extracted.
    pub fn handle_extract(
        &mut self,
        inv: &mut [Option<ItemStack>],
        range: SlotRange,
        cat: &dyn ItemCatalog,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        let fm = self.filter_map(cat);
        let taken = if fm.size() == 0 {
            collect_one_stack(inv, range, None, cat)
        } else {
            let sm = fm.match_any_stack(inv, range);
            if sm < 0 {
                return false;
            }
            let wanted = self.contents[sm as usize].clone();
            collect_one_stack(inv, range, wanted.as_ref(), cat)
        };
        let Some(ist) = taken else { return false };
        let tag = self.tag();
        self.buffer.add_new_color(ist, tag);
        self.drain_buffer(handle_item);
        true
    }

    /// `onBlockNeighborChange` redstone edge. `powered_now` is the signal; returns true when
    /// an extraction should be attempted (rising edge while not active) and marks the machine
    /// active. When the rising edge extracts nothing the engine clears nothing: the tick does.
    pub fn on_power_change(&mut self, powered_now: bool) -> bool {
        if powered_now {
            if !self.flags.powered {
                self.flags.powered = true;
                if !self.flags.active {
                    self.flags.active = true;
                    return true;
                }
            }
            false
        } else {
            if self.flags.active && self.scheduled_tick.is_none() {
                self.schedule(FILTER_ACTIVE_DELAY);
            }
            self.flags.powered = false;
            false
        }
    }

    /// `onTileTick`; returns true when the active flag changed.
    pub fn on_tile_tick(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        self.scheduled_tick = None;
        if !self.buffer.is_empty() {
            self.drain_buffer(handle_item);
            self.schedule(if self.buffer.is_empty() {
                FILTER_ACTIVE_DELAY
            } else {
                FILTER_RETRY_DELAY
            });
            false
        } else if !self.flags.powered {
            self.flags.active = false;
            true
        } else {
            false
        }
    }

    /// `getSizeInventorySide`: the template is reachable from the four side faces.
    pub fn inventory_size_for_side(&self, side: Direction) -> usize {
        if side == self.rotation || side == self.input_side() {
            0
        } else {
            FILTER_SLOTS
        }
    }

    /// `a(int, ItemStack)`: set a ghost slot, clamping the stack to the limit.
    pub fn set(&mut self, slot: usize, stack: Option<ItemStack>) {
        if let Some(cell) = self.contents.get_mut(slot) {
            *cell = stack.map(|mut s| {
                s.count = s.count.min(FILTER_STACK_LIMIT);
                s
            });
        }
    }

    /// `onBlockRemoval`: template and buffered items drop.
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
    use crate::core::machine_lib::PlainCatalog;

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
    fn template_gates_input_and_tags_color() {
        let mut f = TileFilter::new(Direction::North);
        f.set(0, Some(stack("a", 1)));
        f.color = 3;
        let mut seen = Vec::new();
        let mut out = |b: &BufferedItem| {
            seen.push(b.color);
            true
        };
        let c = PlainCatalog;
        assert!(!f.tube_item_enter(Direction::South, 1, bi("b", 1), &c, &mut out));
        assert!(f.tube_item_enter(Direction::South, 1, bi("a", 1), &c, &mut out));
        assert_eq!(seen, vec![Some(DyeColor::Magenta)]);
    }

    #[test]
    fn empty_template_accepts_all_and_powered_blocks() {
        let mut f = TileFilter::new(Direction::North);
        let c = PlainCatalog;
        assert!(f.tube_item_can_enter(Direction::South, 1, &stack("z", 1), &c));
        f.flags.powered = true;
        assert!(!f.tube_item_can_enter(Direction::South, 1, &stack("z", 1), &c));
        assert!(f.tube_item_can_enter(Direction::North, 2, &stack("z", 1), &c));
    }

    #[test]
    fn extraction_follows_template_count() {
        let mut f = TileFilter::new(Direction::North);
        f.set(0, Some(stack("a", 2)));
        let mut inv = vec![
            Some(stack("b", 5)),
            Some(stack("a", 1)),
            Some(stack("a", 4)),
        ];
        let c = PlainCatalog;
        let mut sent = Vec::new();
        let mut out = |b: &BufferedItem| {
            sent.push(b.item.clone());
            true
        };
        assert!(f.handle_extract(&mut inv, SlotRange::all(3), &c, &mut out));
        // Nothing matches once the template wants an item the inventory lacks.
        f.set(0, Some(stack("q", 1)));
        assert!(!f.handle_extract(&mut inv, SlotRange::all(3), &c, &mut out));
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].count, 2);
        assert_eq!(sent[0].item_id, "a");
    }

    #[test]
    fn empty_template_extracts_first_stack() {
        let mut f = TileFilter::new(Direction::North);
        let mut inv = vec![None, Some(stack("b", 5))];
        let c = PlainCatalog;
        let mut out = |_: &BufferedItem| true;
        assert!(f.handle_extract(&mut inv, SlotRange::all(2), &c, &mut out));
        assert!(inv[1].is_none());
    }

    #[test]
    fn color_wraps_and_event() {
        let mut f = TileFilter::new(Direction::North);
        f.step_color(false);
        assert_eq!(f.color, 16);
        f.step_color(true);
        assert_eq!(f.color, 0);
        assert_eq!(f.tag(), None);
        f.handle_gui_event(1, 5);
        assert_eq!(f.tag(), Some(DyeColor::Yellow));
        assert!(TileFilter::is_color_button(118, 66));
        assert!(!TileFilter::is_color_button(130, 60));
    }

    #[test]
    fn pulse_edge_and_slots() {
        let mut f = TileFilter::new(Direction::North);
        assert!(f.on_power_change(true));
        assert!(!f.on_power_change(true));
        f.on_power_change(false);
        assert_eq!(f.scheduled_tick, Some(5));
        assert_eq!(slot_position(8), (98, 53));
        assert_eq!(slot_position(44), (152, 142));
        assert_eq!(f.inventory_size_for_side(Direction::East), 9);
        assert_eq!(f.inventory_size_for_side(Direction::North), 0);
    }
}
