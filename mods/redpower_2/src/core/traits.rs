//! RedPower 2 core traits and the VLC packet codec.
//!
//! Ports the `eloraam.core` interfaces (`IConnectable`, `ICoverable`, `IMultipart`, `IRotatable`,
//! `IPaintable`, `IFrameSupport`, `IPipeConnectable`, `ITubeConnectable`, `IHandlePackets`,
//! `IHandleGuiEvent`), `CoreLib.compareItemStack`, and `PacketVLC` / `Packet211TileDesc` /
//! `Packet212GuiEvent`.

use rekkit_core::inventory::ItemStack;
use std::cmp::Ordering;
use std::io;

/// Wire connection queries.
pub trait Connectable {
    fn connectable_mask(&self) -> u32;
    fn connect_class(&self, side: i32) -> i32;
    fn corner_power_mode(&self) -> i32;
}

/// Cover slots on a multipart tile.
pub trait Coverable {
    fn can_add_cover(&self, side: i32, cover: i32) -> bool;
    fn try_add_cover(&mut self, side: i32, cover: i32) -> bool;
    /// Removed cover id, or -1 when none.
    fn try_remove_cover(&mut self, side: i32) -> i32;
    fn cover(&self, side: i32) -> i32;
    fn cover_mask(&self) -> u32;
}

/// A multipart tile; harvesting yields the dropped item stacks.
pub trait Multipart {
    fn is_side_solid(&self, side: i32) -> bool;
    fn is_side_normal(&self, side: i32) -> bool;
    fn harvest_multipart(&mut self) -> Vec<ItemStack>;
}

pub trait Rotatable {
    fn part_max_rotation(&self, part: i32, sec: bool) -> i32;
    fn part_rotation(&self, part: i32, sec: bool) -> i32;
    fn set_part_rotation(&mut self, part: i32, sec: bool, rot: i32);
}

pub trait Paintable {
    fn try_paint(&mut self, part: i32, side: i32, color: i32) -> bool;
}

/// Frame-carried tiles serialise to a byte packet and react to frame moves.
pub trait FrameSupport {
    fn frame_packet(&self) -> Vec<u8>;
    fn handle_frame_packet(&mut self, data: &[u8]) -> io::Result<()>;
    fn on_frame_refresh(&mut self);
    fn on_frame_pickup(&mut self);
    fn on_frame_drop(&mut self);
}

pub trait PipeConnectable {
    fn pipe_connectable_sides(&self) -> u32;
    fn pipe_flange_sides(&self) -> u32;
    fn pipe_pressure(&self, side: i32) -> i32;
}

pub trait TubeConnectable<Item> {
    fn tube_connectable_sides(&self) -> u32;
    fn tube_con_class(&self) -> i32;
    fn can_route_items(&self) -> bool;
    fn tube_item_enter(&mut self, side: i32, state: i32, item: &Item) -> bool;
    fn tube_item_can_enter(&self, side: i32, state: i32, item: &Item) -> bool;
    fn tube_weight(&self, side: i32, state: i32) -> i32;
}

pub trait HandlePackets {
    fn handle_packet(&mut self, pkt: &Packet211TileDesc);
}

pub trait HandleGuiEvent {
    fn handle_gui_event(&mut self, pkt: &Packet212GuiEvent);
}

/// `CoreLib.compareItemStack`: by item id, then by damage only when the item has subtypes.
pub fn compare_item_stack(a: &ItemStack, b: &ItemStack, has_subtypes: bool) -> Ordering {
    match a.item_id.cmp(&b.item_id) {
        Ordering::Equal if has_subtypes => a.damage.cmp(&b.damage),
        Ordering::Equal => Ordering::Equal,
        o => o,
    }
}

// ---------------------------------------------------------------------------
// Variable-length-coded packet body
// ---------------------------------------------------------------------------

pub fn write_uvlc(out: &mut Vec<u8>, mut l: u64) {
    loop {
        let mut b = (l & 127) as u8;
        l >>= 7;
        if l != 0 {
            b |= 128;
        }
        out.push(b);
        if l == 0 {
            break;
        }
    }
}

pub fn write_vlc(out: &mut Vec<u8>, l: i64) {
    let u = if l >= 0 {
        (l as u64) << 1
    } else {
        (l.unsigned_abs() << 1) | 1
    };
    write_uvlc(out, u);
}

fn eof() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data")
}

/// Read cursor over a packet body (`PacketVLC` input side).
#[derive(Debug, Clone, Default)]
pub struct VlcReader {
    data: Vec<u8>,
    pos: usize,
}

impl VlcReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn byte(&mut self) -> io::Result<u8> {
        let b = *self.data.get(self.pos).ok_or_else(eof)?;
        self.pos += 1;
        Ok(b)
    }

    pub fn uvlc(&mut self) -> io::Result<u64> {
        let mut r = 0u64;
        let mut sc = 0u32;
        loop {
            let b = self.byte()?;
            if sc < 64 {
                r |= ((b & 127) as u64) << sc;
            }
            if b & 128 == 0 {
                return Ok(r);
            }
            sc += 7;
            if sc > 64 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad VLC"));
            }
        }
    }

    pub fn vlc(&mut self) -> io::Result<i64> {
        let r = self.uvlc()?;
        Ok(if r & 1 == 0 {
            (r >> 1) as i64
        } else {
            -((r >> 1) as i64)
        })
    }

    pub fn byte_array(&mut self) -> io::Result<Vec<u8>> {
        let n = self.uvlc()? as usize;
        let end = self
            .pos
            .checked_add(n)
            .filter(|e| *e <= self.data.len())
            .ok_or_else(eof)?;
        let v = self.data[self.pos..end].to_vec();
        self.pos = end;
        Ok(v)
    }
}

/// Body writer shared by both packet kinds.
#[derive(Debug, Clone, Default)]
pub struct VlcBody(pub Vec<u8>);

impl VlcBody {
    pub fn add_byte(&mut self, b: u8) {
        self.0.push(b);
    }
    pub fn add_uvlc(&mut self, l: u64) {
        write_uvlc(&mut self.0, l);
    }
    pub fn add_vlc(&mut self, l: i64) {
        write_vlc(&mut self.0, l);
    }
    pub fn add_byte_array(&mut self, b: &[u8]) {
        self.add_uvlc(b.len() as u64);
        self.0.extend_from_slice(b);
    }
}

const MAX_BODY: usize = 65535;

/// Packet 211: tile description for the block at (x, y, z).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Packet211TileDesc {
    pub sub_id: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub body: Vec<u8>,
}

impl Packet211TileDesc {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.sub_id];
        write_vlc(&mut o, self.x as i64);
        write_vlc(&mut o, self.y as i64);
        write_vlc(&mut o, self.z as i64);
        write_uvlc(&mut o, self.body.len() as u64);
        o.extend_from_slice(&self.body);
        o
    }

    pub fn decode(data: &[u8]) -> io::Result<Self> {
        let mut r = VlcReader::new(data.to_vec());
        let sub_id = r.byte()?;
        let x = r.vlc()? as i32;
        let y = r.vlc()? as i32;
        let z = r.vlc()? as i32;
        let l = r.uvlc()? as usize;
        if l > MAX_BODY {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Packet too big"));
        }
        let body = data.get(r.pos..r.pos + l).ok_or_else(eof)?.to_vec();
        Ok(Self {
            sub_id,
            x,
            y,
            z,
            body,
        })
    }

    pub fn reader(&self) -> VlcReader {
        VlcReader::new(self.body.clone())
    }
}

/// Packet 212: GUI event for an open window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Packet212GuiEvent {
    pub window_id: u8,
    pub event_id: u32,
    pub body: Vec<u8>,
}

impl Packet212GuiEvent {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.window_id];
        write_uvlc(&mut o, self.event_id as u64);
        write_uvlc(&mut o, self.body.len() as u64);
        o.extend_from_slice(&self.body);
        o
    }

    pub fn decode(data: &[u8]) -> io::Result<Self> {
        let mut r = VlcReader::new(data.to_vec());
        let window_id = r.byte()?;
        let event_id = r.uvlc()? as u32;
        let l = r.uvlc()? as usize;
        if l > MAX_BODY {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Packet too big"));
        }
        let body = data.get(r.pos..r.pos + l).ok_or_else(eof)?.to_vec();
        Ok(Self {
            window_id,
            event_id,
            body,
        })
    }

    pub fn reader(&self) -> VlcReader {
        VlcReader::new(self.body.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vlc_round_trip() {
        let mut o = Vec::new();
        for v in [0i64, 1, -1, 63, -64, 1000, -100000, i32::MAX as i64] {
            write_vlc(&mut o, v);
        }
        let mut r = VlcReader::new(o);
        for v in [0i64, 1, -1, 63, -64, 1000, -100000, i32::MAX as i64] {
            assert_eq!(r.vlc().unwrap(), v);
        }
        assert!(r.byte().is_err());
    }

    #[test]
    fn packet211_round_trip() {
        let mut b = VlcBody::default();
        b.add_uvlc(300);
        b.add_byte_array(&[1, 2, 3]);
        let p = Packet211TileDesc {
            sub_id: 7,
            x: -12,
            y: 64,
            z: 99999,
            body: b.0,
        };
        let q = Packet211TileDesc::decode(&p.encode()).unwrap();
        assert_eq!(p, q);
        let mut r = q.reader();
        assert_eq!(r.uvlc().unwrap(), 300);
        assert_eq!(r.byte_array().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn packet212_round_trip_and_truncation() {
        let p = Packet212GuiEvent {
            window_id: 3,
            event_id: 200,
            body: vec![9, 8],
        };
        let e = p.encode();
        assert_eq!(Packet212GuiEvent::decode(&e).unwrap(), p);
        assert!(Packet212GuiEvent::decode(&e[..e.len() - 1]).is_err());
    }

    #[test]
    fn compare_stacks() {
        let a = ItemStack {
            item_id: "a".into(),
            count: 1,
            damage: 1,
            max_stack_size: 64,
        };
        let mut b = a.clone();
        b.damage = 2;
        assert_eq!(compare_item_stack(&a, &b, false), Ordering::Equal);
        assert_eq!(compare_item_stack(&a, &b, true), Ordering::Less);
        b.item_id = "0".into();
        assert_eq!(compare_item_stack(&a, &b, true), Ordering::Greater);
    }
}
