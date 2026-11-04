//! Transposer (`redpower_2:transposer`): port of `TileTranspose`.
//!
//! Mirrors:
//! - Tile entity registration name `"RPTranspose"`, `BlockMachine` metadata 2.
//! - Extended ID 2.
//! - Item extraction and loose entity vacuuming on redstone pulse into rear tube.
//! - Bounced item re-entry into internal buffer (`TubeBuffer`).
//! - Packet synchronization (Packet 211 `subId = 7`, 2-byte payload `[rotation, ps]`).
//! - Frame structural movement packet (`IFrameSupport`).
//! - IRotatable rotation mechanics (6 directions).

use super::base::{MachineFlags, SUBID_TILE_MACHINE, FrameSupport, Rotatable, TubeConnectable};
use super::breaker::{BufferedItem, TubeBuffer, TUBE_STATE_BOUNCE};
use crate::core::foundation::{Packet211TileDesc, PacketError};
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
/// Ticks between pulse and settle/drain tick.
pub const TRANSPOSER_TICK_DELAY: u32 = 5;
/// Tube state for incoming items from the front face.
pub const TUBE_STATE_ENTER: i32 = 1;

/// Transposer Tile Entity (`TileTranspose`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileTranspose {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub buffer: TubeBuffer,
    pub scheduled_tick: Option<u32>,
}

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

    /// `getTubeConnectableSides`: both front and rear ends of the axis.
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
    pub fn tube_item_enter(&mut self, side: Direction, state: i32, item: BufferedItem) -> bool {
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
            true
        } else {
            false
        }
    }

    /// Redstone pulse handler (`onBlockNeighborChange`). Returns `true` if an extraction/vacuum was triggered.
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
            self.flags.powered = false;
        }
        false
    }

    /// Drain items from buffer into the rear tube or inventory.
    pub fn drain_buffer<F>(&mut self, mut offer_item: F) -> bool
    where
        F: FnMut(&BufferedItem) -> bool,
    {
        while let Some(item) = self.buffer.last().cloned() {
            if offer_item(&item) {
                self.buffer.pop();
            } else {
                return false;
            }
        }
        true
    }

    /// Tick handler (`onTileTick`).
    pub fn on_tile_tick<F>(&mut self, offer_item: F)
    where
        F: FnMut(&BufferedItem) -> bool,
    {
        self.drain_buffer(offer_item);
        if self.buffer.is_empty() {
            self.flags.active = false;
            self.scheduled_tick = None;
        } else {
            self.scheduled_tick = Some(TRANSPOSER_TICK_DELAY);
        }
    }

    // ------------------------------------------------------------------------
    // Packet Synchronization (Packet 211, subId = 7)
    // ------------------------------------------------------------------------

    pub fn write_to_packet(&self) -> [u8; 2] {
        [self.rotation as u8, self.flags.to_bits()]
    }

    pub fn read_from_packet(&mut self, data: &[u8]) -> Result<(), PacketError> {
        if data.len() < 2 {
            return Err(PacketError::Truncated);
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

    pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
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
        assert_eq!(restored.buffer.last().unwrap().item.item_id, "minecraft:diamond");
        assert_eq!(restored.buffer.last().unwrap().item.count, 3);
    }
}
