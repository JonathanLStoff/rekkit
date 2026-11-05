//! Red alloy and bundled wire network: the signal-propagation search of
//! `com.eloraam.redpower.core.RedPowerLib` (`updateCurrent`, `updateBlockCurrentStrength`,
//! `getMaxCurrentStrength`, `getIndCur`, `addSearchBlocks`, `addIndBl`, `addUpdateBlock`).
//!
//! Signal strength is 0..=255; a wire reads the strongest neighbouring strength and holds one
//! less (`cs = max(0, mx - 1)`), a source holds 255. Channel 0 is bare red alloy; channels
//! 1..=16 are the 16 dye colours (insulated wire and the 16 bits of a bundled cable).
//! The world is a pure map of tiles so the search runs without the engine.

use crate::core::spatial::WorldCoord;
use std::collections::{HashMap, HashSet, VecDeque};

/// `RedPowerLib.getConDirMask(dir)`: connection-mask bits that face direction `dir` (0..=5).
pub const CON_DIR_MASKS: [u32; 6] = [
    17_895_680,
    35_791_360,
    71_565_329,
    143_130_658,
    268_452_932,
    536_905_864,
];

/// Connection mask of every wire connection (`conm` of `TileRedwire`).
pub const CONM_REDWIRE: u32 = 1_073_741_823;
/// Connection mask of an insulated wire (`TileInsulatedWire`).
pub const CONM_INSULATED: u32 = 16_777_215;

/// Number of channels: 0 = red alloy, 1..=16 = colours.
pub const CHANNELS: usize = 17;

/// What a wire tile carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireKind {
    /// Bare red alloy wire: channel 0.
    Red,
    /// Insulated wire of dye colour 0..=15: channel 0 and channel `colour + 1`.
    Insulated(u8),
    /// Bundled cable: channels 1..=16.
    Cable,
}

/// One wire tile.
#[derive(Debug, Clone)]
pub struct WireNode {
    pub kind: WireKind,
    /// `getConnectionMask()`: direct connections.
    pub connection_mask: u32,
    /// `getExtConnectionMask()`: around-the-corner connections, bit `a * 4 + b`.
    pub ext_mask: u32,
    /// `getConnectableMask()`: faces from which this tile can be read.
    pub connectable_mask: u32,
    /// Where the wire sits (`ConaMask`): faces and edges it occupies, before any neighbour.
    pub base_mask: u32,
    /// Held strength per channel (`PowerState`).
    pub state: [i32; CHANNELS],
    /// `scanPoweringStrength` stand-in: strength pushed into this wire by adjacent sources.
    pub powering: [i32; CHANNELS],
}

impl WireNode {
    pub fn new(kind: WireKind, connection_mask: u32, ext_mask: u32) -> Self {
        Self {
            kind,
            connection_mask,
            ext_mask,
            connectable_mask: connection_mask | ext_mask,
            base_mask: connection_mask | ext_mask,
            state: [0; CHANNELS],
            powering: [0; CHANNELS],
        }
    }

    /// A wire occupying `base_mask` whose connections are computed from its neighbours by
    /// [`WireWorld::refresh_connections`].
    pub fn placed(kind: WireKind, base_mask: u32) -> Self {
        let mut n = Self::new(kind, 0, 0);
        n.connectable_mask = base_mask;
        n.base_mask = base_mask;
        n
    }

    /// `getConnectClass`: 1 red alloy, `2 + colour` insulated, 18 bundled cable.
    pub fn connect_class(&self) -> i32 {
        match self.kind {
            WireKind::Red => 1,
            WireKind::Insulated(c) => 2 + c as i32,
            WireKind::Cable => 18,
        }
    }

    fn conm(&self) -> u32 {
        match self.kind {
            WireKind::Insulated(_) => CONM_INSULATED,
            _ => CONM_REDWIRE,
        }
    }

    /// Channel mask the tile updates (`chm`) for one update call; cables use one call per channel.
    fn channel_masks(&self) -> Vec<u32> {
        match self.kind {
            WireKind::Red => vec![1],
            WireKind::Insulated(c) => vec![1 | (2 << c)],
            WireKind::Cable => (0..16).map(|ch| 2u32 << ch).collect(),
        }
    }

    /// `getCurrentStrength(cons, ch)`: -1 when the face or channel is not served.
    pub fn current_strength(&self, cons: u32, ch: usize) -> i32 {
        let served = match self.kind {
            WireKind::Red => ch == 0,
            WireKind::Insulated(c) => ch == 0 || ch == c as usize + 1,
            WireKind::Cable => (1..=16).contains(&ch),
        };
        if !served || cons & self.connectable_mask == 0 {
            return -1;
        }
        self.state[ch]
    }

    /// `scanPoweringStrength(cons, ch)`; cables take no external power.
    fn scan_powering(&self, ch: usize) -> i32 {
        match self.kind {
            WireKind::Cable => 0,
            WireKind::Red => {
                if ch == 0 {
                    self.powering[0]
                } else {
                    0
                }
            }
            WireKind::Insulated(_) => self.powering[ch],
        }
    }
}

/// Any tile the search can see.
#[derive(Debug, Clone)]
pub enum Tile {
    Wire(WireNode),
    /// Non-wire connectable (gate, lever...): per-channel mask of faces that output power.
    Source {
        powering_mask: [u32; CHANNELS],
    },
    /// Vanilla redstone dust with level 0..=15.
    Dust(i32),
}

/// Pure wire world plus the search state of `RedPowerLib`.
#[derive(Debug, Default, Clone)]
pub struct WireWorld {
    pub tiles: HashMap<WorldCoord, Tile>,
}

/// Highest channel bit of a `chm` mask is at most bit 16.
fn channels_of(mut chm: u32) -> impl Iterator<Item = usize> {
    std::iter::from_fn(move || {
        if chm == 0 {
            None
        } else {
            let ch = chm.trailing_zeros() as usize;
            chm &= !(1 << ch);
            Some(ch)
        }
    })
}

/// `addIndBl` / first half of `getIndCur`: the block reached by an around-the-corner link and
/// the intermediate direction. Returns `(pos after first step, d3)`.
fn ind_first(p: WorldCoord, d1: i32, d2: i32) -> (WorldCoord, i32) {
    let (mut x, mut y, mut z) = (p.x, p.y, p.z);
    let d3 = match d1 {
        0 => {
            y -= 1;
            d2 + 2
        }
        1 => {
            y += 1;
            d2 + 2
        }
        2 => {
            z -= 1;
            d2 + (d2 & 2)
        }
        3 => {
            z += 1;
            d2 + (d2 & 2)
        }
        4 => {
            x -= 1;
            d2
        }
        _ => {
            x += 1;
            d2
        }
    };
    (WorldCoord::new(x, y, z), d3)
}

fn step6(p: WorldCoord, d: i32) -> WorldCoord {
    let (mut x, mut y, mut z) = (p.x, p.y, p.z);
    match d {
        0 => y -= 1,
        1 => y += 1,
        2 => z -= 1,
        3 => z += 1,
        4 => x -= 1,
        5 => x += 1,
        _ => {}
    }
    WorldCoord::new(x, y, z)
}

impl WireWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, pos: WorldCoord, tile: Tile) {
        self.tiles.insert(pos, tile);
    }

    /// Held strength of a wire at `pos` on `ch` (0 when there is none).
    pub fn strength(&self, pos: WorldCoord, ch: usize) -> i32 {
        match self.tiles.get(&pos) {
            Some(Tile::Wire(w)) => w.state[ch],
            _ => 0,
        }
    }

    /// Output of an HSB-bundled cable as a 16-bit mask: bit n set when colour n carries signal.
    pub fn bundled_mask(&self, pos: WorldCoord) -> u16 {
        (0..16).fold(0u16, |m, c| {
            if self.strength(pos, c + 1) > 0 {
                m | (1 << c)
            } else {
                m
            }
        })
    }

    /// `getTileCurrentStrength`.
    fn tile_strength(&self, p: WorldCoord, cons: u32, ch: usize) -> i32 {
        match self.tiles.get(&p) {
            None => -1,
            Some(Tile::Wire(w)) => w.current_strength(cons, ch),
            Some(Tile::Source { powering_mask }) => {
                if powering_mask[ch] & cons > 0 {
                    255
                } else {
                    -1
                }
            }
            Some(Tile::Dust(_)) => -1,
        }
    }

    /// `getTileOrRedstoneCurrentStrength`.
    fn tile_or_dust_strength(&self, p: WorldCoord, cons: u32, ch: usize) -> i32 {
        if let Some(Tile::Dust(level)) = self.tiles.get(&p) {
            return if *level > 0 { *level } else { -1 };
        }
        self.tile_strength(p, cons, ch)
    }

    /// `getIndCur`: strength across a corner link `(d1, d2)`.
    fn ind_current(&self, p: WorldCoord, d1: i32, d2: i32, ch: usize) -> i32 {
        let (q, d3) = ind_first(p, d1, d2);
        let (r, d4) = match d3 {
            0 | 1 => (step6(q, d3), d1 - 2),
            2 | 3 => (step6(q, d3), (d1 & 1) | ((d1 & 4) >> 1)),
            _ => (step6(q, d3), d1),
        };
        if !(0..6).contains(&d4) || !(0..6).contains(&d3) {
            return -1;
        }
        let cons = (1u32 << (d4 ^ 1)) << (((d3 ^ 1) << 2) as u32);
        self.tile_strength(r, cons, ch)
    }

    /// `getMaxCurrentStrength`.
    fn max_current(&self, p: WorldCoord, cons: u32, indcon: u32, ch: usize) -> i32 {
        let mut mcs = -1;
        let ocon = (cons << 1 & 715_827_882) | (cons >> 1 & 357_913_941);
        // Opposite-face masks per direction, as in the original.
        let opposite = [
            35_791_360u32,
            17_895_680,
            143_130_658,
            71_565_329,
            536_905_864,
            268_452_932,
        ];
        for d in 0..6 {
            if cons & CON_DIR_MASKS[d] > 0 {
                mcs =
                    mcs.max(self.tile_or_dust_strength(step6(p, d as i32), ocon & opposite[d], ch));
            }
        }
        for a in 0..6 {
            for b in 0..4 {
                if indcon & (1 << (a * 4 + b)) > 0 {
                    mcs = mcs.max(self.ind_current(p, a, b, ch));
                }
            }
        }
        mcs
    }

    /// Neighbours queued by `addSearchBlocks` (direct sides first, then corner links).
    fn search_blocks(p: WorldCoord, cons: u32, indcon: u32) -> Vec<WorldCoord> {
        let mut out = Vec::new();
        for (d, &mask) in CON_DIR_MASKS.iter().enumerate() {
            if cons & mask > 0 {
                out.push(step6(p, d as i32));
            }
        }
        for a in 0..6 {
            for b in 0..4 {
                if indcon & (1 << (a * 4 + b)) > 0 {
                    let (q, d3) = ind_first(p, a, b);
                    out.push(step6(q, d3));
                }
            }
        }
        out
    }

    /// `updateBlockCurrentStrength`: new strength for one channel group of the wire at `p`.
    /// Appends queued neighbours to `search` and manhattan-3 notifications to `updates`.
    fn update_block(
        &self,
        p: WorldCoord,
        node: &WireNode,
        chm: u32,
        search: &mut Vec<WorldCoord>,
        updates: &mut HashSet<WorldCoord>,
    ) -> i32 {
        let conm = node.conm();
        let cons = node.connection_mask & conm;
        let indcon = node.ext_mask & conm;
        let (mut mx, mut ps, mut cs) = (-1, 0, 0);
        for ch in channels_of(chm) {
            cs = cs.max(node.current_strength(conm, ch));
            mx = mx.max(self.max_current(p, cons, indcon, ch));
            ps = ps.max(node.scan_powering(ch));
        }
        if ps > cs || (mx != cs + 1 && (cs != 0 || mx != 0)) {
            if ps == cs && mx <= cs {
                return cs;
            }
            cs = cs.max(ps);
            if cs >= mx {
                if cs > ps {
                    cs = 0;
                }
            } else {
                cs = (mx - 1).max(0);
            }
            if chm & 1 > 0 {
                add_update_block(p, updates);
            }
            search.extend(Self::search_blocks(p, cons, indcon));
            cs
        } else {
            cs
        }
    }

    /// `updateCurrent`: relax the network starting from `start` until stable. Returns the
    /// blocks to notify (the diamond of radius 3 around every changed red-alloy wire plus every
    /// searched neighbour), sorted.
    pub fn update_current(&mut self, start: WorldCoord) -> Vec<WorldCoord> {
        let mut queue: VecDeque<WorldCoord> = VecDeque::new();
        let mut queued: HashSet<WorldCoord> = HashSet::new();
        let mut updates: HashSet<WorldCoord> = HashSet::new();
        queue.push_back(start);
        queued.insert(start);
        // Guard against pathological oscillation (loops with a source of unequal height).
        let mut budget = 1_000_000usize;
        while let Some(p) = queue.pop_front() {
            queued.remove(&p);
            if budget == 0 {
                break;
            }
            budget -= 1;
            let node = match self.tiles.get(&p) {
                Some(Tile::Wire(w)) => w.clone(),
                _ => continue,
            };
            let mut search = Vec::new();
            let mut new_state = node.state;
            for chm in node.channel_masks() {
                let cs = self.update_block(p, &node, chm, &mut search, &mut updates);
                // Single-channel groups write their channel; insulated wires hold one value.
                match node.kind {
                    WireKind::Red => new_state[0] = cs,
                    WireKind::Insulated(c) => {
                        new_state[0] = cs;
                        new_state[c as usize + 1] = cs;
                    }
                    WireKind::Cable => new_state[chm.trailing_zeros() as usize] = cs,
                }
            }
            if let Some(Tile::Wire(w)) = self.tiles.get_mut(&p) {
                w.state = new_state;
            }
            for s in search {
                updates.insert(s);
                if queued.insert(s) {
                    queue.push_back(s);
                }
            }
        }
        let mut out: Vec<WorldCoord> = updates.into_iter().collect();
        out.sort_by_key(|c| (c.x, c.y, c.z));
        out
    }
}

/// `addUpdateBlock`: every block within manhattan distance 3.
fn add_update_block(p: WorldCoord, out: &mut HashSet<WorldCoord>) {
    for a in -3i32..=3 {
        for b in -3i32..=3 {
            for c in -3i32..=3 {
                if a.abs() + b.abs() + c.abs() <= 3 {
                    out.insert(WorldCoord::new(p.x + a, p.y + b, p.z + c));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bit of `CON_DIR_MASKS[dir]` that a straight wire uses: the lowest set bit.
    fn straight(dir: usize) -> u32 {
        CON_DIR_MASKS[dir] & CON_DIR_MASKS[dir].wrapping_neg()
    }

    fn line(kind: WireKind, n: i32) -> WireWorld {
        let mut w = WireWorld::new();
        for x in 1..=n {
            let mut m = 0;
            if x > 1 {
                m |= CON_DIR_MASKS[4];
            }
            if x < n {
                m |= CON_DIR_MASKS[5];
            }
            w.set(
                WorldCoord::new(x, 0, 0),
                Tile::Wire(WireNode::new(kind, m, 0)),
            );
        }
        w
    }

    #[test]
    fn straight_helper_is_nonzero() {
        assert!(straight(4) > 0);
    }

    #[test]
    fn red_wire_decays_one_per_hop() {
        let mut w = line(WireKind::Red, 5);
        if let Some(Tile::Wire(n)) = w.tiles.get_mut(&WorldCoord::new(1, 0, 0)) {
            n.powering[0] = 255;
        }
        w.update_current(WorldCoord::new(1, 0, 0));
        let got: Vec<i32> = (1..=5)
            .map(|x| w.strength(WorldCoord::new(x, 0, 0), 0))
            .collect();
        assert_eq!(got, vec![255, 254, 253, 252, 251]);
    }

    #[test]
    fn signal_clears_when_source_removed() {
        let mut w = line(WireKind::Red, 4);
        if let Some(Tile::Wire(n)) = w.tiles.get_mut(&WorldCoord::new(1, 0, 0)) {
            n.powering[0] = 255;
        }
        w.update_current(WorldCoord::new(1, 0, 0));
        assert_eq!(w.strength(WorldCoord::new(4, 0, 0), 0), 252);
        if let Some(Tile::Wire(n)) = w.tiles.get_mut(&WorldCoord::new(1, 0, 0)) {
            n.powering[0] = 0;
        }
        w.update_current(WorldCoord::new(1, 0, 0));
        for x in 1..=4 {
            assert_eq!(w.strength(WorldCoord::new(x, 0, 0), 0), 0, "x={x}");
        }
    }

    #[test]
    fn cable_carries_channels_independently() {
        let mut w = line(WireKind::Cable, 3);
        // Drive colour 2 (channel 3) at the first cable only; the cable scan returns 0,
        // so feed it through a source tile next to it.
        let mut pm = [0u32; CHANNELS];
        pm[3] = CON_DIR_MASKS[5];
        w.set(WorldCoord::new(0, 0, 0), Tile::Source { powering_mask: pm });
        if let Some(Tile::Wire(n)) = w.tiles.get_mut(&WorldCoord::new(1, 0, 0)) {
            n.connection_mask |= CON_DIR_MASKS[4];
            n.connectable_mask |= CON_DIR_MASKS[4];
        }
        w.update_current(WorldCoord::new(1, 0, 0));
        assert_eq!(w.strength(WorldCoord::new(3, 0, 0), 3), 252);
        assert_eq!(w.strength(WorldCoord::new(3, 0, 0), 4), 0);
        assert_eq!(w.bundled_mask(WorldCoord::new(3, 0, 0)), 1 << 2);
    }

    #[test]
    fn insulated_ignores_other_channels() {
        let w = line(WireKind::Insulated(5), 2);
        let Some(Tile::Wire(n)) = w.tiles.get(&WorldCoord::new(1, 0, 0)) else {
            panic!()
        };
        assert_eq!(n.current_strength(CONM_INSULATED, 7), -1);
        assert_eq!(n.current_strength(CONM_INSULATED, 6), 0);
    }

    #[test]
    fn update_diamond_has_63_blocks() {
        let mut s = HashSet::new();
        add_update_block(WorldCoord::new(0, 0, 0), &mut s);
        assert_eq!(s.len(), 63);
    }
}
