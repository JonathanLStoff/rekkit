//! Redbus address lookup: port of `RedbusLib.getAddr` / `RedbusPathfinder`, plus the
//! `WirePathfinder` search-block expansion it relies on.

use crate::core::spatial::WorldCoord;
use crate::wiring::rules::con_dir_mask;
use std::collections::{HashSet, VecDeque};

/// What the search needs to know about the tile at a position.
#[derive(Debug, Clone, Copy, Default)]
pub struct RedbusNode {
    /// `IRedbusConnectable.rbGetAddr()`, if the tile is a Redbus device.
    pub device_addr: Option<i32>,
    /// `IWiring` connection masks `(getConnectionMask, getExtConnectionMask)`, if a wire.
    pub wiring: Option<(u32, u32)>,
}

/// World lookup for the search.
pub trait RedbusWorld {
    fn node(&self, wc: WorldCoord) -> RedbusNode;
}

/// Neighbours of `wc` selected by connection mask `cons` and corner mask `indcon`.
pub fn search_blocks(wc: WorldCoord, cons: u32, indcon: u32) -> Vec<WorldCoord> {
    let mut out = Vec::new();
    for a in 0..6i32 {
        if cons & con_dir_mask(a) > 0 {
            out.push(wc.coord_step(a));
        }
    }
    for d1 in 0..6i32 {
        for b in 0..4i32 {
            if indcon & (1 << (d1 * 4 + b)) > 0 {
                out.push(wc.ind_step_coord(d1, b));
            }
        }
    }
    out
}

/// Breadth-first search through wiring from `pos` for the device at Redbus address `addr`.
/// Returns the device's position.
pub fn get_addr<W: RedbusWorld>(world: &W, pos: WorldCoord, addr: i32) -> Option<WorldCoord> {
    let mut seen: HashSet<WorldCoord> = HashSet::new();
    let mut queue: VecDeque<WorldCoord> = VecDeque::new();
    let add = |wc: WorldCoord, seen: &mut HashSet<WorldCoord>, q: &mut VecDeque<_>| {
        if seen.insert(wc) {
            q.push_back(wc);
        }
    };
    for wc in search_blocks(pos, 16777215, 0) {
        add(wc, &mut seen, &mut queue);
    }
    while let Some(wc) = queue.pop_front() {
        let node = world.node(wc);
        if node.device_addr == Some(addr) {
            return Some(wc);
        }
        if let Some((c, e)) = node.wiring {
            for n in search_blocks(wc, c, e) {
                add(n, &mut seen, &mut queue);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct W(HashMap<WorldCoord, RedbusNode>);
    impl RedbusWorld for W {
        fn node(&self, wc: WorldCoord) -> RedbusNode {
            self.0.get(&wc).copied().unwrap_or_default()
        }
    }

    #[test]
    fn finds_device_through_wire() {
        let mut m = HashMap::new();
        for x in 1..=3 {
            m.insert(
                WorldCoord::new(x, 0, 0),
                RedbusNode {
                    device_addr: None,
                    wiring: Some((16777215, 0)),
                },
            );
        }
        m.insert(
            WorldCoord::new(4, 0, 0),
            RedbusNode {
                device_addr: Some(3),
                wiring: None,
            },
        );
        let w = W(m);
        assert_eq!(
            get_addr(&w, WorldCoord::ZERO, 3),
            Some(WorldCoord::new(4, 0, 0))
        );
        assert_eq!(get_addr(&w, WorldCoord::ZERO, 9), None);
    }
}
