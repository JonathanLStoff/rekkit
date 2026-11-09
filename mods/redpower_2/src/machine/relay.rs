//! Relay (`redpower_2:relay`): port of `TileRelay` and `TileEjectBase`.
//!
//! Mirrors:
//! - Tile entity registration name `"RPRelay"`, `BlockMachine` metadata 15.
//! - Extended ID 15.
//! - 9-slot buffer inventory (`contents[9]`, 3x3 GUI slots).
//! - Automatic ejection of one full stack every 10 ticks into connected pneumatic tube.
//! - Tube connection only on the rear face (`Rotation`).
//! - Sided inventory: rear face accepts 0 items, all other 5 faces accept items.
//! - Bounced item re-entry into internal buffer (`TubeBuffer`).
//! - Packet synchronization (Packet 211 `subId = 7`, 2-byte payload `[rotation, ps]`).
//! - Frame structural movement packet (`IFrameSupport`).
//! - Chunk reload persistence of all 9 slots and buffer (Level 5 contract).

use super::base::{FrameSupport, MachineFlags, Rotatable, TubeConnectable, SUBID_TILE_MACHINE};
use super::breaker::{BufferedItem, TubeBuffer, BLOCKED_RETRY_DELAY, TUBE_STATE_BOUNCE};
use crate::core::foundation::{Packet211TileDesc, PacketError};
use crate::ItemStack;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tile entity registration name in `GameRegistry`.
pub const RELAY_TE_NAME: &str = "RPRelay";
/// Extended ID (`getExtendedID`).
pub const RELAY_EXTENDED_ID: u8 = 15;
/// Block ID.
pub const RELAY_BLOCK_ID: &str = "redpower_2:relay";
/// BlockMachine metadata index.
pub const RELAY_METADATA: u8 = 15;
/// Number of inventory slots (`k_()`).
pub const RELAY_SLOTS: usize = 9;
/// Per-slot stack limit (`c()`).
pub const RELAY_STACK_LIMIT: u32 = 64;
/// Ticks between extraction and settle tick.
pub const RELAY_ACTIVE_DELAY: u32 = 5;
/// Automatic pulse / tick interval in game ticks.
pub const RELAY_INTERVAL_TICKS: u32 = 10;
/// Ticks before retrying when the buffer is plugged.
pub const RELAY_BLOCKED_RETRY_DELAY: u32 = BLOCKED_RETRY_DELAY;
/// GUI window dimensions `[width, height]`.
pub const GUI_SIZE: [u32; 2] = [176, 166];
/// GUI title string.
pub const GUI_TITLE: &str = "Relay";
/// 3x3 inventory grid origin in GUI coordinates.
pub const RELAY_GRID_ORIGIN: [f32; 2] = [62.0, 17.0];
/// Player inventory origin in GUI coordinates.
pub const RELAY_INVENTORY_ORIGIN: [f32; 2] = [8.0, 84.0];

/// Slot positions for `TileRelay` GUI (3x3 grid at `[62, 17]` with 18px pitch).
pub fn slot_positions() -> Vec<[f32; 2]> {
    let mut v = Vec::with_capacity(RELAY_SLOTS);
    for row in 0..3 {
        for col in 0..3 {
            v.push([
                RELAY_GRID_ORIGIN[0] + col as f32 * 18.0,
                RELAY_GRID_ORIGIN[1] + row as f32 * 18.0,
            ]);
        }
    }
    v
}

/// Relay Tile Entity (`TileRelay`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileRelay {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub buffer: TubeBuffer,
    pub contents: [Option<ItemStack>; RELAY_SLOTS],
    pub scheduled_tick: Option<u32>,
}

impl Default for TileRelay {
    fn default() -> Self {
        Self::new(Direction::Down)
    }
}

impl TileRelay {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            buffer: TubeBuffer::default(),
            contents: Default::default(),
            scheduled_tick: None,
        }
    }

    /// Rear face where tube connects and items are ejected (`Rotation`).
    pub fn rear(&self) -> Direction {
        self.rotation
    }

    /// Front face (`Rotation ^ 1`).
    pub fn front(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Tubes connect only on the rear face (`getTubeConnectableSides`: `1 << Rotation`).
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rear()
    }

    /// `tubeItemCanEnter`: only bounced items through the rear face (`state == 2`).
    pub fn tube_item_can_enter(&self, side: Direction, state: i32) -> bool {
        side == self.rear() && state == TUBE_STATE_BOUNCE
    }

    /// `tubeWeight`.
    pub fn tube_weight(&self, side: Direction, state: i32) -> usize {
        if self.tube_item_can_enter(side, state) {
            self.buffer.size()
        } else {
            0
        }
    }

    /// `tubeItemEnter`: a bounced item re-enters through the rear face.
    pub fn tube_item_enter(&mut self, side: Direction, state: i32, item: BufferedItem) -> bool {
        if self.tube_item_can_enter(side, state) {
            self.buffer.add_bounce(item);
            self.flags.active = true;
            self.scheduled_tick = Some(RELAY_ACTIVE_DELAY);
            true
        } else {
            false
        }
    }

    /// `getSizeInventorySide`: 0 on rear face, 9 on all other 5 faces.
    pub fn inventory_size_for_side(&self, side: Direction) -> usize {
        if side == self.rear() {
            0
        } else {
            RELAY_SLOTS
        }
    }

    /// `getStartInventorySide`: always 0.
    pub fn start_inventory_side(&self, _side: Direction) -> usize {
        0
    }

    /// Get slot content.
    pub fn get(&self, slot: usize) -> Option<&ItemStack> {
        self.contents.get(slot)?.as_ref()
    }

    /// Set slot content, clamping to stack limit.
    pub fn set(&mut self, slot: usize, stack: Option<ItemStack>) {
        if let Some(cell) = self.contents.get_mut(slot) {
            *cell = stack.map(|mut s| {
                s.count = s.count.min(RELAY_STACK_LIMIT);
                s
            });
        }
    }

    /// Take up to `n` items from a slot (`a(int, int)`).
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

    /// Insert an item stack into the relay inventory. Returns items that did not fit.
    pub fn insert_item(&mut self, mut item: ItemStack) -> Option<ItemStack> {
        if item.count == 0 {
            return None;
        }
        // First try to merge into matching existing slots
        for slot in &mut self.contents {
            if let Some(existing) = slot.as_mut() {
                if existing.item_id == item.item_id && existing.damage == item.damage {
                    let space = RELAY_STACK_LIMIT.saturating_sub(existing.count);
                    let to_add = space.min(item.count);
                    existing.count += to_add;
                    item.count -= to_add;
                    if item.count == 0 {
                        return None;
                    }
                }
            }
        }
        // Then place into empty slots
        for slot in &mut self.contents {
            if slot.is_none() {
                let to_add = RELAY_STACK_LIMIT.min(item.count);
                let mut placed = item.clone();
                placed.count = to_add;
                *slot = Some(placed);
                item.count -= to_add;
                if item.count == 0 {
                    return None;
                }
            }
        }
        Some(item)
    }

    /// Drop all slot contents and buffer items on block removal (`onBlockRemoval`).
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

    /// Drain items from buffer into the connected rear tube.
    /// If an item is refused, the buffer is marked plugged.
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

    /// `handleExtract`: takes the entire stack from the first non-empty slot into the buffer, then drains.
    pub fn handle_extract(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        for n in 0..RELAY_SLOTS {
            if self.contents[n].as_ref().is_some_and(|s| s.count > 0) {
                if let Some(stack) = self.contents[n].take() {
                    self.buffer.add_new(stack);
                    self.drain_buffer(handle_item);
                    return true;
                }
            }
        }
        false
    }

    /// Entity update tick (`g()` / `updateEntity`). Ensures a tick is scheduled every 10 ticks.
    pub fn update_entity(&mut self) {
        if self.scheduled_tick.is_none() {
            self.scheduled_tick = Some(RELAY_INTERVAL_TICKS);
        }
    }

    /// Tile tick handler (`onTileTick`).
    /// Returns `true` when the active flag changed (client block update needed).
    pub fn on_tile_tick(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        self.scheduled_tick = None;
        let mut update_needed = false;

        // Super: TileEjectBase.onTileTick()
        if !self.buffer.is_empty() {
            self.drain_buffer(handle_item);
            if !self.buffer.is_empty() {
                self.scheduled_tick = Some(RELAY_BLOCKED_RETRY_DELAY);
            } else {
                self.scheduled_tick = Some(RELAY_ACTIVE_DELAY);
            }
        } else if !self.flags.powered && self.flags.active {
            self.flags.active = false;
            update_needed = true;
        }

        // TileRelay.onTileTick()
        if !self.flags.active && self.handle_extract(handle_item) {
            self.flags.active = true;
            self.scheduled_tick = Some(RELAY_ACTIVE_DELAY);
            update_needed = true;
        }

        update_needed
    }

    /// Full 1-tick simulation step.
    /// Decrements scheduled tick, fires `on_tile_tick` when it elapses, and maintains the 10-tick schedule.
    pub fn tick(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        let mut triggered = false;
        if let Some(ticks) = self.scheduled_tick.as_mut() {
            if *ticks > 0 {
                *ticks -= 1;
            }
            if *ticks == 0 {
                triggered = self.on_tile_tick(handle_item);
            }
        }
        self.update_entity();
        triggered
    }

    /// Redstone neighbor change.
    pub fn on_neighbor_change(&mut self, powered: bool) {
        self.flags.powered = powered;
    }

    // ------------------------------------------------------------------------
    // Packet Synchronization (Packet 211, subId = 7)
    // ------------------------------------------------------------------------

    pub fn write_to_packet(&self) -> [u8; 2] {
        [self.rotation as u8, self.flags.to_bits()]
    }

    pub fn read_from_packet(&mut self, data: &[u8]) -> Result<(), PacketError> {
        if data.len() < 2 {
            return Err(PacketError::NotEnoughData);
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

    pub fn handle_tile_packet(&mut self, pkt: &Packet211TileDesc) -> Result<(), PacketError> {
        if pkt.sub_id != SUBID_TILE_MACHINE {
            return Err(PacketError::BadVlc);
        }
        self.read_from_packet(&pkt.body)
    }

    // ------------------------------------------------------------------------
    // NBT Persistence (Level 5 contract: keeps inventory across chunk reload)
    // ------------------------------------------------------------------------

    pub fn write_nbt(&mut self, nbt: &mut HashMap<String, String>) {
        nbt.insert("rot".into(), (self.rotation as u8).to_string());
        nbt.insert("ps".into(), self.flags.to_bits().to_string());
        for (i, item) in self.contents.iter().enumerate() {
            if let Some(stack) = item {
                nbt.insert(
                    format!("slot_{i}"),
                    format!("{}:{}*{}", stack.item_id, stack.damage, stack.count),
                );
            }
        }
        let buf = self.buffer.take_all_for_drop();
        if !buf.is_empty() {
            let encoded = buf
                .iter()
                .map(|s| format!("{}:{}*{}", s.item_id, s.damage, s.count))
                .collect::<Vec<_>>()
                .join(";");
            nbt.insert("buffer".into(), encoded);
        }
    }

    pub fn read_nbt(&mut self, nbt: &HashMap<String, String>) {
        if let Some(r) = nbt.get("rot").and_then(|v| v.parse::<u8>().ok()) {
            self.rotation = match r {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                5 => Direction::East,
                _ => Direction::Down,
            };
        }
        if let Some(ps) = nbt.get("ps").and_then(|v| v.parse::<u8>().ok()) {
            self.flags = MachineFlags::from_bits(ps);
        }
        for i in 0..RELAY_SLOTS {
            if let Some(slot_str) = nbt.get(&format!("slot_{i}")) {
                if let Some((id_dmg, cnt)) = slot_str.rsplit_once('*') {
                    if let Ok(count) = cnt.parse::<u32>() {
                        let (id, dmg) = id_dmg
                            .rsplit_once(':')
                            .map(|(a, b)| (a, b.parse::<u32>().unwrap_or(0)))
                            .unwrap_or((id_dmg, 0));
                        let mut st = ItemStack::new(id, count);
                        st.damage = dmg as u16;
                        self.contents[i] = Some(st);
                    }
                }
            } else {
                self.contents[i] = None;
            }
        }
        if let Some(buf_str) = nbt.get("buffer") {
            for entry in buf_str.split(';') {
                if let Some((id_dmg, cnt)) = entry.rsplit_once('*') {
                    if let Ok(count) = cnt.parse::<u32>() {
                        let (id, dmg) = id_dmg
                            .rsplit_once(':')
                            .map(|(a, b)| (a, b.parse::<u32>().unwrap_or(0)))
                            .unwrap_or((id_dmg, 0));
                        let mut st = ItemStack::new(id, count);
                        st.damage = dmg as u16;
                        self.buffer.add_new(st);
                    }
                }
            }
        }
    }
}

impl Rotatable for TileRelay {
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

impl FrameSupport for TileRelay {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileRelay frame packet header".into());
        }
        self.read_from_packet(&data[1..3])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

impl TubeConnectable for TileRelay {
    fn can_connect_tube(&self, side: Direction) -> bool {
        self.can_connect_tube(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_gui_slots_match_contract() {
        let slots = slot_positions();
        assert_eq!(slots.len(), 9);
        let expected: Vec<[f32; 2]> = vec![
            [62.0, 17.0],
            [80.0, 17.0],
            [98.0, 17.0],
            [62.0, 35.0],
            [80.0, 35.0],
            [98.0, 35.0],
            [62.0, 53.0],
            [80.0, 53.0],
            [98.0, 53.0],
        ];
        assert_eq!(slots, expected);
    }

    #[test]
    fn test_relay_connectivity_only_rear() {
        let r = TileRelay::new(Direction::North);
        assert_eq!(r.rear(), Direction::North);
        assert!(r.can_connect_tube(Direction::North));
        assert!(!r.can_connect_tube(Direction::South));
        assert!(!r.can_connect_tube(Direction::East));
        assert!(!r.can_connect_tube(Direction::West));
        assert!(!r.can_connect_tube(Direction::Up));
        assert!(!r.can_connect_tube(Direction::Down));

        // Sided inventory: rear is 0, sides are 9
        assert_eq!(r.inventory_size_for_side(Direction::North), 0);
        assert_eq!(r.inventory_size_for_side(Direction::South), 9);
        assert_eq!(r.inventory_size_for_side(Direction::East), 9);
    }

    #[test]
    fn test_relay_auto_ejection_every_10_ticks() {
        let mut r = TileRelay::new(Direction::North);
        r.set(0, Some(ItemStack::new("minecraft:cobblestone", 64)));
        r.set(1, Some(ItemStack::new("minecraft:iron_ingot", 16)));

        let mut ejected = Vec::new();

        // Initial update_entity schedules 10 ticks
        r.update_entity();
        assert_eq!(r.scheduled_tick, Some(10));

        // Ticks 1 to 9: nothing ejected
        for _ in 0..9 {
            r.tick(&mut |b| {
                ejected.push((b.item.item_id.clone(), b.item.count));
                true
            });
        }
        assert!(ejected.is_empty());
        assert_eq!(r.get(0).unwrap().count, 64);

        // Tick 10: 10th tick fires, full stack from slot 0 is ejected!
        r.tick(&mut |b| {
            ejected.push((b.item.item_id.clone(), b.item.count));
            true
        });
        assert_eq!(ejected.len(), 1);
        assert_eq!(ejected[0], ("minecraft:cobblestone".to_string(), 64));
        assert!(r.get(0).is_none());
        assert!(r.flags.active);

        // Active delay: next extraction happens 5 ticks later
        for _ in 0..4 {
            r.tick(&mut |b| {
                ejected.push((b.item.item_id.clone(), b.item.count));
                true
            });
        }
        assert_eq!(ejected.len(), 1);

        // 5th tick: slot 1 is ejected!
        r.tick(&mut |b| {
            ejected.push((b.item.item_id.clone(), b.item.count));
            true
        });
        assert_eq!(ejected.len(), 2);
        assert_eq!(ejected[1], ("minecraft:iron_ingot".to_string(), 16));
        assert!(r.get(1).is_none());
    }

    #[test]
    fn test_relay_nbt_chunk_reload_preserves_inventory() {
        let mut r = TileRelay::new(Direction::South);
        r.set(0, Some(ItemStack::new("minecraft:diamond", 5)));
        r.set(4, Some(ItemStack::new("minecraft:gold_ingot", 12)));
        r.set(8, Some(ItemStack::new("minecraft:redstone", 64)));

        let mut nbt = HashMap::new();
        r.write_nbt(&mut nbt);

        let mut restored = TileRelay::new(Direction::Down);
        restored.read_nbt(&nbt);

        assert_eq!(restored.rotation, Direction::South);
        assert_eq!(restored.get(0).unwrap().item_id, "minecraft:diamond");
        assert_eq!(restored.get(0).unwrap().count, 5);
        assert_eq!(restored.get(4).unwrap().item_id, "minecraft:gold_ingot");
        assert_eq!(restored.get(4).unwrap().count, 12);
        assert_eq!(restored.get(8).unwrap().item_id, "minecraft:redstone");
        assert_eq!(restored.get(8).unwrap().count, 64);
        assert!(restored.get(1).is_none());
    }

    #[test]
    fn test_relay_bounce_entry() {
        let mut r = TileRelay::new(Direction::North);
        assert!(r.tube_item_can_enter(Direction::North, TUBE_STATE_BOUNCE));
        assert!(!r.tube_item_can_enter(Direction::South, TUBE_STATE_BOUNCE));

        let entered = r.tube_item_enter(
            Direction::North,
            TUBE_STATE_BOUNCE,
            BufferedItem {
                item: ItemStack::new("minecraft:emerald", 2),
                color: None,
            },
        );
        assert!(entered);
        assert_eq!(r.buffer.size(), 1);
        assert!(r.flags.active);
    }
}
