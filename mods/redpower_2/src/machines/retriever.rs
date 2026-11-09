//! Retriever (`redpower_2:retriever`): port of `TileRetriever` and `ContainerRetriever`.
//!
//! Features:
//! - 9-slot request filter template (`contents[9]`).
//! - Searches connected pneumatic tube network on redstone pulse to pull matching items.
//! - Blutricity consumption: requires minimum 60V threshold, draws 25 W-t per item.
//! - Sided tube routing: input on front face (`rotation.opposite()`), output on rear face (`rotation`).
//! - Selection modes: mode 0 (step / sequential slot selection), mode 1 (all matching slots).
//! - Color tagging: applies configured color tag (0..=16) to all pulled / ejected items.
//! - Chunk reload persistence for 9 filter slots, color, mode, buffer, and charge state.

use crate::core::foundation::{Packet211TileDesc, PacketError};
use crate::core::machine_lib::{collect_one_stack, FilterMap, ItemCatalog, SlotRange, Slots};
use crate::machine::base::{
    FrameSupport, MachineFlags, Rotatable, TubeConnectable, SUBID_TILE_MACHINE,
};
use crate::machine::breaker::{BufferedItem, TubeBuffer, TUBE_STATE_BOUNCE};
use crate::machine::logistics_gui::{
    retriever_layout, BTN_COLOR_NEXT, BTN_COLOR_PREV, BTN_MODE_NEXT, BTN_MODE_PREV,
};
use crate::{DyeColor, ItemStack};
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const RETRIEVER_BLOCK_ID: &str = "redpower_2:retriever";
/// Tile entity name in `GameRegistry`.
pub const RETRIEVER_TE_NAME: &str = "RPRetrieve";
/// Metadata index in `BlockMachine`.
pub const RETRIEVER_METADATA: u8 = 8;
/// Extended ID index.
pub const RETRIEVER_EXTENDED_ID: u8 = 8;
/// Number of filter template slots.
pub const RETRIEVER_SLOTS: usize = 9;
/// Minimum operating voltage in Volts.
pub const RETRIEVER_MIN_VOLTAGE: f64 = 60.0;
/// Power drawn per extracted item unit.
pub const RETRIEVER_POWER_PER_ITEM: u32 = 25;
/// Active delay in ticks.
pub const RETRIEVER_ACTIVE_DELAY: u32 = 5;
/// GUI window dimensions.
pub const GUI_SIZE: [u32; 2] = [176, 166];
/// GUI title string.
pub const GUI_TITLE: &str = "Retriever";
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

/// Slot coordinates `[x, y]` for the 9 filter slots matching `retriever.json`.
pub fn slot_positions() -> Vec<[f32; 2]> {
    retriever_layout().slots
}

/// `TileRetriever` state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileRetriever {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub contents: [Option<ItemStack>; RETRIEVER_SLOTS],
    pub color: u8,
    pub mode: u8,
    pub select: u8,
    pub buffer: TubeBuffer,
    pub voltage: f64,
    pub power_drawn: f64,
    pub scheduled_tick: Option<u32>,
}

impl Default for TileRetriever {
    fn default() -> Self {
        Self::new(Direction::North)
    }
}

impl TileRetriever {
    /// Create a new Retriever facing `rotation`.
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            contents: [const { None }; RETRIEVER_SLOTS],
            color: 0,
            mode: 0,
            select: 0,
            buffer: TubeBuffer::default(),
            voltage: 100.0,
            power_drawn: 0.0,
            scheduled_tick: None,
        }
    }

    /// Front / input face where pulled items arrive.
    pub fn input_side(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Rear face where pulled items are ejected into attached tubes.
    pub fn output_side(&self) -> Direction {
        self.rotation
    }

    /// Tubes connect on both front and rear faces.
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation || side == self.input_side()
    }

    /// Draw power if voltage is above threshold.
    pub fn draw_power(&mut self, amount: u32) -> bool {
        if self.voltage < RETRIEVER_MIN_VOLTAGE {
            return false;
        }
        self.power_drawn += amount as f64;
        true
    }

    /// Advance select index to next populated filter slot.
    pub fn step_select(&mut self) {
        for _ in 0..RETRIEVER_SLOTS {
            self.select = (self.select + 1) % (RETRIEVER_SLOTS as u8);
            if self.contents[self.select as usize].is_some() {
                return;
            }
        }
        self.select = 0;
    }

    /// Find matching slot for incoming item, or -1 if no match.
    pub fn get_match(&self, ist: &ItemStack, cat: &dyn ItemCatalog) -> i32 {
        let fmap = FilterMap::new(&self.contents, cat);
        fmap.first_match(ist)
    }

    /// Handle incoming tube item entering from adjacent tube (`tubeItemEnter`).
    pub fn tube_item_enter(
        &mut self,
        side: Direction,
        state: i32,
        item: ItemStack,
        tag: Option<DyeColor>,
        cat: &dyn ItemCatalog,
    ) -> bool {
        // Rear bounce re-entry
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            self.buffer.add_bounce(BufferedItem { item, color: tag });
            return true;
        }

        // Must enter from the front input face in ENTER state
        if side != self.input_side() || state != TUBE_STATE_ENTER {
            return false;
        }

        if self.voltage < RETRIEVER_MIN_VOLTAGE {
            return false;
        }

        let m = self.get_match(&item, cat);
        let fmap = FilterMap::new(&self.contents, cat);
        if fmap.size() > 0 && m < 0 {
            return false;
        }

        let count = item.count;
        let color = color_to_tag(self.color);
        self.buffer.add_new_color(item, color);
        self.draw_power(RETRIEVER_POWER_PER_ITEM * count);
        true
    }

    /// Extract matching items from adjacent inventory slice or network.
    pub fn handle_extract(
        &mut self,
        inv: &mut Slots,
        range: SlotRange,
        cat: &dyn ItemCatalog,
    ) -> bool {
        if self.voltage < RETRIEVER_MIN_VOLTAGE {
            return false;
        }

        let fmap = FilterMap::new(&self.contents, cat);

        if fmap.size() == 0 {
            let coll = collect_one_stack(inv, range, None, cat);
            let Some(item) = coll else { return false };
            let count = item.count;
            let color = color_to_tag(self.color);
            self.buffer.add_new_color(item, color);
            self.draw_power(RETRIEVER_POWER_PER_ITEM * count);
            return true;
        }

        if self.mode == 0 {
            // Mode 0: Sequential single slot pull
            let idx = self.select as usize;
            let target = self.contents[idx].as_ref();
            let coll = collect_one_stack(inv, range, target, cat);
            let Some(item) = coll else { return false };

            let count = item.count;
            let color = color_to_tag(self.color);
            self.buffer.add_new_color(item, color);
            self.draw_power(RETRIEVER_POWER_PER_ITEM * count);
            self.step_select();
            true
        } else {
            // Mode 1: Pull any matching stack in filter map
            let sm = fmap.match_any_stack(inv, range);
            if sm < 0 {
                return false;
            }

            let target = self.contents[sm as usize].as_ref();
            let coll = collect_one_stack(inv, range, target, cat);
            let Some(item) = coll else { return false };

            let count = item.count;
            let color = color_to_tag(self.color);
            self.buffer.add_new_color(item, color);
            self.draw_power(RETRIEVER_POWER_PER_ITEM * count);
            true
        }
    }

    /// Redstone pulse event (`onBlockNeighborChange`).
    pub fn on_pulse(&mut self, powered: bool) {
        if powered && !self.flags.powered {
            self.flags.powered = true;
            self.flags.active = true;
            self.scheduled_tick = Some(RETRIEVER_ACTIVE_DELAY);
        } else if !powered {
            self.flags.powered = false;
        }
    }

    /// Process GUI networking packet event (`ContainerRetriever.handleGuiEvent`).
    pub fn handle_gui_event(&mut self, event_id: i32) {
        match event_id {
            BTN_MODE_NEXT => self.mode = (self.mode + 1).rem_euclid(2),
            BTN_MODE_PREV => self.mode = (self.mode + 1).rem_euclid(2),
            BTN_COLOR_NEXT => self.color = (self.color + 1).rem_euclid(17),
            BTN_COLOR_PREV => self.color = (self.color + 16).rem_euclid(17),
            _ => {}
        }
    }

    /// Serialize machine state to key-value string map for chunk reload persistence (Level 5 contract).
    pub fn to_nbt(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("rotation".into(), (self.rotation as u8).to_string());
        map.insert("color".into(), self.color.to_string());
        map.insert("mode".into(), self.mode.to_string());
        map.insert("select".into(), self.select.to_string());
        map.insert("voltage".into(), self.voltage.to_string());

        for (i, item) in self.contents.iter().enumerate() {
            if let Some(it) = item {
                if let Ok(json) = serde_json::to_string(it) {
                    map.insert(format!("slot_{i}"), json);
                }
            }
        }

        map
    }

    /// Reconstitute machine state from key-value string map.
    pub fn from_nbt(map: &HashMap<String, String>) -> Self {
        let mut r = Self::new(Direction::North);
        if let Some(rot) = map.get("rotation").and_then(|v| v.parse::<u8>().ok()) {
            r.rotation = match rot {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                _ => Direction::East,
            };
        }
        if let Some(c) = map.get("color").and_then(|v| v.parse::<u8>().ok()) {
            r.color = c;
        }
        if let Some(m) = map.get("mode").and_then(|v| v.parse::<u8>().ok()) {
            r.mode = m;
        }
        if let Some(s) = map.get("select").and_then(|v| v.parse::<u8>().ok()) {
            r.select = s;
        }
        if let Some(v) = map.get("voltage").and_then(|v| v.parse::<f64>().ok()) {
            r.voltage = v;
        }

        for i in 0..RETRIEVER_SLOTS {
            if let Some(json) = map.get(&format!("slot_{i}")) {
                if let Ok(item) = serde_json::from_str::<ItemStack>(json) {
                    r.contents[i] = Some(item);
                }
            }
        }

        r
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

impl Rotatable for TileRetriever {
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

impl FrameSupport for TileRetriever {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileRetriever frame packet header".into());
        }
        self.read_from_packet(&data[1..3])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

impl TubeConnectable for TileRetriever {
    fn can_connect_tube(&self, side: Direction) -> bool {
        self.can_connect_tube(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::machine_lib::PlainCatalog;

    #[test]
    fn test_retriever_initialization_and_gui() {
        let ret = TileRetriever::new(Direction::East);
        assert_eq!(ret.rotation, Direction::East);
        assert_eq!(ret.input_side(), Direction::West);
        assert_eq!(ret.output_side(), Direction::East);
        assert!(ret.can_connect_tube(Direction::East));
        assert!(ret.can_connect_tube(Direction::West));
        assert!(!ret.can_connect_tube(Direction::North));

        let slots = slot_positions();
        assert_eq!(slots.len(), 9);
        assert_eq!(slots[0], [62.0, 17.0]);
        assert_eq!(slots[8], [98.0, 53.0]);
    }

    #[test]
    fn test_retriever_pull_and_power() {
        let mut ret = TileRetriever::new(Direction::North);
        ret.color = 4; // Yellow
        ret.contents[0] = Some(ItemStack::new("minecraft:iron_ingot", 2));
        let cat = PlainCatalog;

        let mut inv = vec![Some(ItemStack::new("minecraft:iron_ingot", 10))];
        let range = SlotRange::all(1);

        assert!(ret.handle_extract(&mut inv, range, &cat));
        assert_eq!(inv[0].as_ref().unwrap().count, 8); // 2 extracted
        assert_eq!(ret.power_drawn, (25 * 2) as f64);
        assert_eq!(ret.buffer.size(), 1);
    }

    #[test]
    fn test_retriever_voltage_gating() {
        let mut ret = TileRetriever::new(Direction::North);
        ret.voltage = 59.0; // Below 60V
        let cat = PlainCatalog;

        let mut inv = vec![Some(ItemStack::new("minecraft:iron_ingot", 10))];
        let range = SlotRange::all(1);

        assert!(!ret.handle_extract(&mut inv, range, &cat));
        assert_eq!(inv[0].as_ref().unwrap().count, 10);
    }

    #[test]
    fn test_retriever_chunk_persistence() {
        let mut original = TileRetriever::new(Direction::South);
        original.color = 11;
        original.mode = 1;
        original.select = 4;
        original.voltage = 95.0;
        original.contents[2] = Some(ItemStack::new("minecraft:diamond", 3));

        let nbt = original.to_nbt();
        let restored = TileRetriever::from_nbt(&nbt);

        assert_eq!(restored.rotation, Direction::South);
        assert_eq!(restored.color, 11);
        assert_eq!(restored.mode, 1);
        assert_eq!(restored.select, 4);
        assert_eq!(restored.voltage, 95.0);
        assert_eq!(
            restored.contents[2],
            Some(ItemStack::new("minecraft:diamond", 3))
        );
    }

    #[test]
    fn test_retriever_packet_and_frame_support() {
        let mut ret = TileRetriever::new(Direction::East);
        ret.flags.active = true;
        ret.flags.charged = true;

        let pkt = ret.write_tile_packet(5, 60, -10);
        assert_eq!(pkt.sub_id, SUBID_TILE_MACHINE);
        assert_eq!(pkt.x, 5);
        assert_eq!(pkt.y, 60);
        assert_eq!(pkt.z, -10);

        let mut restored = TileRetriever::new(Direction::Down);
        restored.handle_tile_packet(&pkt).unwrap();
        assert_eq!(restored.rotation, Direction::East);
        assert!(restored.flags.active);
        assert!(restored.flags.charged);

        let frame_pkt = ret.frame_packet();
        let mut frame_restored = TileRetriever::new(Direction::Down);
        frame_restored.handle_frame_packet(&frame_pkt).unwrap();
        assert_eq!(frame_restored.rotation, Direction::East);
        assert!(frame_restored.flags.active);
    }
}
