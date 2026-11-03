//! Ribbon cable: a flat 8-bit bus microblock mounted on one face of a block space
//! (`BlockMicro` damage 3072, `planning/mods/redpower_2.md` §2.2 and the item table).
//! It is 2/16 thick (contract shape `[0,0,0,16,2,16]`) and links neighbouring bus devices
//! (CPU, RAM, display backplanes) and other ribbon cables on the same face.

use crate::Direction;
use serde::{Deserialize, Serialize};

pub const RIBBON_ITEM_ID: &str = "redpower_2:ribbon_cable";
pub const RIBBON_THICKNESS_SIXTEENTHS: u8 = 2;

/// `TileRibbon.getConnectClass` (also `TileCPU`, `TileDiskDrive`, `TileDisplay`): the
/// ribbon bus is its own connection class; `RedPowerWiring` registers 66 compatible with 67 only.
pub const RIBBON_CONNECT_CLASS: u8 = 66;

/// `RedPowerLib.isCompatible` restricted to the ribbon class: same class, or the 66<->67 mapping.
pub fn ribbon_class_compatible(other: u8) -> bool {
    other == RIBBON_CONNECT_CLASS || other == 67
}

/// One placed ribbon cable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RibbonCable {
    /// Face of the block space the cable lies against (the supporting face).
    pub face: Direction,
    /// Current 8-bit bus value.
    pub value: u8,
    /// Bitmask over `Direction as usize` of sides the cable connects to.
    pub connections: u8,
}

impl RibbonCable {
    /// Place against the clicked face of the targeted block: the cable sits in the adjacent
    /// space with its backing toward the clicked block, so `face` is the clicked face's opposite.
    pub fn place_on_clicked_face(clicked_face: Direction) -> Self {
        Self {
            face: clicked_face.opposite(),
            value: 0,
            connections: 0,
        }
    }

    /// Sides in the cable's plane (the four not perpendicular to `face`).
    pub fn plane_sides(&self) -> impl Iterator<Item = Direction> + '_ {
        let face = self.face;
        Direction::ALL
            .into_iter()
            .filter(move |d| *d != face && *d != face.opposite())
    }

    /// Mark a side connected if it lies in the cable's plane.
    pub fn connect(&mut self, side: Direction) -> bool {
        if side == self.face || side == self.face.opposite() {
            return false;
        }
        self.connections |= 1 << (side as u8);
        true
    }

    /// Connect toward a neighbour of connection class `other` (as `getConSides` does:
    /// incompatible classes, e.g. redstone wire (0) or bluewire (64), never join the bus).
    pub fn connect_to_class(&mut self, side: Direction, other: u8) -> bool {
        ribbon_class_compatible(other) && self.connect(side)
    }

    pub fn is_connected(&self, side: Direction) -> bool {
        self.connections & (1 << (side as u8)) != 0
    }

    /// Drive the bus; every bit is independent, so concurrent drivers are OR-ed.
    pub fn drive(&mut self, drivers: &[u8]) {
        self.value = drivers.iter().fold(0, |a, b| a | b);
    }

    /// Bit `n` (0..8) of the bus.
    pub fn bit(&self, n: u8) -> bool {
        n < 8 && self.value & (1 << n) != 0
    }

    /// Persistence: `[face, value, connections]`.
    pub fn to_bytes(&self) -> [u8; 3] {
        [self.face as u8, self.value, self.connections]
    }

    pub fn from_bytes(b: [u8; 3]) -> Option<Self> {
        let face = *Direction::ALL.get(b[0] as usize)?;
        Some(Self {
            face,
            value: b[1],
            connections: b[2],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_faces_opposite_of_click() {
        let c = RibbonCable::place_on_clicked_face(Direction::Up);
        assert_eq!(c.face, Direction::Down);
    }

    #[test]
    fn connects_only_in_plane() {
        let mut c = RibbonCable::place_on_clicked_face(Direction::Up);
        assert!(!c.connect(Direction::Up));
        assert!(!c.connect(Direction::Down));
        assert!(c.connect(Direction::East));
        assert!(c.is_connected(Direction::East));
        assert_eq!(c.plane_sides().count(), 4);
    }

    #[test]
    fn class_rules() {
        let mut c = RibbonCable::place_on_clicked_face(Direction::Up);
        assert!(!c.connect_to_class(Direction::East, 0));
        assert!(!c.connect_to_class(Direction::East, 64));
        assert!(c.connect_to_class(Direction::East, 66));
        assert!(ribbon_class_compatible(67));
    }

    #[test]
    fn bus_or_and_roundtrip() {
        let mut c = RibbonCable::place_on_clicked_face(Direction::North);
        c.connect(Direction::Up);
        c.drive(&[0b0000_0101, 0b1000_0000]);
        assert_eq!(c.value, 0b1000_0101);
        assert!(c.bit(7) && !c.bit(1) && !c.bit(8));
        assert_eq!(RibbonCable::from_bytes(c.to_bytes()), Some(c));
        assert_eq!(RibbonCable::from_bytes([9, 0, 0]), None);
    }
}
