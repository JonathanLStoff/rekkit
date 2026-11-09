//! Transposer (`redpower_2:transposer`): port of `TileTranspose`.
//!
//! Mirrors:
//! - Tile entity registration name `"RPTranspose"`, `BlockMachine` metadata 2.
//! - Extended ID 2.
//! - Loose entity vacuuming or adjacent inventory item extraction on redstone pulse into rear tube.
//! - Sided tube connectivity: front (`Rotation ^ 1`) and rear (`Rotation`).
//! - Bounced item re-entry into internal buffer (`TubeBuffer`).
//! - Packet synchronization (Packet 211 `subId = 7`, 2-byte payload `[rotation, ps]`).
//! - Frame structural movement packet (`IFrameSupport`).
//! - IRotatable rotation mechanics (6 directions).

use super::base::{FrameSupport, MachineFlags, Rotatable, TubeConnectable, SUBID_TILE_MACHINE};
use super::breaker::{BufferedItem, TubeBuffer, BLOCKED_RETRY_DELAY, TUBE_STATE_BOUNCE};
use crate::core::foundation::{Packet211TileDesc, PacketError};
use crate::core::machine_lib::SlotRange;
use crate::ItemStack;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tile entity registration name in `GameRegistry`.
pub const TRANSPOSER_TE_NAME: &str = "RPTranspose";
/// Extended ID (`getExtendedID`).
pub const TRANSPOSER_EXTENDED_ID: u8 = 2;
/// Block ID.
pub const TRANSPOSER_BLOCK_ID: &str = "redpower_2:transposer";
/// BlockMachine metadata index.
pub const TRANSPOSER_METADATA: u8 = 2;
/// Ticks between pulse/entry and settle/drain tick.
pub const TRANSPOSER_TICK_DELAY: u32 = 5;
/// Ticks before retrying when the buffer is plugged.
pub const TRANSPOSER_RETRY_DELAY: u32 = BLOCKED_RETRY_DELAY;
/// Tube state for incoming items from the front face (`state == 1`).
pub const TUBE_STATE_ENTER: i32 = 1;

/// Transposer Tile Entity (`TileTranspose`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileTranspose {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub buffer: TubeBuffer,
    pub scheduled_tick: Option<u32>,
}

pub type TileTransposer = TileTranspose;

impl Default for TileTranspose {
    fn default() -> Self {
        Self::new(Direction::Down)
    }
}

impl TileTranspose {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            buffer: TubeBuffer::default(),
            scheduled_tick: None,
        }
    }

    /// Front face where vacuuming/extraction happens (`Rotation ^ 1`).
    pub fn front(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Rear face where tube connects and items are injected (`Rotation`).
    pub fn rear(&self) -> Direction {
        self.rotation
    }

    /// Front face where items can enter as well.
    pub fn input_side(&self) -> Direction {
        self.front()
    }

    /// `getTubeConnectableSides`: both front and rear ends of the axis (`3 << (Rotation & -2)`).
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rear() || side == self.front()
    }

    /// `tubeItemCanEnter`.
    pub fn tube_item_can_enter(&self, side: Direction, state: i32) -> bool {
        if side == self.rear() && state == TUBE_STATE_BOUNCE {
            true
        } else if side == self.front() && state == TUBE_STATE_ENTER {
            self.buffer.is_empty() && !self.flags.powered
        } else {
            false
        }
    }

    /// `tubeWeight`.
    pub fn tube_weight(&self, side: Direction, state: i32) -> usize {
        if side == self.rear() && state == TUBE_STATE_BOUNCE {
            self.buffer.size()
        } else {
            0
        }
    }

    /// `tubeItemEnter`: adds bounced items or front-entering items to buffer.
    pub fn tube_item_enter(
        &mut self,
        side: Direction,
        state: i32,
        item: BufferedItem,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        if side == self.rear() && state == TUBE_STATE_BOUNCE {
            self.buffer.add_bounce(item);
            self.flags.active = true;
            self.scheduled_tick = Some(TRANSPOSER_TICK_DELAY);
            true
        } else if side == self.front() && state == TUBE_STATE_ENTER {
            if self.flags.powered || !self.buffer.is_empty() {
                return false;
            }
            self.buffer.add(item);
            self.flags.active = true;
            self.scheduled_tick = Some(TRANSPOSER_TICK_DELAY);
            self.drain_buffer(handle_item);
            true
        } else {
            false
        }
    }

    /// `canSuck`: checks if the block in front can have entities vacuumed through it.
    /// Returns false if solid or if it has an inventory / tube connection.
    pub fn can_suck(is_solid_on_side: bool, has_inventory_or_tube: bool) -> bool {
        !is_solid_on_side && !has_inventory_or_tube
    }

    /// `getSizeBox`: computes the bounding box in front or behind the machine.
    /// `bw`: half-width, `bf`: forward extent, `bb`: backward extent.
    /// Origin is block minimum coordinate `(x, y, z)`.
    pub fn get_size_box(
        rotation: Direction,
        origin: (f64, f64, f64),
        bw: f64,
        bf: f64,
        bb: f64,
    ) -> [f64; 6] {
        let fx = origin.0 + 0.5;
        let fy = origin.1 + 0.5;
        let fz = origin.2 + 0.5;
        match rotation.opposite() {
            Direction::Down => [
                fx - bw,
                origin.1 - bb,
                fz - bw,
                fx + bw,
                origin.1 + bf,
                fz + bw,
            ],
            Direction::Up => [
                fx - bw,
                origin.1 + 1.0 - bf,
                fz - bw,
                fx + bw,
                origin.1 + 1.0 + bb,
                fz + bw,
            ],
            Direction::North => [
                fx - bw,
                fy - bw,
                origin.2 - bb,
                fx + bw,
                fy + bw,
                origin.2 + bf,
            ],
            Direction::South => [
                fx - bw,
                fy - bw,
                origin.2 + 1.0 - bf,
                fx + bw,
                fy + bw,
                origin.2 + 1.0 + bb,
            ],
            Direction::West => [
                origin.0 - bb,
                fy - bw,
                fz - bw,
                origin.0 + bf,
                fy + bw,
                fz + bw,
            ],
            Direction::East => [
                origin.0 + 1.0 - bf,
                fy - bw,
                fz - bw,
                origin.0 + 1.0 + bb,
                fy + bw,
                fz + bw,
            ],
        }
    }

    /// Vacuum bounding box for suction pulse: `getSizeBox(1.55, 3.05, -0.95)`.
    pub fn vacuum_bounding_box(rotation: Direction, origin: (f64, f64, f64)) -> [f64; 6] {
        Self::get_size_box(rotation, origin, 1.55, 3.05, -0.95)
    }

    /// Passive collision bounding box for entity contact: `getSizeBox(0.55, 1.05, -0.95)`.
    pub fn collision_bounding_box(rotation: Direction, origin: (f64, f64, f64)) -> [f64; 6] {
        Self::get_size_box(rotation, origin, 0.55, 1.05, -0.95)
    }

    /// Minecart chest detection box behind rear face: `getSizeBox(0.8, 0.05, 1.05)`.
    pub fn minecart_bounding_box(rotation: Direction, origin: (f64, f64, f64)) -> [f64; 6] {
        Self::get_size_box(rotation, origin, 0.8, 0.05, 1.05)
    }

    /// Sucks a single dropped item stack into the buffer.
    pub fn suck_item(
        &mut self,
        item: ItemStack,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        if item.count == 0 {
            return false;
        }
        self.buffer.add_new(item);
        if !self.flags.active {
            self.flags.active = true;
        }
        self.drain_buffer(handle_item);
        self.scheduled_tick = Some(TRANSPOSER_TICK_DELAY);
        true
    }

    /// Sucks multiple dropped entity stacks in front of the transposer.
    pub fn suck_entities(
        &mut self,
        items: impl IntoIterator<Item = ItemStack>,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> usize {
        let mut count = 0;
        for item in items {
            if item.count > 0 {
                self.buffer.add_new(item);
                count += 1;
            }
        }
        if count > 0 {
            if !self.flags.active {
                self.flags.active = true;
            }
            self.drain_buffer(handle_item);
            self.scheduled_tick = Some(TRANSPOSER_TICK_DELAY);
        }
        count
    }

    /// Passive suction on entity collision with front face (`onEntityCollidedWithBlock`).
    pub fn on_entity_collide(
        &mut self,
        items: impl IntoIterator<Item = ItemStack>,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> usize {
        if !self.flags.powered && self.buffer.is_empty() {
            self.suck_entities(items, handle_item)
        } else {
            0
        }
    }

    /// Extract 1 item (`inv.a(n, 1)`) from the first non-empty slot in range into the buffer.
    pub fn handle_extract(
        &mut self,
        inv: &mut [Option<ItemStack>],
        range: SlotRange,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        let start = range.start.min(inv.len());
        let end = (range.start + range.len).min(inv.len());
        for slot in &mut inv[start..end] {
            if let Some(stack) = slot.as_mut() {
                if stack.count > 0 {
                    let extracted = if stack.count <= 1 {
                        slot.take().unwrap()
                    } else {
                        stack.count -= 1;
                        let mut s = stack.clone();
                        s.count = 1;
                        s
                    };
                    self.buffer.add_new(extracted);
                    self.drain_buffer(handle_item);
                    return true;
                }
            }
        }
        false
    }

    /// Extract a full stack from the first non-empty slot in range into the buffer.
    pub fn handle_extract_stack(
        &mut self,
        inv: &mut [Option<ItemStack>],
        range: SlotRange,
        handle_item: &mut dyn FnMut(&BufferedItem) -> bool,
    ) -> bool {
        let start = range.start.min(inv.len());
        let end = (range.start + range.len).min(inv.len());
        for slot in &mut inv[start..end] {
            if let Some(stack) = slot.as_mut() {
                if stack.count > 0 {
                    let extracted = slot.take().unwrap();
                    self.buffer.add_new(extracted);
                    self.drain_buffer(handle_item);
                    return true;
                }
            }
        }
        false
    }

    /// Redstone pulse handler (`onBlockNeighborChange`).
    /// Returns `true` if a rising edge occurred while inactive, indicating that suction
    /// or inventory extraction should be executed.
    pub fn on_neighbor_change(&mut self, powered: bool) -> bool {
        if powered {
            if !self.flags.powered {
                self.flags.powered = true;
                if !self.flags.active {
                    self.flags.active = true;
                    self.scheduled_tick = Some(TRANSPOSER_TICK_DELAY);
                    return true;
                }
            }
        } else {
            if self.flags.active && self.scheduled_tick.is_none() {
                self.scheduled_tick = Some(TRANSPOSER_TICK_DELAY);
            }
            self.flags.powered = false;
        }
        false
    }

    /// Drain items from buffer into the rear tube or inventory.
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

    /// Tick handler (`onTileTick`).
    /// Returns `true` when the active state changed (client block update needed).
    pub fn on_tile_tick(&mut self, handle_item: &mut dyn FnMut(&BufferedItem) -> bool) -> bool {
        self.scheduled_tick = None;
        if !self.buffer.is_empty() {
            self.drain_buffer(handle_item);
            self.scheduled_tick = Some(if self.buffer.is_empty() {
                TRANSPOSER_TICK_DELAY
            } else {
                TRANSPOSER_RETRY_DELAY
            });
            false
        } else if !self.flags.powered {
            self.flags.active = false;
            true
        } else {
            false
        }
    }

    /// Drop all items stored in buffer on block break.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        self.buffer.take_all_for_drop()
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
    // NBT Persistence
    // ------------------------------------------------------------------------

    pub fn write_nbt(&mut self, nbt: &mut HashMap<String, String>) {
        nbt.insert("rot".into(), (self.rotation as u8).to_string());
        nbt.insert("ps".into(), self.flags.to_bits().to_string());
        let buf = self.buffer.take_all_for_drop();
        if !buf.is_empty() {
            let encoded = buf
                .iter()
                .map(|s| format!("{}*{}", s.item_id, s.count))
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
        if let Some(buf_str) = nbt.get("buffer") {
            for entry in buf_str.split(';') {
                if let Some((id, cnt)) = entry.rsplit_once('*') {
                    if let Ok(c) = cnt.parse::<u32>() {
                        self.buffer.add_new(ItemStack::new(id, c));
                    }
                }
            }
        }
    }
}

impl Rotatable for TileTranspose {
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

impl FrameSupport for TileTranspose {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileTranspose frame packet header".into());
        }
        self.read_from_packet(&data[1..3])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

impl TubeConnectable for TileTranspose {
    fn can_connect_tube(&self, side: Direction) -> bool {
        self.can_connect_tube(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transposer_connectivity_and_directions() {
        let t = TileTranspose::new(Direction::North);
        assert_eq!(t.rear(), Direction::North);
        assert_eq!(t.front(), Direction::South);
        assert!(t.can_connect_tube(Direction::North));
        assert!(t.can_connect_tube(Direction::South));
        assert!(!t.can_connect_tube(Direction::East));
        assert!(!t.can_connect_tube(Direction::West));
    }

    #[test]
    fn test_transposer_suction_vacuum_ground_entity() {
        let mut t = TileTranspose::new(Direction::North);
        let mut routed = Vec::new();

        let triggered = t.on_neighbor_change(true);
        assert!(triggered);
        assert!(t.flags.active);
        assert!(t.flags.powered);

        // Vacuum a dropped item stack into rear tube
        let sucked = t.suck_item(ItemStack::new("minecraft:redstone", 4), &mut |b| {
            routed.push((b.item.item_id.clone(), b.item.count));
            true
        });
        assert!(sucked);
        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0], ("minecraft:redstone".to_string(), 4));
        assert!(t.buffer.is_empty());
    }

    #[test]
    fn test_transposer_extract_from_inventory() {
        let mut t = TileTranspose::new(Direction::North);
        let mut inv = vec![
            None,
            Some(ItemStack::new("minecraft:iron_ingot", 5)),
            Some(ItemStack::new("minecraft:gold_ingot", 2)),
        ];
        let mut routed = Vec::new();

        let triggered = t.on_neighbor_change(true);
        assert!(triggered);

        let extracted = t.handle_extract(&mut inv, SlotRange::all(3), &mut |b| {
            routed.push((b.item.item_id.clone(), b.item.count));
            true
        });
        assert!(extracted);
        assert_eq!(routed, vec![("minecraft:iron_ingot".to_string(), 1)]);
        assert_eq!(inv[1].as_ref().unwrap().count, 4);
    }

    #[test]
    fn test_transposer_extract_stack() {
        let mut t = TileTranspose::new(Direction::North);
        let mut inv = vec![Some(ItemStack::new("minecraft:coal", 16))];
        let mut routed = Vec::new();

        let extracted = t.handle_extract_stack(&mut inv, SlotRange::all(1), &mut |b| {
            routed.push((b.item.item_id.clone(), b.item.count));
            true
        });
        assert!(extracted);
        assert_eq!(routed, vec![("minecraft:coal".to_string(), 16)]);
        assert!(inv[0].is_none());
    }

    #[test]
    fn test_transposer_bounce_and_front_tube_entry() {
        let mut t = TileTranspose::new(Direction::North);
        assert!(t.tube_item_can_enter(Direction::North, TUBE_STATE_BOUNCE));
        assert!(t.tube_item_can_enter(Direction::South, TUBE_STATE_ENTER));
        assert!(!t.tube_item_can_enter(Direction::North, TUBE_STATE_ENTER));

        // Enter through front
        let mut routed = Vec::new();
        let entered = t.tube_item_enter(
            Direction::South,
            TUBE_STATE_ENTER,
            BufferedItem {
                item: ItemStack::new("minecraft:diamond", 1),
                color: None,
            },
            &mut |b| {
                routed.push(b.item.item_id.clone());
                true
            },
        );
        assert!(entered);
        assert_eq!(routed, vec!["minecraft:diamond".to_string()]);
    }

    #[test]
    fn test_transposer_packet_and_frame_roundtrip() {
        let mut t = TileTranspose::new(Direction::East);
        t.flags.active = true;
        t.flags.charged = true;

        let pkt = t.write_tile_packet(5, 64, 12);
        assert_eq!(pkt.sub_id, SUBID_TILE_MACHINE);
        assert_eq!(pkt.x, 5);
        assert_eq!(pkt.y, 64);
        assert_eq!(pkt.z, 12);
        assert_eq!(pkt.body.len(), 2);

        let mut restored = TileTranspose::new(Direction::Down);
        assert!(restored.handle_tile_packet(&pkt).is_ok());
        assert_eq!(restored.rotation, Direction::East);
        assert!(restored.flags.active);
        assert!(restored.flags.charged);
        assert!(!restored.flags.powered);

        // Frame packet roundtrip
        let frame_pkt = t.frame_packet();
        let mut frame_restored = TileTranspose::new(Direction::Down);
        assert!(frame_restored.handle_frame_packet(&frame_pkt).is_ok());
        assert_eq!(frame_restored.rotation, Direction::East);
        assert_eq!(frame_restored.flags, t.flags);
    }

    #[test]
    fn test_transposer_nbt_roundtrip() {
        let mut t = TileTranspose::new(Direction::South);
        t.flags.powered = true;
        t.buffer.add_new(ItemStack::new("minecraft:diamond", 3));

        let mut nbt = HashMap::new();
        t.write_nbt(&mut nbt);

        let mut restored = TileTranspose::new(Direction::Down);
        restored.read_nbt(&nbt);
        assert_eq!(restored.rotation, Direction::South);
        assert!(restored.flags.powered);
        assert_eq!(restored.buffer.size(), 1);
        assert_eq!(
            restored.buffer.last().unwrap().item.item_id,
            "minecraft:diamond"
        );
        assert_eq!(restored.buffer.last().unwrap().item.count, 3);
    }
}
