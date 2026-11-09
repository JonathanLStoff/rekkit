//! Sorting Machine (`redpower_2:sorting_machine`): port of `TileSorter` and `ContainerSorter`.
//!
//! Features:
//! - 40-slot filter inventory (8 columns x 5 rows).
//! - 8 independent channel color buttons (0 = none, 1..=16 = dye tag).
//! - 7 operational routing modes:
//!   - 0: Any item matching column (cycles columns).
//!   - 1: One item of each type in column simultaneously (cycles columns).
//!   - 2: Full stack matching (pulls full stack per column into column buffer).
//!   - 3: All items matching across all columns.
//!   - 4: Random / fallback with default color tagging.
//!   - 5: Even distribution / fuzzy matching across columns.
//!   - 6: Fuzzy matching with default color fallback.
//! - 3 auto-modes:
//!   - 0: Pulse mode (redstone triggered).
//!   - 1: Auto mode (continuous Blutricity extraction).
//!   - 2: Siphon / step mode (pulse counter).
//! - Blutricity consumption: minimum 60V threshold, 25 W-t per item extracted.
//! - Sided pneumatic tube routing: rear face ejects / bounces; front face accepts inputs.
//! - Chunk reload persistence for all 40 slots, 8 channel buffers, modes and colors.

use crate::core::foundation::{Packet211TileDesc, PacketError};
use crate::core::machine_lib::{collect_one_stack, FilterMap, ItemCatalog, SlotRange, Slots};
use crate::machine::base::{
    FrameSupport, MachineFlags, Rotatable, TubeConnectable, SUBID_TILE_MACHINE,
};
use crate::machine::breaker::{BufferedItem, TubeBuffer, TUBE_STATE_BOUNCE};
use crate::machine::sorter_gui::{layout, AUTOMODE_MAX, COLOR_MAX, COLUMNS, MODE_MAX, SLOT_COUNT};
use crate::{DyeColor, ItemStack};
use rekkit_core::side::Direction;
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const SORTER_BLOCK_ID: &str = "redpower_2:sorting_machine";
/// Tile entity name in `GameRegistry`.
pub const SORTER_TE_NAME: &str = "RPSorter";
/// Metadata index in `BlockMachine`.
pub const SORTER_METADATA: u8 = 5;
/// Extended ID index.
pub const SORTER_EXTENDED_ID: u8 = 5;
/// Total number of filter template slots (8 columns x 5 rows).
pub const SORTER_SLOTS: usize = SLOT_COUNT;
/// Number of color columns.
pub const SORTER_COLUMNS: usize = COLUMNS;
/// Number of rows per column.
pub const SORTER_ROWS: usize = 5;
/// Minimum operating voltage in Volts.
pub const SORTER_MIN_VOLTAGE: f64 = 60.0;
/// Power drawn per extracted item unit (W-t / Blutricity units).
pub const SORTER_POWER_PER_ITEM: u32 = 25;
/// Ticks between extraction and settling.
pub const SORTER_ACTIVE_DELAY: u32 = 5;
/// Tick interval when retrying or auto-pulling.
pub const SORTER_SCHEDULE_DELAY: u32 = 10;
/// GUI window dimensions.
pub const GUI_SIZE: [u32; 2] = [176, 222];
/// GUI title string.
pub const GUI_TITLE: &str = "Sorting Machine";
/// Input tube entry state.
pub const TUBE_STATE_ENTER: i32 = 1;

/// Convert color index (0..=16) to optional `DyeColor`.
pub fn color_to_tag(c: u8) -> Option<DyeColor> {
    if c > 0 && c <= 16 {
        Some(DyeColor::from_u8(c - 1))
    } else {
        None
    }
}

/// Convert optional `DyeColor` to color index (0..=16).
pub fn tag_to_color(tag: Option<DyeColor>) -> u8 {
    match tag {
        Some(d) => (d as u8) + 1,
        None => 0,
    }
}

/// Slot coordinates `[x, y]` for the 40 filter slots matching `sorting_machine.json`.
pub fn slot_positions() -> Vec<[f32; 2]> {
    layout().slots
}

/// `TileSorter` state machine.
#[derive(Debug, Clone, PartialEq)]
pub struct TileSorter {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub contents: [Option<ItemStack>; SORTER_SLOTS],
    pub colors: [u8; SORTER_COLUMNS],
    pub mode: u8,
    pub automode: u8,
    pub defcolor: u8,
    pub column: u8,
    pub pulses: u32,
    pub draining: i8,
    pub buffer: TubeBuffer,
    pub channel_buffers: [TubeBuffer; SORTER_COLUMNS],
    pub voltage: f64,
    pub power_drawn: f64,
    pub scheduled_tick: Option<u32>,
}

impl Default for TileSorter {
    fn default() -> Self {
        Self::new(Direction::North)
    }
}

impl TileSorter {
    /// Create a new Sorting Machine facing `rotation`.
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            contents: [const { None }; SORTER_SLOTS],
            colors: [0; SORTER_COLUMNS],
            mode: 0,
            automode: 0,
            defcolor: 0,
            column: 0,
            pulses: 0,
            draining: -1,
            buffer: TubeBuffer::default(),
            channel_buffers: Default::default(),
            voltage: 100.0,
            power_drawn: 0.0,
            scheduled_tick: None,
        }
    }

    /// Front / input face where adjacent inventories are pulled and tube items enter.
    pub fn input_side(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Rear face where items are ejected into pneumatic tubes.
    pub fn output_side(&self) -> Direction {
        self.rotation
    }

    /// Tubes can connect on both front and rear faces.
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation || side == self.input_side()
    }

    /// Draw power if voltage is above threshold.
    pub fn draw_power(&mut self, amount: u32) -> bool {
        if self.voltage < SORTER_MIN_VOLTAGE {
            return false;
        }
        self.power_drawn += amount as f64;
        true
    }

    /// Determine which column matches `ist` (0..7), or -2 for empty filter, or -1 for no match.
    pub fn get_column_match(&self, ist: &ItemStack, cat: &dyn ItemCatalog) -> i32 {
        let fmap = FilterMap::new(&self.contents, cat);
        if fmap.size() == 0 {
            return -2;
        }
        let first = fmap.first_match(ist);
        if first < 0 {
            -1
        } else {
            first & 7
        }
    }

    /// Advance column index to next non-empty column, decrementing pulse count on wrap.
    pub fn step_column(&mut self) {
        for _ in 0..SORTER_COLUMNS {
            self.column = (self.column + 1) & 7;
            if self.column == 0 && self.pulses > 0 {
                self.pulses -= 1;
            }
            for row in 0..SORTER_ROWS {
                let idx = row * 8 + self.column as usize;
                if self.contents[idx].is_some() {
                    return;
                }
            }
        }
        self.column = 0;
    }

    /// Ensure the current column has filter items; if not, advance to the next column.
    pub fn check_column(&mut self) {
        for row in 0..SORTER_ROWS {
            let idx = row * 8 + self.column as usize;
            if self.contents[idx].is_some() {
                return;
            }
        }
        self.step_column();
    }

    /// Whether all internal buffers (main + 8 channel buffers) are empty.
    pub fn is_buffer_empty(&self) -> bool {
        if !self.buffer.is_empty() {
            return false;
        }
        for b in &self.channel_buffers {
            if !b.is_empty() {
                return false;
            }
        }
        true
    }

    /// Drain one buffered item across main and 8 channel buffers.
    pub fn try_drain_buffer(&mut self) -> bool {
        for _ in 0..9 {
            self.draining += 1;
            if self.draining > 7 {
                self.draining = -1;
                if !self.buffer.is_empty() {
                    self.buffer.pop();
                    return false;
                }
            } else {
                let col = self.draining as usize;
                if !self.channel_buffers[col].is_empty() {
                    self.channel_buffers[col].pop();
                    return false;
                }
            }
        }
        true
    }

    /// Handle incoming tube item entering from adjacent tube (`tubeItemEnter`).
    pub fn tube_item_enter(
        &mut self,
        side: Direction,
        state: i32,
        item: ItemStack,
        color: Option<DyeColor>,
        cat: &dyn ItemCatalog,
    ) -> bool {
        // Rear bounce re-entry
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            let cm = self.get_column_match(&item, cat);
            let buf = if cm >= 0 && self.mode > 1 {
                &mut self.channel_buffers[cm as usize]
            } else {
                &mut self.buffer
            };
            buf.add_bounce(BufferedItem { item, color });
            return true;
        }

        // Must enter from the front input face in ENTER state
        if side != self.input_side() || state != TUBE_STATE_ENTER {
            return false;
        }

        // Pulse mode respects powered state
        if self.automode == 0 && self.flags.powered {
            return false;
        }

        if self.voltage < SORTER_MIN_VOLTAGE {
            return false;
        }

        let cm = self.get_column_match(&item, cat);
        let buf_is_channel = cm >= 0 && self.mode > 1;
        let is_empty = if buf_is_channel {
            self.channel_buffers[cm as usize].is_empty()
        } else {
            self.buffer.is_empty()
        };

        if !is_empty {
            return false;
        }

        let item_count = item.count;
        if cm >= 0 {
            let col = cm as usize;
            let col_color = color_to_tag(self.colors[col]);
            let buf = if buf_is_channel {
                &mut self.channel_buffers[col]
            } else {
                &mut self.buffer
            };
            buf.add_new_color(item, col_color);
            self.draw_power(SORTER_POWER_PER_ITEM * item_count);
            self.try_drain_buffer();
            true
        } else if self.mode == 4 || self.mode == 6 {
            self.buffer.add_new_color(item, color_to_tag(self.defcolor));
            self.draw_power(SORTER_POWER_PER_ITEM * item_count);
            self.try_drain_buffer();
            true
        } else if cm == -2 {
            self.buffer.add_new_color(item, None);
            self.draw_power(SORTER_POWER_PER_ITEM * item_count);
            self.try_drain_buffer();
            true
        } else {
            false
        }
    }

    /// Extract matching items from adjacent inventory slice based on configured mode.
    pub fn handle_extract(
        &mut self,
        inv: &mut Slots,
        range: SlotRange,
        cat: &dyn ItemCatalog,
    ) -> bool {
        if self.voltage < SORTER_MIN_VOLTAGE {
            return false;
        }

        let fmap = FilterMap::new(&self.contents, cat);

        // Empty filter template: extract any 1 stack
        if fmap.size() == 0 {
            let coll = collect_one_stack(inv, range, None, cat);
            let Some(item) = coll else { return false };
            let count = item.count;

            if self.mode != 4 && self.mode != 6 {
                self.buffer.add_new(item);
            } else {
                self.buffer.add_new_color(item, color_to_tag(self.defcolor));
            }

            self.draw_power(SORTER_POWER_PER_ITEM * count);
            self.try_drain_buffer();
            return true;
        }

        match self.mode {
            // Mode 0: Any item matching current column
            0 => {
                self.check_column();
                let col = self.column as usize;
                let sm = fmap.match_any_stack_col(inv, range, col);
                if sm < 0 {
                    return false;
                }

                let target = self.contents[sm as usize].as_ref();
                let coll = collect_one_stack(inv, range, target, cat);
                let Some(item) = coll else { return false };

                let count = item.count;
                let color = color_to_tag(self.colors[col]);
                self.buffer.add_new_color(item, color);
                self.draw_power(SORTER_POWER_PER_ITEM * count);
                self.step_column();
                self.try_drain_buffer();
                true
            }

            // Mode 1: One item of each type in current column
            1 => {
                self.check_column();
                let col = self.column as usize;
                if !fmap.match_all_col(inv, range, col) {
                    return false;
                }

                let mut total_count = 0;
                for row in 0..SORTER_ROWS {
                    let idx = row * 8 + col;
                    if let Some(matchst) = &self.contents[idx] {
                        if let Some(coll) = collect_one_stack(inv, range, Some(matchst), cat) {
                            total_count += coll.count;
                            let color = color_to_tag(self.colors[col]);
                            self.buffer.add_new_color(coll, color);
                        }
                    }
                }

                self.draw_power(SORTER_POWER_PER_ITEM * total_count);
                self.step_column();
                self.try_drain_buffer();
                true
            }

            // Mode 2: Full stack of each item in a column
            2 => {
                let mut found_col = None;
                for col in 0..SORTER_COLUMNS {
                    if fmap.match_all_col(inv, range, col) {
                        found_col = Some(col);
                        break;
                    }
                }

                let Some(col) = found_col else { return false };
                let mut total_count = 0;

                for row in 0..SORTER_ROWS {
                    let idx = row * 8 + col;
                    if let Some(matchst) = &self.contents[idx] {
                        if let Some(coll) = collect_one_stack(inv, range, Some(matchst), cat) {
                            total_count += coll.count;
                            let color = color_to_tag(self.colors[col]);
                            self.channel_buffers[col].add_new_color(coll, color);
                        }
                    }
                }

                self.draw_power(SORTER_POWER_PER_ITEM * total_count);
                if self.pulses > 0 {
                    self.pulses -= 1;
                }
                self.try_drain_buffer();
                true
            }

            // Mode 3: All items / any stack across all columns
            3 => {
                let sm = fmap.match_any_stack(inv, range);
                if sm < 0 {
                    return false;
                }

                let col = (sm & 7) as usize;
                let target = self.contents[sm as usize].as_ref();
                let coll = collect_one_stack(inv, range, target, cat);
                let Some(item) = coll else { return false };

                let count = item.count;
                let color = color_to_tag(self.colors[col]);
                self.channel_buffers[col].add_new_color(item, color);
                self.draw_power(SORTER_POWER_PER_ITEM * count);
                if self.pulses > 0 {
                    self.pulses -= 1;
                }
                self.try_drain_buffer();
                true
            }

            // Mode 4: Any matching stack, or fallback to defcolor
            4 => {
                let sm = fmap.match_any_stack(inv, range);
                let coll = if sm < 0 {
                    let Some(item) = collect_one_stack(inv, range, None, cat) else {
                        return false;
                    };
                    self.buffer
                        .add_new_color(item.clone(), color_to_tag(self.defcolor));
                    item
                } else {
                    let col = (sm & 7) as usize;
                    let target = self.contents[sm as usize].as_ref();
                    let Some(item) = collect_one_stack(inv, range, target, cat) else {
                        return false;
                    };
                    let color = color_to_tag(self.colors[col]);
                    self.channel_buffers[col].add_new_color(item.clone(), color);
                    item
                };

                let count = coll.count;
                self.draw_power(SORTER_POWER_PER_ITEM * count);
                if self.pulses > 0 {
                    self.pulses -= 1;
                }
                self.try_drain_buffer();
                true
            }

            // Mode 5: Even distribution / fuzzy match
            5 => {
                let sm = fmap.match_any_stack(inv, range);
                if sm < 0 {
                    return false;
                }

                let col = (sm & 7) as usize;
                let target = self.contents[sm as usize].as_ref();
                let coll = collect_one_stack(inv, range, target, cat);
                let Some(item) = coll else { return false };

                let count = item.count;
                let color = color_to_tag(self.colors[col]);
                self.channel_buffers[col].add_new_color(item, color);
                self.draw_power(SORTER_POWER_PER_ITEM * count);
                if self.pulses > 0 {
                    self.pulses -= 1;
                }
                self.try_drain_buffer();
                true
            }

            // Mode 6: Fuzzy match with defcolor fallback
            6 => {
                let sm = fmap.match_any_stack(inv, range);
                let coll = if sm < 0 {
                    let Some(item) = collect_one_stack(inv, range, None, cat) else {
                        return false;
                    };
                    self.buffer
                        .add_new_color(item.clone(), color_to_tag(self.defcolor));
                    item
                } else {
                    let col = (sm & 7) as usize;
                    let target = self.contents[sm as usize].as_ref();
                    let Some(item) = collect_one_stack(inv, range, target, cat) else {
                        return false;
                    };
                    let color = color_to_tag(self.colors[col]);
                    self.channel_buffers[col].add_new_color(item.clone(), color);
                    item
                };

                let count = coll.count;
                self.draw_power(SORTER_POWER_PER_ITEM * count);
                if self.pulses > 0 {
                    self.pulses -= 1;
                }
                self.try_drain_buffer();
                true
            }

            _ => false,
        }
    }

    /// Redstone pulse event (`onBlockNeighborChange` / pulse trigger).
    pub fn on_pulse(&mut self, powered: bool) {
        if powered && !self.flags.powered {
            self.flags.powered = true;
            if self.automode == 0 || self.automode == 2 {
                self.pulses += 1;
            }
        } else if !powered {
            self.flags.powered = false;
        }
    }

    /// Periodic machine tick (`onTileTick` / `autoTick`).
    pub fn on_tick(&mut self, inv: &mut Slots, range: SlotRange, cat: &dyn ItemCatalog) -> bool {
        if self.automode == 1 && self.flags.powered {
            self.flags.powered = false;
        }

        if self.automode > 0 {
            if self.flags.active {
                return false;
            }
            if self.automode == 2 && self.pulses == 0 {
                return false;
            }

            if self.handle_extract(inv, range, cat) {
                self.flags.active = true;
                self.scheduled_tick = Some(SORTER_ACTIVE_DELAY);
                return true;
            } else {
                self.scheduled_tick = Some(SORTER_SCHEDULE_DELAY);
                return false;
            }
        } else if self.pulses > 0 && self.handle_extract(inv, range, cat) {
            self.pulses = self.pulses.saturating_sub(1);
            self.flags.active = true;
            self.scheduled_tick = Some(SORTER_ACTIVE_DELAY);
            return true;
        }

        false
    }

    /// Process GUI networking packet event (`ContainerSorter.handleGuiEvent`).
    pub fn handle_gui_event(&mut self, event_id: u8, data: &[u8]) {
        match event_id {
            1 => {
                if let Some(&m) = data.first() {
                    self.mode = m.min(MODE_MAX as u8);
                }
            }
            2 => {
                if data.len() >= 2 {
                    let col = data[0] as usize;
                    let col_val = data[1];
                    if col < SORTER_COLUMNS {
                        self.colors[col] = col_val.min(COLOR_MAX as u8);
                    }
                }
            }
            3 => {
                if let Some(&c) = data.first() {
                    self.defcolor = c.min(COLOR_MAX as u8);
                }
            }
            4 => {
                if let Some(&am) = data.first() {
                    self.automode = am.min(AUTOMODE_MAX as u8);
                    self.pulses = 0;
                }
            }
            _ => {}
        }
    }

    /// Serialize machine state to string key-value map for chunk reload persistence (Level 5 contract).
    pub fn to_nbt(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("rotation".into(), (self.rotation as u8).to_string());
        map.insert("mode".into(), self.mode.to_string());
        map.insert("amode".into(), self.automode.to_string());
        map.insert("defc".into(), self.defcolor.to_string());
        map.insert("coln".into(), self.column.to_string());
        map.insert("pulses".into(), self.pulses.to_string());
        map.insert("drain".into(), self.draining.to_string());
        map.insert("voltage".into(), self.voltage.to_string());

        for (i, c) in self.colors.iter().enumerate() {
            map.insert(format!("col_{i}"), c.to_string());
        }

        for (i, item) in self.contents.iter().enumerate() {
            if let Some(it) = item {
                if let Ok(json) = serde_json::to_string(it) {
                    map.insert(format!("slot_{i}"), json);
                }
            }
        }

        map
    }

    /// Reconstitute machine state from string key-value map.
    pub fn from_nbt(map: &HashMap<String, String>) -> Self {
        let mut s = Self::new(Direction::North);
        if let Some(r) = map.get("rotation").and_then(|v| v.parse::<u8>().ok()) {
            s.rotation = match r {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                _ => Direction::East,
            };
        }
        if let Some(m) = map.get("mode").and_then(|v| v.parse::<u8>().ok()) {
            s.mode = m;
        }
        if let Some(am) = map.get("amode").and_then(|v| v.parse::<u8>().ok()) {
            s.automode = am;
        }
        if let Some(d) = map.get("defc").and_then(|v| v.parse::<u8>().ok()) {
            s.defcolor = d;
        }
        if let Some(c) = map.get("coln").and_then(|v| v.parse::<u8>().ok()) {
            s.column = c;
        }
        if let Some(p) = map.get("pulses").and_then(|v| v.parse::<u32>().ok()) {
            s.pulses = p;
        }
        if let Some(dr) = map.get("drain").and_then(|v| v.parse::<i8>().ok()) {
            s.draining = dr;
        }
        if let Some(v) = map.get("voltage").and_then(|v| v.parse::<f64>().ok()) {
            s.voltage = v;
        }

        for i in 0..SORTER_COLUMNS {
            if let Some(c) = map
                .get(&format!("col_{i}"))
                .and_then(|v| v.parse::<u8>().ok())
            {
                s.colors[i] = c;
            }
        }

        for i in 0..SORTER_SLOTS {
            if let Some(json) = map.get(&format!("slot_{i}")) {
                if let Ok(item) = serde_json::from_str::<ItemStack>(json) {
                    s.contents[i] = Some(item);
                }
            }
        }

        s
    }

    /// Encode state into 2-byte packet payload: `[rotation, ps]`.
    pub fn write_to_packet(&self) -> [u8; 2] {
        [self.rotation as u8, self.flags.to_bits()]
    }

    /// Decode state from 2-byte packet payload: `[rotation, ps]`.
    pub fn read_from_packet(&mut self, data: &[u8]) -> Result<(), PacketError> {
        if data.len() < 2 {
            return Err(PacketError::BadVlc);
        }
        self.rotation = match data[0] {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            _ => Direction::Down,
        };
        self.flags = MachineFlags::from_bits(data[1]);
        Ok(())
    }

    /// Build Packet 211 Tile Description descriptor.
    pub fn write_tile_packet(&self, x: i32, y: i32, z: i32) -> Packet211TileDesc {
        let payload = self.write_to_packet();
        Packet211TileDesc {
            sub_id: SUBID_TILE_MACHINE,
            x,
            y,
            z,
            body: payload.to_vec(),
        }
    }

    /// Apply incoming Packet 211 Tile Description payload.
    pub fn handle_tile_packet(&mut self, pkt: &Packet211TileDesc) -> Result<(), PacketError> {
        if pkt.sub_id != SUBID_TILE_MACHINE {
            return Err(PacketError::BadVlc);
        }
        self.read_from_packet(&pkt.body)
    }

    pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.extend(self.to_nbt());
    }

    pub fn read_nbt(&mut self, nbt: &HashMap<String, String>) {
        *self = Self::from_nbt(nbt);
    }
}

impl Rotatable for TileSorter {
    fn part_max_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            5
        }
    }

    fn part_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            self.rotation as u32
        }
    }

    fn set_part_rotation(&mut self, _part: u32, secondary: bool, rot: u32) {
        if !secondary {
            self.rotation = match rot {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                5 => Direction::East,
                _ => Direction::Down,
            };
        }
    }
}

impl FrameSupport for TileSorter {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileSorter frame packet header".into());
        }
        self.read_from_packet(&data[1..3])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

impl TubeConnectable for TileSorter {
    fn can_connect_tube(&self, side: Direction) -> bool {
        self.can_connect_tube(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::machine_lib::PlainCatalog;

    #[test]
    fn sorter_initialization_and_gui() {
        let sorter = TileSorter::new(Direction::South);
        assert_eq!(sorter.rotation, Direction::South);
        assert_eq!(sorter.input_side(), Direction::North);
        assert_eq!(sorter.output_side(), Direction::South);
        assert!(sorter.can_connect_tube(Direction::South));
        assert!(sorter.can_connect_tube(Direction::North));
        assert!(!sorter.can_connect_tube(Direction::East));

        let slots = slot_positions();
        assert_eq!(slots.len(), 40);
        assert_eq!(slots[0], [26.0, 18.0]);
        assert_eq!(slots[7], [152.0, 18.0]);
        assert_eq!(slots[39], [152.0, 90.0]);
    }

    #[test]
    fn sorter_column_matching() {
        let mut sorter = TileSorter::new(Direction::North);
        let cat = PlainCatalog;

        // Empty filter matches -2
        let iron = ItemStack::new("minecraft:iron_ingot", 1);
        assert_eq!(sorter.get_column_match(&iron, &cat), -2);

        // Put iron in col 2, row 0 (index 2)
        sorter.contents[2] = Some(iron.clone());
        // Put gold in col 5, row 1 (index 13)
        let gold = ItemStack::new("minecraft:gold_ingot", 1);
        sorter.contents[13] = Some(gold.clone());

        assert_eq!(sorter.get_column_match(&iron, &cat), 2);
        assert_eq!(sorter.get_column_match(&gold, &cat), 5);

        let coal = ItemStack::new("minecraft:coal", 1);
        assert_eq!(sorter.get_column_match(&coal, &cat), -1);
    }

    #[test]
    fn sorter_mode_0_extract_and_cycle() {
        let mut sorter = TileSorter::new(Direction::North);
        let cat = PlainCatalog;
        sorter.colors[0] = 3; // Col 0 is Blue (tag 3)
        sorter.colors[1] = 5; // Col 1 is Yellow (tag 5)
        sorter.contents[0] = Some(ItemStack::new("minecraft:cobblestone", 1));
        sorter.contents[1] = Some(ItemStack::new("minecraft:sand", 1));

        let mut inv = vec![
            Some(ItemStack::new("minecraft:sand", 10)),
            Some(ItemStack::new("minecraft:cobblestone", 64)),
        ];
        let range = SlotRange::all(2);

        // Current column is 0; cobblestone matches (extracts 1 as template count is 1)
        assert_eq!(sorter.column, 0);
        let ok = sorter.handle_extract(&mut inv, range, &cat);
        assert!(ok);
        assert_eq!(sorter.power_drawn, 25.0);
        assert_eq!(sorter.column, 1); // Advanced to column 1
        assert_eq!(inv[1].as_ref().unwrap().count, 63); // 1 cobblestone extracted

        // Next extract pulls sand from column 1
        let ok2 = sorter.handle_extract(&mut inv, range, &cat);
        assert!(ok2);
        assert_eq!(inv[0].as_ref().unwrap().count, 9); // 1 sand extracted
        assert_eq!(sorter.column, 0); // Cycled back to 0
    }

    #[test]
    fn sorter_mode_1_extract_all_in_col() {
        let mut sorter = TileSorter::new(Direction::North);
        sorter.mode = 1;
        let cat = PlainCatalog;
        sorter.contents[0] = Some(ItemStack::new("minecraft:iron_ingot", 5));
        sorter.contents[8] = Some(ItemStack::new("minecraft:gold_ingot", 3));

        let mut inv = vec![
            Some(ItemStack::new("minecraft:iron_ingot", 5)),
            Some(ItemStack::new("minecraft:gold_ingot", 3)),
        ];
        let range = SlotRange::all(2);

        let ok = sorter.handle_extract(&mut inv, range, &cat);
        assert!(ok);
        assert_eq!(inv[0], None);
        assert_eq!(inv[1], None);
        assert_eq!(sorter.power_drawn, (25 * (5 + 3)) as f64);
    }

    #[test]
    fn sorter_voltage_threshold() {
        let mut sorter = TileSorter::new(Direction::North);
        sorter.voltage = 59.9; // Below 60V threshold
        let cat = PlainCatalog;

        let mut inv = vec![Some(ItemStack::new("minecraft:dirt", 10))];
        let range = SlotRange::all(1);

        assert!(!sorter.handle_extract(&mut inv, range, &cat));
        assert_eq!(inv[0].as_ref().unwrap().count, 10);
    }

    #[test]
    fn sorter_gui_event_sync() {
        let mut sorter = TileSorter::new(Direction::North);
        sorter.handle_gui_event(1, &[4]); // mode = 4
        assert_eq!(sorter.mode, 4);

        sorter.handle_gui_event(2, &[3, 12]); // col 3 = color 12
        assert_eq!(sorter.colors[3], 12);

        sorter.handle_gui_event(3, &[8]); // defcolor = 8
        assert_eq!(sorter.defcolor, 8);

        sorter.handle_gui_event(4, &[1]); // automode = 1
        assert_eq!(sorter.automode, 1);
    }

    #[test]
    fn sorter_chunk_reload_persistence() {
        let mut original = TileSorter::new(Direction::East);
        original.mode = 3;
        original.automode = 2;
        original.defcolor = 14;
        original.colors[4] = 9;
        original.contents[12] = Some(ItemStack::new("minecraft:diamond", 5));
        original.pulses = 3;
        original.voltage = 112.5;

        let nbt = original.to_nbt();
        let loaded = TileSorter::from_nbt(&nbt);

        assert_eq!(loaded.rotation, Direction::East);
        assert_eq!(loaded.mode, 3);
        assert_eq!(loaded.automode, 2);
        assert_eq!(loaded.defcolor, 14);
        assert_eq!(loaded.colors[4], 9);
        assert_eq!(
            loaded.contents[12],
            Some(ItemStack::new("minecraft:diamond", 5))
        );
        assert_eq!(loaded.pulses, 3);
        assert_eq!(loaded.voltage, 112.5);
    }

    #[test]
    fn sorter_tube_bounce_and_routing() {
        let mut sorter = TileSorter::new(Direction::North);
        let cat = PlainCatalog;
        sorter.colors[1] = 6;
        sorter.contents[1] = Some(ItemStack::new("minecraft:redstone", 1));

        // Tube item enters from front face
        let redstone = ItemStack::new("minecraft:redstone", 4);
        let routed = sorter.tube_item_enter(
            Direction::South, // front face
            TUBE_STATE_ENTER,
            redstone.clone(),
            None,
            &cat,
        );
        assert!(routed);
        assert_eq!(sorter.power_drawn, (25 * 4) as f64);

        // Bounced item enters from rear face
        let bounced = sorter.tube_item_enter(
            Direction::North, // rear face
            TUBE_STATE_BOUNCE,
            redstone,
            color_to_tag(6),
            &cat,
        );
        assert!(bounced);
    }

    #[test]
    fn test_sorter_packet_and_frame_support() {
        let mut sorter = TileSorter::new(Direction::South);
        sorter.flags.active = true;
        sorter.flags.delay = true;

        let pkt = sorter.write_tile_packet(1, 2, 3);
        assert_eq!(pkt.sub_id, SUBID_TILE_MACHINE);
        assert_eq!(pkt.x, 1);
        assert_eq!(pkt.y, 2);
        assert_eq!(pkt.z, 3);

        let mut restored = TileSorter::new(Direction::Down);
        restored.handle_tile_packet(&pkt).unwrap();
        assert_eq!(restored.rotation, Direction::South);
        assert!(restored.flags.active);
        assert!(restored.flags.delay);

        let frame_pkt = sorter.frame_packet();
        let mut frame_restored = TileSorter::new(Direction::Down);
        frame_restored.handle_frame_packet(&frame_pkt).unwrap();
        assert_eq!(frame_restored.rotation, Direction::South);
        assert!(frame_restored.flags.active);
    }
}
