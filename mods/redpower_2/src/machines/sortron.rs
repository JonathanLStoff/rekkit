//! Sortron (`redpower_2:sortron`): port of `TileSortron`.
//!
//! Features:
//! - Redbus memory-mapped I/O peripheral for 6502 Forth control (`rbaddr`, `rbRead`, `rbWrite`).
//! - 14 MMIO registers:
//!   - 0: Command (0=idle/done, 1=query inv size, 2=query slot, 3=extract, 4=route tube item, 255=error).
//!   - 1: Item quantity (`itemQty`).
//!   - 2..3: Slot index (16-bit `itemSlot`).
//!   - 4..7: Item type hash (32-bit `itemType`).
//!   - 8..9: Item damage (`itemDamage`).
//!   - 10..11: Item max damage (`itemDamageMax`).
//!   - 12: Outgoing color tag (`itemColor`, 0..=16).
//!   - 13: Incoming filter color (`itemInColor`, 0..=16).
//! - Blutricity consumption: requires minimum 60V, draws 50 W-t per item.
//! - Sided pneumatic tube routing: input on front face, output on rear face.
//! - Chunk reload persistence for all registers, buffer, and Blutricity state.

use crate::core::foundation::{Packet211TileDesc, PacketError};
use crate::core::machine_lib::{ItemCatalog, Slots};
use crate::machine::base::{
    FrameSupport, MachineFlags, Rotatable, TubeConnectable, SUBID_TILE_MACHINE,
};
use crate::machine::breaker::{BufferedItem, TubeBuffer, TUBE_STATE_BOUNCE};
use crate::{DyeColor, ItemStack};
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const SORTRON_BLOCK_ID: &str = "redpower_2:sortron";
/// Tile entity name in `GameRegistry`.
pub const SORTRON_TE_NAME: &str = "RPSortron";
/// Metadata on `BlockMachine2`.
pub const SORTRON_METADATA: u8 = 0;
/// Extended ID index.
pub const SORTRON_EXTENDED_ID: u8 = 0;
/// Default Redbus peripheral address.
pub const DEFAULT_RBADDR: u8 = 4;
/// Minimum operating voltage in Volts.
pub const SORTRON_MIN_VOLTAGE: f64 = 60.0;
/// Power drawn per extracted/routed item (W-t / Blutricity units).
pub const SORTRON_POWER_PER_ITEM: u32 = 50;
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

/// Calculate 32-bit item hash mirroring Java `TileSortron.hashItem`.
pub fn hash_item(item: &ItemStack) -> i32 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&item.item_id, &mut h);
    let mut val = (std::hash::Hasher::finish(&h) & 0x7FFFFFFF) as i32;
    if item.damage != 0 {
        val ^= item.damage as i32;
    }
    val
}

/// `TileSortron` state machine and Redbus MMIO interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileSortron {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub rbaddr: u8,
    pub cmd_delay: u8,
    pub command: u8,
    pub item_slot: u16,
    pub item_type: i32,
    pub item_damage: u16,
    pub item_damage_max: u16,
    pub item_qty: u8,
    pub item_color: u8,
    pub item_in_color: u8,
    pub buffer: TubeBuffer,
    pub voltage: f64,
    pub power_drawn: f64,
    pub scheduled_tick: Option<u32>,
}

impl Default for TileSortron {
    fn default() -> Self {
        Self::new(Direction::North)
    }
}

impl TileSortron {
    /// Create a new Sortron facing `rotation`.
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            rbaddr: DEFAULT_RBADDR,
            cmd_delay: 0,
            command: 0,
            item_slot: 0,
            item_type: 0,
            item_damage: 0,
            item_damage_max: 0,
            item_qty: 0,
            item_color: 0,
            item_in_color: 0,
            buffer: TubeBuffer::default(),
            voltage: 100.0,
            power_drawn: 0.0,
            scheduled_tick: None,
        }
    }

    /// Front face pointing to connected inventory or tube entrance (`Rotation ^ 1`).
    pub fn input_side(&self) -> Direction {
        self.rotation.opposite()
    }

    /// Rear face where pneumatic tube connects (`Rotation`).
    pub fn output_side(&self) -> Direction {
        self.rotation
    }

    /// Tubes connect on both front and rear faces.
    pub fn can_connect_tube(&self, side: Direction) -> bool {
        side == self.rotation || side == self.input_side()
    }

    /// Draw power if voltage is above threshold.
    pub fn draw_power(&mut self, amount: u32) -> bool {
        if self.voltage < SORTRON_MIN_VOLTAGE {
            return false;
        }
        self.power_drawn += amount as f64;
        true
    }

    // ========================================================================
    // Redbus Memory-Mapped I/O Peripheral Interface
    // ========================================================================

    pub fn rb_get_addr(&self) -> u8 {
        self.rbaddr
    }

    pub fn rb_set_addr(&mut self, addr: u8) {
        self.rbaddr = addr;
    }

    /// Read an 8-bit Redbus register `0..14`.
    pub fn rb_read(&self, reg: u8) -> u8 {
        match reg {
            0 => self.command,
            1 => self.item_qty,
            2 => (self.item_slot & 0xFF) as u8,
            3 => ((self.item_slot >> 8) & 0xFF) as u8,
            4 => (self.item_type & 0xFF) as u8,
            5 => ((self.item_type >> 8) & 0xFF) as u8,
            6 => ((self.item_type >> 16) & 0xFF) as u8,
            7 => ((self.item_type >> 24) & 0xFF) as u8,
            8 => (self.item_damage & 0xFF) as u8,
            9 => ((self.item_damage >> 8) & 0xFF) as u8,
            10 => (self.item_damage_max & 0xFF) as u8,
            11 => ((self.item_damage_max >> 8) & 0xFF) as u8,
            12 => self.item_color,
            13 => self.item_in_color,
            _ => 0,
        }
    }

    /// Write an 8-bit Redbus register `0..14`.
    pub fn rb_write(&mut self, reg: u8, data: u8) {
        match reg {
            0 => {
                self.command = data;
                self.cmd_delay = 2;
            }
            1 => self.item_qty = data,
            2 => self.item_slot = (self.item_slot & 0xFF00) | (data as u16),
            3 => self.item_slot = (self.item_slot & 0x00FF) | ((data as u16) << 8),
            4 => self.item_type = (self.item_type & !0xFF) | (data as i32),
            5 => self.item_type = (self.item_type & !(0xFF << 8)) | ((data as i32) << 8),
            6 => self.item_type = (self.item_type & !(0xFF << 16)) | ((data as i32) << 16),
            7 => self.item_type = (self.item_type & !(0xFF << 24)) | ((data as i32) << 24),
            8 => self.item_damage = (self.item_damage & 0xFF00) | (data as u16),
            9 => self.item_damage = (self.item_damage & 0x00FF) | ((data as u16) << 8),
            10 => self.item_damage_max = (self.item_damage_max & 0xFF00) | (data as u16),
            11 => self.item_damage_max = (self.item_damage_max & 0x00FF) | ((data as u16) << 8),
            12 => self.item_color = data,
            13 => self.item_in_color = data,
            _ => {}
        }
    }

    /// Process active command against connected inventory.
    pub fn process_command(&mut self, connected_inv: Option<&mut Slots>, cat: &dyn ItemCatalog) {
        if self.voltage < SORTRON_MIN_VOLTAGE {
            self.cmd_delay = 20;
            return;
        }

        match self.command {
            0 => {}
            // Command 1: Query inventory slot count
            1 => {
                if let Some(inv) = connected_inv {
                    self.item_slot = inv.len() as u16;
                    self.command = 0;
                } else {
                    self.command = 255;
                }
            }
            // Command 2: Query slot contents
            2 => {
                let slot = self.item_slot as usize;
                if let Some(inv) = connected_inv {
                    if slot >= inv.len() {
                        self.command = 255;
                    } else if let Some(item) = &inv[slot] {
                        self.item_qty = item.count.min(255) as u8;
                        self.item_type = hash_item(item);
                        self.item_damage = item.damage;
                        self.item_damage_max = cat.meta(&item.item_id).max_damage as u16;
                        self.command = 0;
                    } else {
                        self.item_qty = 0;
                        self.item_type = 0;
                        self.item_damage = 0;
                        self.item_damage_max = 0;
                        self.command = 0;
                    }
                } else {
                    self.command = 255;
                }
            }
            // Command 3: Extract from slot
            3 => {
                let slot = self.item_slot as usize;
                let qty = self.item_qty;
                if let Some(inv) = connected_inv {
                    if slot >= inv.len() {
                        self.command = 255;
                    } else if let Some(item) = &mut inv[slot] {
                        let transfer = item.count.min(qty as u32);
                        if transfer > 0 {
                            let mut extracted = item.clone();
                            extracted.count = transfer;
                            item.count -= transfer;
                            if item.count == 0 {
                                inv[slot] = None;
                            }
                            let tag = color_to_tag(self.item_color);
                            self.buffer.add_new_color(extracted, tag);
                            self.draw_power(SORTRON_POWER_PER_ITEM * transfer);
                            self.command = 0;
                        } else {
                            self.item_qty = 0;
                            self.command = 0;
                        }
                    } else {
                        self.item_qty = 0;
                        self.command = 0;
                    }
                } else {
                    self.command = 255;
                }
            }
            // Command 4: Wait / route tube items
            4 => {
                if self.item_qty == 0 {
                    self.command = 0;
                }
            }
            _ => {
                self.command = 255;
            }
        }
    }

    /// Handle incoming tube item entering Sortron (`tubeItemEnter`).
    pub fn tube_item_enter(
        &mut self,
        side: Direction,
        state: i32,
        item: ItemStack,
        color: Option<DyeColor>,
    ) -> bool {
        // Rear bounce re-entry
        if side == self.rotation && state == TUBE_STATE_BOUNCE {
            self.buffer.add_bounce(BufferedItem { item, color });
            return true;
        }

        // Must enter from front in ENTER state
        if side != self.input_side() || state != TUBE_STATE_ENTER {
            return false;
        }

        // Only routes when command == 4
        if self.command != 4 {
            return false;
        }

        if self.voltage < SORTRON_MIN_VOLTAGE {
            return false;
        }

        let h = hash_item(&item);
        if self.item_type != 0 && self.item_type != h {
            return false;
        }

        let in_tag = tag_to_color(color);
        if self.item_in_color != 0 && self.item_in_color != in_tag {
            return false;
        }

        let count = item.count.min(self.item_qty as u32);
        let mut routed = item.clone();
        routed.count = count;

        self.item_qty = self.item_qty.saturating_sub(count as u8);
        if self.item_qty == 0 {
            self.command = 0;
        }

        let out_tag = color_to_tag(self.item_color);
        self.buffer.add_new_color(routed, out_tag);
        self.draw_power(SORTRON_POWER_PER_ITEM * count);
        true
    }

    /// Serialize state to key-value string map for chunk reload persistence (Level 5 contract).
    pub fn to_nbt(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("rotation".into(), (self.rotation as u8).to_string());
        map.insert("rbaddr".into(), self.rbaddr.to_string());
        map.insert("cmddelay".into(), self.cmd_delay.to_string());
        map.insert("cmd".into(), self.command.to_string());
        map.insert("itemslot".into(), self.item_slot.to_string());
        map.insert("itemtype".into(), self.item_type.to_string());
        map.insert("itemdmg".into(), self.item_damage.to_string());
        map.insert("itemdmgmax".into(), self.item_damage_max.to_string());
        map.insert("itemqty".into(), self.item_qty.to_string());
        map.insert("itemcolor".into(), self.item_color.to_string());
        map.insert("itemincolor".into(), self.item_in_color.to_string());
        map.insert("voltage".into(), self.voltage.to_string());
        map
    }

    /// Reconstitute state from key-value string map.
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
        if let Some(a) = map.get("rbaddr").and_then(|v| v.parse::<u8>().ok()) {
            s.rbaddr = a;
        }
        if let Some(d) = map.get("cmddelay").and_then(|v| v.parse::<u8>().ok()) {
            s.cmd_delay = d;
        }
        if let Some(c) = map.get("cmd").and_then(|v| v.parse::<u8>().ok()) {
            s.command = c;
        }
        if let Some(slot) = map.get("itemslot").and_then(|v| v.parse::<u16>().ok()) {
            s.item_slot = slot;
        }
        if let Some(t) = map.get("itemtype").and_then(|v| v.parse::<i32>().ok()) {
            s.item_type = t;
        }
        if let Some(dmg) = map.get("itemdmg").and_then(|v| v.parse::<u16>().ok()) {
            s.item_damage = dmg;
        }
        if let Some(max) = map.get("itemdmgmax").and_then(|v| v.parse::<u16>().ok()) {
            s.item_damage_max = max;
        }
        if let Some(q) = map.get("itemqty").and_then(|v| v.parse::<u8>().ok()) {
            s.item_qty = q;
        }
        if let Some(c) = map.get("itemcolor").and_then(|v| v.parse::<u8>().ok()) {
            s.item_color = c;
        }
        if let Some(inc) = map.get("itemincolor").and_then(|v| v.parse::<u8>().ok()) {
            s.item_in_color = inc;
        }
        if let Some(v) = map.get("voltage").and_then(|v| v.parse::<f64>().ok()) {
            s.voltage = v;
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

impl Rotatable for TileSortron {
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

impl FrameSupport for TileSortron {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileSortron frame packet header".into());
        }
        self.read_from_packet(&data[1..3])
            .map_err(|e| format!("packet decode error: {e:?}"))
    }
}

impl TubeConnectable for TileSortron {
    fn can_connect_tube(&self, side: Direction) -> bool {
        self.can_connect_tube(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::machine_lib::PlainCatalog;

    #[test]
    fn test_sortron_redbus_registers() {
        let mut s = TileSortron::new(Direction::North);
        assert_eq!(s.rb_get_addr(), DEFAULT_RBADDR);

        // Write and read command
        s.rb_write(0, 2);
        assert_eq!(s.rb_read(0), 2);

        // Write and read itemSlot (16-bit: reg 2 and 3)
        s.rb_write(2, 0x34);
        s.rb_write(3, 0x12);
        assert_eq!(s.item_slot, 0x1234);
        assert_eq!(s.rb_read(2), 0x34);
        assert_eq!(s.rb_read(3), 0x12);

        // Color and quantity
        s.rb_write(1, 15);
        s.rb_write(12, 5);
        assert_eq!(s.rb_read(1), 15);
        assert_eq!(s.rb_read(12), 5);
    }

    #[test]
    fn test_sortron_commands() {
        let mut s = TileSortron::new(Direction::North);
        let cat = PlainCatalog;

        let mut inv = vec![
            Some(ItemStack::new("minecraft:iron_ingot", 8)),
            Some(ItemStack::new("minecraft:gold_ingot", 4)),
        ];

        // Command 1: query length
        s.command = 1;
        s.process_command(Some(&mut inv), &cat);
        assert_eq!(s.command, 0);
        assert_eq!(s.item_slot, 2);

        // Command 2: query slot 0
        s.command = 2;
        s.item_slot = 0;
        s.process_command(Some(&mut inv), &cat);
        assert_eq!(s.command, 0);
        assert_eq!(s.item_qty, 8);
        assert_ne!(s.item_type, 0);

        // Command 3: extract 3 items from slot 0, color tag 2
        s.command = 3;
        s.item_slot = 0;
        s.item_qty = 3;
        s.item_color = 2;
        s.process_command(Some(&mut inv), &cat);
        assert_eq!(s.command, 0);
        assert_eq!(inv[0].as_ref().unwrap().count, 5); // 8 - 3 = 5
        assert_eq!(s.power_drawn, (50 * 3) as f64);
        assert_eq!(s.buffer.size(), 1);
    }

    #[test]
    fn test_sortron_tube_routing() {
        let mut s = TileSortron::new(Direction::North);
        s.command = 4; // wait/route
        s.item_qty = 5;
        s.item_color = 9; // tag Cyan
        let item = ItemStack::new("minecraft:diamond", 5);
        s.item_type = hash_item(&item);

        let routed = s.tube_item_enter(Direction::South, TUBE_STATE_ENTER, item, None);
        assert!(routed);
        assert_eq!(s.command, 0); // finished routing wanted qty
        assert_eq!(s.item_qty, 0);
        assert_eq!(s.power_drawn, (50 * 5) as f64);
    }

    #[test]
    fn test_sortron_chunk_persistence() {
        let mut original = TileSortron::new(Direction::West);
        original.rbaddr = 7;
        original.command = 4;
        original.item_slot = 42;
        original.item_qty = 10;
        original.item_type = 12345678;
        original.voltage = 110.0;

        let nbt = original.to_nbt();
        let restored = TileSortron::from_nbt(&nbt);

        assert_eq!(restored.rotation, Direction::West);
        assert_eq!(restored.rbaddr, 7);
        assert_eq!(restored.command, 4);
        assert_eq!(restored.item_slot, 42);
        assert_eq!(restored.item_qty, 10);
        assert_eq!(restored.item_type, 12345678);
        assert_eq!(restored.voltage, 110.0);
    }

    #[test]
    fn test_sortron_packet_and_frame_support() {
        let mut sortron = TileSortron::new(Direction::West);
        sortron.flags.active = true;
        sortron.flags.charged = true;

        let pkt = sortron.write_tile_packet(10, 20, 30);
        assert_eq!(pkt.sub_id, SUBID_TILE_MACHINE);
        assert_eq!(pkt.x, 10);
        assert_eq!(pkt.y, 20);
        assert_eq!(pkt.z, 30);

        let mut restored = TileSortron::new(Direction::Down);
        restored.handle_tile_packet(&pkt).unwrap();
        assert_eq!(restored.rotation, Direction::West);
        assert!(restored.flags.active);
        assert!(restored.flags.charged);

        let frame_pkt = sortron.frame_packet();
        let mut frame_restored = TileSortron::new(Direction::Down);
        frame_restored.handle_frame_packet(&frame_pkt).unwrap();
        assert_eq!(frame_restored.rotation, Direction::West);
        assert!(frame_restored.flags.active);
    }
}
