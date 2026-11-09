//! Regulator (`redpower_2:regulator`): port of `TileRegulator` and `ContainerRegulator`.
//!
//! Features:
//! - 27 inventory slots:
//!   - Slots 0..9: Stocking target template.
//!   - Slots 9..18: Input buffer / holding slots.
//!   - Slots 18..27: Excess export filter template.
//! - Maintains exact item counts in an adjacent inventory using attached pneumatic tubes.
//! - Blutricity consumption: minimum 60V threshold, 25 W-t per item transferred.
//! - Chunk reload persistence for all 27 slots, buffer, mode, and color channel.

use crate::core::foundation::{Packet211TileDesc, PacketError};
use crate::core::machine_lib::{
    collect_one_stack, compare_item, FilterMap, ItemCatalog, SlotRange, Slots,
};
use crate::machine::base::{
    FrameSupport, MachineFlags, Rotatable, TubeConnectable, SUBID_TILE_MACHINE,
};
use crate::machine::breaker::{BufferedItem, TubeBuffer, TUBE_STATE_BOUNCE};
use crate::machine::gui_management::{
    regulator_gui_layout, BTN_COLOR_NEXT, BTN_COLOR_PREV, BTN_MODE_NEXT, BTN_MODE_PREV,
};
use crate::{DyeColor, ItemStack};
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const REGULATOR_BLOCK_ID: &str = "redpower_2:regulator";
/// Tile entity name in `GameRegistry`.
pub const REGULATOR_TE_NAME: &str = "RPRegulate";
/// Metadata index in `BlockMachine`.
pub const REGULATOR_METADATA: u8 = 10;
/// Extended ID index.
pub const REGULATOR_EXTENDED_ID: u8 = 10;
/// Total number of regulator slots (9 template + 9 buffer + 9 excess filter).
pub const REGULATOR_SLOTS: usize = 27;
/// Minimum operating voltage in Volts.
pub const REGULATOR_MIN_VOLTAGE: f64 = 60.0;
/// Power drawn per transferred item unit.
pub const REGULATOR_POWER_PER_ITEM: u32 = 25;
/// Active delay in ticks.
pub const REGULATOR_ACTIVE_DELAY: u32 = 5;
/// GUI window dimensions.
pub const GUI_SIZE: [u32; 2] = [211, 167];
/// GUI title string.
pub const GUI_TITLE: &str = "Regulator";
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

/// Slot coordinates `[x, y]` for the 27 slots matching `regulator.json`.
pub fn slot_positions() -> Vec<[f32; 2]> {
    regulator_gui_layout().slots
}

/// `TileRegulator` state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileRegulator {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub contents: [Option<ItemStack>; REGULATOR_SLOTS],
    pub mode: u8,
    pub color: u8,
    pub buffer: TubeBuffer,
    pub voltage: f64,
    pub power_drawn: f64,
    pub scheduled_tick: Option<u32>,
}

impl Default for TileRegulator {
    fn default() -> Self {
        Self::new(Direction::North)
    }
}

impl TileRegulator {
    /// Create a new Regulator facing `rotation`.
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            contents: [const { None }; REGULATOR_SLOTS],
            mode: 0,
            color: 0,
            buffer: TubeBuffer::default(),
            voltage: 100.0,
            power_drawn: 0.0,
            scheduled_tick: None,
        }
    }

    /// Front face pointing to the regulated adjacent inventory (`Rotation ^ 1`).
    pub fn regulated_inventory_side(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Rear face where pneumatic tube connects (`Rotation`).
    pub fn output_side(&self) -> Direction {
        self.rotation
    }

    /// Tubes connect on the rear face (and optionally front for incoming supply).
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation || side == self.regulated_inventory_side()
    }

    /// Draw power if voltage is above threshold.
    pub fn draw_power(&mut self, amount: u32) -> bool {
        if self.voltage < REGULATOR_MIN_VOLTAGE {
            return false;
        }
        self.power_drawn += amount as f64;
        true
    }

    /// Balance contents between buffer (slots 9..18) and target stocking template (slots 0..9).
    pub fn sync_stocking(
        &mut self,
        adjacent_inv: &mut Slots,
        range: SlotRange,
        cat: &dyn ItemCatalog,
    ) -> bool {
        if self.voltage < REGULATOR_MIN_VOLTAGE {
            return false;
        }

        let mut changed = false;
        let end = (range.start + range.len).min(adjacent_inv.len());

        // 1. Move items from buffer (9..18) into adjacent inventory if wanted by template (0..9)
        for i in 0..9 {
            let buf_idx = 9 + i;
            let tmpl_idx = i;

            if let Some(template) = self.contents[tmpl_idx].clone() {
                // Count current items in adjacent inventory matching template
                let current_have: u32 = (range.start..end)
                    .filter_map(|idx| adjacent_inv.get(idx).and_then(|s| s.as_ref()))
                    .filter(|s| compare_item(s, &template, cat) == 0)
                    .map(|s| s.count)
                    .sum();

                let needed = template.count.saturating_sub(current_have);
                if needed > 0 {
                    if let Some(buf_item) = &mut self.contents[buf_idx] {
                        if compare_item(buf_item, &template, cat) == 0 {
                            let transfer = needed.min(buf_item.count);
                            // Deposit into adjacent inventory
                            let mut deposited = 0;
                            for slot_opt in &mut adjacent_inv[range.start..end] {
                                if deposited >= transfer {
                                    break;
                                }
                                let to_put = transfer - deposited;
                                if slot_opt.is_none() {
                                    *slot_opt = Some(ItemStack::new(&buf_item.item_id, to_put));
                                    deposited += to_put;
                                } else if let Some(slot) = slot_opt.as_mut() {
                                    if compare_item(slot, buf_item, cat) == 0 {
                                        let space = slot.max_stack_size.saturating_sub(slot.count);
                                        let can_add = space.min(to_put);
                                        slot.count += can_add;
                                        deposited += can_add;
                                    }
                                }
                            }

                            if deposited > 0 {
                                buf_item.count -= deposited;
                                if buf_item.count == 0 {
                                    self.contents[buf_idx] = None;
                                }
                                self.draw_power(REGULATOR_POWER_PER_ITEM * deposited);
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        // 2. Export excess items into rear tube buffer
        if let Some(excess_item) = self.scan_and_extract_excess(adjacent_inv, range, cat) {
            let count = excess_item.count;
            let tag = color_to_tag(self.color);
            self.buffer.add_new_color(excess_item, tag);
            self.draw_power(REGULATOR_POWER_PER_ITEM * count);
            changed = true;
        }

        changed
    }

    /// Extract excess items exceeding template counts and matching export filter (18..27).
    fn scan_and_extract_excess(
        &mut self,
        adjacent_inv: &mut Slots,
        range: SlotRange,
        cat: &dyn ItemCatalog,
    ) -> Option<ItemStack> {
        let export_map = FilterMap::new(&self.contents[18..27], cat);
        let end = (range.start + range.len).min(adjacent_inv.len());

        for i in 0..9 {
            if let Some(template) = &self.contents[i] {
                let current_have: u32 = (range.start..end)
                    .filter_map(|idx| adjacent_inv.get(idx).and_then(|s| s.as_ref()))
                    .filter(|s| compare_item(s, template, cat) == 0)
                    .map(|s| s.count)
                    .sum();

                if current_have > template.count {
                    // Check if excess matches export filter
                    if export_map.size() == 0 || export_map.contains_key(template) {
                        return collect_one_stack(adjacent_inv, range, Some(template), cat);
                    }
                }
            }
        }

        None
    }

    /// Handle incoming tube item entering buffer (slots 9..18) or bounce re-entry.
    pub fn tube_item_enter(
        &mut self,
        side: Direction,
        state: i32,
        item: ItemStack,
        tag: Option<DyeColor>,
        _cat: &dyn ItemCatalog,
    ) -> bool {
        // Rear bounce re-entry
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            self.buffer.add_bounce(BufferedItem { item, color: tag });
            return true;
        }

        // Incoming stock supplies buffer slots 9..18
        for i in 9..18 {
            if self.contents[i].is_none() {
                self.contents[i] = Some(item);
                return true;
            } else if let Some(slot) = &mut self.contents[i] {
                if slot.item_id == item.item_id && slot.damage == item.damage {
                    let space = slot.max_stack_size.saturating_sub(slot.count);
                    if space >= item.count {
                        slot.count += item.count;
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Process GUI networking packet event.
    pub fn handle_gui_event(&mut self, event_id: i32) {
        match event_id {
            BTN_MODE_NEXT => self.mode = (self.mode + 1).rem_euclid(3),
            BTN_MODE_PREV => self.mode = (self.mode + 2).rem_euclid(3),
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
        if let Some(v) = map.get("voltage").and_then(|v| v.parse::<f64>().ok()) {
            r.voltage = v;
        }

        for i in 0..REGULATOR_SLOTS {
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

impl Rotatable for TileRegulator {
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

impl FrameSupport for TileRegulator {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileRegulator frame packet header".into());
        }
        self.read_from_packet(&data[1..3])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

impl TubeConnectable for TileRegulator {
    fn can_connect_tube(&self, side: Direction) -> bool {
        self.can_connect_tube(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::machine_lib::PlainCatalog;

    #[test]
    fn test_regulator_initialization_and_gui() {
        let reg = TileRegulator::new(Direction::South);
        assert_eq!(reg.rotation, Direction::South);
        assert_eq!(reg.regulated_inventory_side(), Direction::North);
        assert_eq!(reg.output_side(), Direction::South);
        assert!(reg.can_connect_tube(Direction::South));
        assert!(reg.can_connect_tube(Direction::North));

        let slots = slot_positions();
        assert_eq!(slots.len(), 27);
        assert_eq!(slots[0], [8.0, 18.0]);
        assert_eq!(slots[9], [80.0, 18.0]);
        assert_eq!(slots[18], [152.0, 18.0]);
    }

    #[test]
    fn test_regulator_inventory_stocking() {
        let mut reg = TileRegulator::new(Direction::North);
        let cat = PlainCatalog;

        // Target: maintain 16 coal in adjacent chest
        reg.contents[0] = Some(ItemStack::new("minecraft:coal", 16));
        // Buffer currently holds 16 coal
        reg.contents[9] = Some(ItemStack::new("minecraft:coal", 16));

        // Adjacent chest is empty
        let mut adjacent_inv = vec![None; 9];
        let range = SlotRange::all(9);

        // Sync stocking: should move 16 coal into adjacent chest
        assert!(reg.sync_stocking(&mut adjacent_inv, range, &cat));
        assert_eq!(adjacent_inv[0].as_ref().unwrap().count, 16);
        assert_eq!(reg.contents[9], None);
        assert_eq!(reg.power_drawn, (25 * 16) as f64);
    }

    #[test]
    fn test_regulator_chunk_persistence() {
        let mut original = TileRegulator::new(Direction::West);
        original.color = 7;
        original.mode = 2;
        original.voltage = 105.0;
        original.contents[0] = Some(ItemStack::new("minecraft:wheat", 32));
        original.contents[10] = Some(ItemStack::new("minecraft:bread", 8));

        let nbt = original.to_nbt();
        let restored = TileRegulator::from_nbt(&nbt);

        assert_eq!(restored.rotation, Direction::West);
        assert_eq!(restored.color, 7);
        assert_eq!(restored.mode, 2);
        assert_eq!(restored.voltage, 105.0);
        assert_eq!(
            restored.contents[0],
            Some(ItemStack::new("minecraft:wheat", 32))
        );
        assert_eq!(
            restored.contents[10],
            Some(ItemStack::new("minecraft:bread", 8))
        );
    }

    #[test]
    fn test_regulator_packet_and_frame_support() {
        let mut reg = TileRegulator::new(Direction::North);
        reg.flags.active = true;
        reg.flags.powered = true;

        let pkt = reg.write_tile_packet(10, 64, -20);
        assert_eq!(pkt.sub_id, SUBID_TILE_MACHINE);
        assert_eq!(pkt.x, 10);
        assert_eq!(pkt.y, 64);
        assert_eq!(pkt.z, -20);

        let mut restored = TileRegulator::new(Direction::Down);
        restored.handle_tile_packet(&pkt).unwrap();
        assert_eq!(restored.rotation, Direction::North);
        assert!(restored.flags.active);
        assert!(restored.flags.powered);

        let frame_pkt = reg.frame_packet();
        let mut frame_restored = TileRegulator::new(Direction::Down);
        frame_restored.handle_frame_packet(&frame_pkt).unwrap();
        assert_eq!(frame_restored.rotation, Direction::North);
        assert!(frame_restored.flags.active);
    }
}
