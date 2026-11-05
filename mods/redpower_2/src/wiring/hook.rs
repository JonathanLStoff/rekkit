//! Hooks `rules::connections` / `rules::ext_connections` into [`WireWorld`]: a wire placed or
//! removed recomputes its own and its neighbours' connection masks, then the signal relaxes.

use crate::core::spatial::WorldCoord;
use crate::wiring::network::{Tile, WireWorld, CHANNELS};
use crate::wiring::rules::{self, offset, BlockKind, Connectable, PowerClasses, RuleWorld};

/// `RedPowerWiring.initialize`: the compatible connect-class pairs (0 vanilla, 1 red alloy,
/// 2..=17 insulated colours, 18 cable, 19..=34 coloured cable, 64/65/67 gates and bus).
pub fn wiring_classes() -> PowerClasses {
    let mut c = PowerClasses::new();
    c.add_compatible_mapping(0, 1);
    for i in 0..16 {
        c.add_compatible_mapping(0, 2 + i);
        c.add_compatible_mapping(1, 2 + i);
        c.add_compatible_mapping(65, 2 + i);
        for j in 0..16 {
            c.add_compatible_mapping(19 + j, 2 + i);
        }
        c.add_compatible_mapping(18, 2 + i);
        c.add_compatible_mapping(18, 19 + i);
    }
    c.add_compatible_mapping(0, 65);
    c.add_compatible_mapping(1, 65);
    c.add_compatible_mapping(64, 65);
    c.add_compatible_mapping(64, 67);
    c.add_compatible_mapping(65, 67);
    c
}

/// Read-only [`RuleWorld`] view over a [`WireWorld`].
pub struct WireView<'a> {
    pub world: &'a WireWorld,
    pub classes: &'a PowerClasses,
}

impl RuleWorld for WireView<'_> {
    fn kind(&self, p: WorldCoord) -> BlockKind {
        match self.world.tiles.get(&p) {
            None => BlockKind::Air,
            Some(Tile::Dust(_)) => BlockKind::RedstoneWire,
            Some(_) => BlockKind::Other,
        }
    }
    fn connectable(&self, p: WorldCoord) -> Option<Connectable> {
        match self.world.tiles.get(&p)? {
            Tile::Wire(w) => Some(Connectable {
                connect_class: [w.connect_class(); 6],
                connectable_mask: w.connectable_mask,
                corner_mode: 1,
                powering_mask: Vec::new(),
            }),
            Tile::Source { powering_mask } => Some(Connectable {
                connect_class: [0; 6],
                connectable_mask: powering_mask.iter().fold(0, |a, m| a | m),
                corner_mode: 0,
                powering_mask: powering_mask.to_vec(),
            }),
            Tile::Dust(_) => None,
        }
    }
    fn classes(&self) -> &PowerClasses {
        self.classes
    }
    fn is_opaque_cube(&self, _: WorldCoord) -> bool {
        false
    }
    fn is_normal_cube(&self, _: WorldCoord) -> bool {
        false
    }
    fn is_solid_on_side(&self, _: WorldCoord, _: i32) -> bool {
        false
    }
    fn multipart_side_normal(&self, _: WorldCoord, _: i32) -> Option<bool> {
        None
    }
    fn weak_power(&self, _: WorldCoord, _: i32) -> bool {
        false
    }
    fn strong_power(&self, _: WorldCoord, _: i32) -> bool {
        false
    }
    fn can_connect_redstone(&self, p: WorldCoord, _: i32) -> bool {
        matches!(self.world.tiles.get(&p), Some(Tile::Dust(_)))
    }
    fn searching(&self) -> bool {
        false
    }
}

impl WireWorld {
    /// Recompute the connection and corner masks of the wire at `pos` from its neighbours.
    /// Returns whether either changed.
    pub fn refresh_connections(&mut self, pos: WorldCoord, classes: &PowerClasses) -> bool {
        let Some(Tile::Wire(w)) = self.tiles.get(&pos) else {
            return false;
        };
        let irp = Connectable {
            connect_class: [w.connect_class(); 6],
            connectable_mask: w.base_mask,
            corner_mode: 1,
            powering_mask: Vec::new(),
        };
        let view = WireView {
            world: self,
            classes,
        };
        let cons = rules::connections(&view, &irp, pos);
        let ext = rules::ext_connections(&view, &irp, pos);
        let Some(Tile::Wire(w)) = self.tiles.get_mut(&pos) else {
            return false;
        };
        let changed = w.connection_mask != cons || w.ext_mask != ext;
        w.connection_mask = cons;
        w.ext_mask = ext;
        w.connectable_mask = w.base_mask;
        changed
    }

    /// Wires that can be affected by a change at `pos`: itself, its six neighbours and the
    /// around-the-corner ring (every block within one step of those).
    fn nearby_wires(&self, pos: WorldCoord) -> Vec<WorldCoord> {
        let mut out = Vec::new();
        for dx in -2i32..=2 {
            for dy in -2i32..=2 {
                for dz in -2i32..=2 {
                    if dx.abs() + dy.abs() + dz.abs() <= 2 {
                        let q = WorldCoord::new(pos.x + dx, pos.y + dy, pos.z + dz);
                        if matches!(self.tiles.get(&q), Some(Tile::Wire(_))) {
                            out.push(q);
                        }
                    }
                }
            }
        }
        out
    }

    /// Place (or replace) a tile, refresh the connections around it, and relax the signal
    /// through the wires it touches. Returns the blocks to notify.
    pub fn place_tile(
        &mut self,
        pos: WorldCoord,
        tile: Tile,
        classes: &PowerClasses,
    ) -> Vec<WorldCoord> {
        self.tiles.insert(pos, tile);
        self.settle(pos, classes)
    }

    /// Remove a tile and settle the neighbours.
    pub fn remove_tile(&mut self, pos: WorldCoord, classes: &PowerClasses) -> Vec<WorldCoord> {
        self.tiles.remove(&pos);
        self.settle(pos, classes)
    }

    fn settle(&mut self, pos: WorldCoord, classes: &PowerClasses) -> Vec<WorldCoord> {
        let near = self.nearby_wires(pos);
        for &q in &near {
            self.refresh_connections(q, classes);
        }
        let mut starts: Vec<WorldCoord> = near;
        for d in 0..6 {
            let q = offset(pos, d);
            if matches!(self.tiles.get(&q), Some(Tile::Wire(_))) && !starts.contains(&q) {
                starts.push(q);
            }
        }
        let mut notify = Vec::new();
        for s in starts {
            notify.extend(self.update_current(s));
        }
        notify.sort_by_key(|c| (c.x, c.y, c.z));
        notify.dedup();
        notify
    }
}

#[allow(dead_code)]
const _: usize = CHANNELS;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wiring::network::{WireKind, WireNode, CON_DIR_MASKS};

    /// Face -Y occupancy (bits 0..=3 on the bottom face).
    const FLOOR: u32 = 1 << 24;

    fn c(x: i32) -> WorldCoord {
        WorldCoord::new(x, 0, 0)
    }

    fn floor_wire(kind: WireKind) -> Tile {
        // The connectable mask of a floor wire: its face-0 bits (all four edges) plus the face.
        Tile::Wire(WireNode::placed(kind, 15 | FLOOR))
    }

    #[test]
    fn adjacent_red_wires_connect_and_carry() {
        let cl = wiring_classes();
        let mut w = WireWorld::new();
        w.place_tile(c(1), floor_wire(WireKind::Red), &cl);
        w.place_tile(c(2), floor_wire(WireKind::Red), &cl);
        let Some(Tile::Wire(a)) = w.tiles.get(&c(1)) else {
            panic!()
        };
        assert!(a.connection_mask & CON_DIR_MASKS[5] != 0, "no +X link");
        if let Some(Tile::Wire(n)) = w.tiles.get_mut(&c(1)) {
            n.powering[0] = 255;
        }
        w.update_current(c(1));
        assert_eq!(w.strength(c(2), 0), 254);
    }

    #[test]
    fn insulated_colours_do_not_leak_but_join_red() {
        let cl = wiring_classes();
        let mut w = WireWorld::new();
        w.place_tile(c(1), floor_wire(WireKind::Insulated(3)), &cl);
        w.place_tile(c(2), floor_wire(WireKind::Insulated(4)), &cl);
        let Some(Tile::Wire(a)) = w.tiles.get(&c(1)) else {
            panic!()
        };
        // Different insulated colours are not compatible classes.
        assert_eq!(a.connection_mask & CON_DIR_MASKS[5], 0);
        w.place_tile(c(3), floor_wire(WireKind::Red), &cl);
        let Some(Tile::Wire(b)) = w.tiles.get(&c(2)) else {
            panic!()
        };
        assert!(b.connection_mask & CON_DIR_MASKS[5] != 0);
    }

    #[test]
    fn removing_a_wire_drops_the_link() {
        let cl = wiring_classes();
        let mut w = WireWorld::new();
        w.place_tile(c(1), floor_wire(WireKind::Red), &cl);
        w.place_tile(c(2), floor_wire(WireKind::Red), &cl);
        w.remove_tile(c(2), &cl);
        let Some(Tile::Wire(a)) = w.tiles.get(&c(1)) else {
            panic!()
        };
        assert_eq!(a.connection_mask & CON_DIR_MASKS[5], 0);
    }
}
