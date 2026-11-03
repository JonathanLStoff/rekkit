//! Redstone rules of `com.eloraam.redpower.core.RedPowerLib`: connection-mask mapping, vanilla
//! powering checks, connectable-face queries and around-the-corner (external) connections.
//!
//! The Java reads the world through block ids and tile entities; here the world is the
//! [`RuleWorld`] trait so the rules stay pure and unit-testable (no engine calls). Connection
//! masks are 30-bit: bit `face * 4 + n` of [`CON_DIR_MASKS`] words, as in the original.

use crate::core::spatial::WorldCoord;
use crate::wiring::network::CON_DIR_MASKS;
use std::collections::HashSet;

/// Every connection bit (`1073741823`).
pub const ALL_CONNECTIONS: u32 = 1_073_741_823;
/// Bits of the 24 on-face connections (`16777215`).
pub const FACE_CONNECTIONS: u32 = 16_777_215;

/// `getConDirMask(dir)`: connection bits facing `dir`; any `dir > 4` is the +X mask.
pub fn con_dir_mask(dir: i32) -> u32 {
    CON_DIR_MASKS[dir.clamp(0, 5) as usize]
}

fn swap1(c: u32) -> u32 {
    c ^ (((c ^ (c >> 1)) & 1) * 3)
}

fn swap2(c: u32) -> u32 {
    c ^ (((c ^ (c >> 2)) & 3) * 5)
}

/// `mapConToLocal(cons, face)`.
pub fn map_con_to_local(cons: u32, face: u32) -> u32 {
    let c = (cons >> (face * 4)) & 15;
    match face {
        0 => c,
        1 => swap1(c),
        3 | 4 => {
            let c = swap2(c);
            swap1(c)
        }
        _ => swap2(c),
    }
}

/// `mapLocalToCon(loc, face)`.
pub fn map_local_to_con(loc: u32, face: u32) -> u32 {
    let l = match face {
        0 => loc,
        1 => swap1(loc),
        3 | 4 => {
            let l = swap1(loc);
            swap2(l)
        }
        _ => swap2(loc),
    };
    l << (face * 4)
}

/// `mapRotToLocal(rm, rot)` (`rot` 0..=3).
pub fn map_rot_to_local(rm: u32, rot: u32) -> u32 {
    let rm = ((rm << rot) | (rm >> (4 - rot))) & 15;
    (rm & 8) | ((rm & 3) << 1) | ((rm >> 2) & 1)
}

/// `mapLocalToRot(rm, rot)` (`rot` 0..=3).
pub fn map_local_to_rot(rm: u32, rot: u32) -> u32 {
    let rm = (rm & 8) | ((rm & 6) >> 1) | ((rm << 2) & 4);
    ((rm << (4 - rot)) | (rm >> rot)) & 15
}

/// `mapConToRot(con, rot)`: `rot` is `face * 4 + spin`.
pub fn map_con_to_rot(con: u32, rot: u32) -> u32 {
    map_local_to_rot(map_con_to_local(con, rot >> 2), rot & 3)
}

/// `mapRotToCon(con, rot)`.
pub fn map_rot_to_con(con: u32, rot: u32) -> u32 {
    map_local_to_con(map_rot_to_local(con, rot & 3), rot >> 2)
}

/// `getDirToRedstone(rsd)`: our side to vanilla's horizontal redstone direction.
pub fn dir_to_redstone(rsd: i32) -> i32 {
    match rsd {
        3 => 2,
        4 => 3,
        5 => 1,
        _ => 0,
    }
}

/// Block one step from `p` through `side` (0..=5: -Y, +Y, -Z, +Z, -X, +X).
pub fn offset(p: WorldCoord, side: i32) -> WorldCoord {
    let (dx, dy, dz) = match side {
        0 => (0, -1, 0),
        1 => (0, 1, 0),
        2 => (0, 0, -1),
        3 => (0, 0, 1),
        4 => (-1, 0, 0),
        5 => (1, 0, 0),
        _ => (0, 0, 0),
    };
    WorldCoord::new(p.x + dx, p.y + dy, p.z + dz)
}

/// `addCompatibleMapping` / `isCompatible`: which connect classes talk to each other.
#[derive(Debug, Default, Clone)]
pub struct PowerClasses {
    pairs: HashSet<(i32, i32)>,
}

impl PowerClasses {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_compatible_mapping(&mut self, a: i32, b: i32) {
        self.pairs.insert((a, b));
        self.pairs.insert((b, a));
    }
    pub fn is_compatible(&self, a: i32, b: i32) -> bool {
        a == b || self.pairs.contains(&(a, b))
    }
}

/// The `IConnectable` / `IRedPowerConnectable` view of a tile.
#[derive(Debug, Clone, Default)]
pub struct Connectable {
    /// `getConnectClass(side)` for sides 0..=5.
    pub connect_class: [i32; 6],
    /// `getConnectableMask()`.
    pub connectable_mask: u32,
    /// `getCornerPowerMode()`: 0 none, 1 and 2 and 3 as in `getES1`.
    pub corner_mode: i32,
    /// `getPoweringMask(ch)` for channels 0..=16 (empty when not a power source).
    pub powering_mask: Vec<u32>,
}

impl Connectable {
    fn powering(&self, ch: usize) -> u32 {
        self.powering_mask.get(ch).copied().unwrap_or(0)
    }
}

/// What kind of block stands at a position, for the cases `RedPowerLib` special-cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    /// Vanilla redstone dust (`amq.ay`).
    RedstoneWire,
    /// Piston base or sticky piston (`amq.Y`, `amq.ac`) with metadata.
    Piston {
        meta: i32,
    },
    /// Piston head (`amq.af`); `base_is_piston` and the base's metadata from its tile.
    PistonExtension {
        base_is_piston: bool,
        meta: i32,
    },
    /// Blocks that connect on every face (`amq.S`, `aU`, `ci`, `aM`, `aT`, `aS`).
    ConnectsEverywhere,
    /// Redstone repeater (`amq.bk`, `bl`) with metadata.
    Repeater {
        meta: i32,
    },
    /// Microblock cover plate (`CoverLib.blockCoverPlate`) with metadata and its cover mask.
    CoverPlate {
        meta: i32,
        cover_mask: Option<u32>,
    },
    /// Fluid blocks (`amq.D`, `amq.E`) that an external connection may pass through.
    Fluid,
    Other,
}

/// World queries the rules need.
pub trait RuleWorld {
    fn kind(&self, p: WorldCoord) -> BlockKind;
    fn connectable(&self, p: WorldCoord) -> Option<Connectable>;
    fn classes(&self) -> &PowerClasses;
    /// `iba.t`: opaque normal cube.
    fn is_opaque_cube(&self, p: WorldCoord) -> bool;
    /// `world.f`: normal cube for wire support (only on a real world).
    fn is_normal_cube(&self, p: WorldCoord) -> bool;
    /// `world.isBlockSolidOnSide`.
    fn is_solid_on_side(&self, p: WorldCoord, side: i32) -> bool;
    /// `IMultipart.isSideNormal(side)` when the tile is a multipart.
    fn multipart_side_normal(&self, p: WorldCoord, side: i32) -> Option<bool>;
    /// `Block.b`: weak power the block gives toward `side`.
    fn weak_power(&self, p: WorldCoord, side: i32) -> bool;
    /// `Block.c`: strong power the block gives toward `side`.
    fn strong_power(&self, p: WorldCoord, side: i32) -> bool;
    /// `Block.canConnectRedstone`.
    fn can_connect_redstone(&self, p: WorldCoord, side: i32) -> bool;
    /// `RedPowerLib.searching`.
    fn searching(&self) -> bool;
}

fn exists<W: RuleWorld>(w: &W, p: WorldCoord) -> bool {
    w.kind(p) != BlockKind::Air
}

fn is_wire<W: RuleWorld>(w: &W, p: WorldCoord) -> bool {
    w.kind(p) == BlockKind::RedstoneWire
}

/// `isBlockRedstone(p, l)`: redstone dust one step through `l`.
pub fn is_block_redstone<W: RuleWorld>(w: &W, p: WorldCoord, l: i32) -> bool {
    is_wire(w, offset(p, l))
}

/// `isSideNormal(p, l)`: the neighbour through `l` has a solid face toward us.
pub fn is_side_normal<W: RuleWorld>(w: &W, p: WorldCoord, l: i32) -> bool {
    let q = offset(p, l);
    w.is_opaque_cube(q) || w.multipart_side_normal(q, l ^ 1).unwrap_or(false)
}

/// `canSupportWire(p, l)`. The piston branch compares the metadata with the block's x
/// coordinate, exactly as the decompiled original does (`i != md`).
pub fn can_support_wire<W: RuleWorld>(w: &W, p: WorldCoord, l: i32) -> bool {
    let q = offset(p, l);
    let l = l ^ 1;
    if !w.is_normal_cube(q) || w.is_solid_on_side(q, l) || w.is_opaque_cube(q) {
        return true;
    }
    match w.kind(q) {
        BlockKind::PistonExtension { .. } => true,
        BlockKind::Piston { meta } => {
            let md = meta & 7;
            q.x != md && md != 7
        }
        _ => w.multipart_side_normal(q, l).unwrap_or(false),
    }
}

/// `isStrongPoweringTo(p, l)`.
pub fn is_strong_powering_to<W: RuleWorld>(w: &W, p: WorldCoord, l: i32) -> bool {
    if !exists(w, p) || (w.searching() && is_wire(w, p)) {
        return false;
    }
    w.strong_power(p, l)
}

/// `isStrongPowered(p, l)`: any neighbour except the one through `l` strongly powers `p`.
pub fn is_strong_powered<W: RuleWorld>(w: &W, p: WorldCoord, l: i32) -> bool {
    // (excluded side, neighbour side, powering direction)
    const CHECKS: [(i32, i32, i32); 6] = [
        (1, 0, 0),
        (0, 1, 1),
        (3, 2, 2),
        (2, 3, 3),
        (5, 4, 4),
        (4, 5, 5),
    ];
    CHECKS
        .iter()
        .any(|&(skip, side, dir)| l != skip && is_strong_powering_to(w, offset(p, side), dir))
}

/// `isWeakPoweringTo(p, l)`.
pub fn is_weak_powering_to<W: RuleWorld>(w: &W, p: WorldCoord, l: i32) -> bool {
    if !exists(w, p) || (w.searching() && is_wire(w, p)) {
        return false;
    }
    w.weak_power(p, l) || (l > 1 && is_wire(w, p) && w.weak_power(p, 1))
}

/// `isPoweringTo(p, l)`.
pub fn is_powering_to<W: RuleWorld>(w: &W, p: WorldCoord, l: i32) -> bool {
    if !exists(w, p) {
        return false;
    }
    if w.weak_power(p, l) {
        return true;
    }
    if w.is_opaque_cube(p) && is_strong_powered(w, p, l) {
        return true;
    }
    if l > 1 && is_wire(w, p) {
        if w.searching() {
            return false;
        }
        if w.weak_power(p, 1) {
            return true;
        }
    }
    false
}

/// `isPowered(p, cons, indside)`: weakly powered through `cons` faces, or indirectly through
/// the faces in the `indside` bit set (bit `n` = side `n`).
pub fn is_powered<W: RuleWorld>(w: &W, p: WorldCoord, cons: u32, indside: u32) -> bool {
    (0..6).any(|d| cons & CON_DIR_MASKS[d as usize] != 0 && is_weak_powering_to(w, offset(p, d), d))
        || (0..6).any(|d| indside & (1 << d) != 0 && is_powering_to(w, offset(p, d), d))
}

/// `getSidePowerMask(p, ch, side)` (private in Java).
pub fn side_power_mask<W: RuleWorld>(w: &W, p: WorldCoord, ch: usize, side: i32) -> u32 {
    let mask = con_dir_mask(side);
    if let Some(irp) = w.connectable(p) {
        let m = irp.powering(ch);
        let m = ((m & 0x5555_5555) << 1) | ((m & 0x2AAA_AAAA) >> 1);
        m & mask
    } else if ch != 0 {
        0
    } else if is_weak_powering_to(w, p, side) {
        mask & FACE_CONNECTIONS
    } else if is_powering_to(w, p, side) {
        mask
    } else {
        0
    }
}

/// `getPowerState(p, cons, ch)`: which of the `cons` bits are powered on channel `ch`.
pub fn power_state<W: RuleWorld>(w: &W, p: WorldCoord, cons: u32, ch: usize) -> u32 {
    let mut trs = 0;
    for d in 0..6 {
        if cons & CON_DIR_MASKS[d as usize] != 0 {
            trs |= side_power_mask(w, offset(p, d), ch, d);
        }
    }
    trs & cons
}

/// `getRotPowerState(p, rcon, rot, ch)`.
pub fn rot_power_state<W: RuleWorld>(w: &W, p: WorldCoord, rcon: u32, rot: u32, ch: usize) -> u32 {
    let ps = power_state(w, p, map_rot_to_con(rcon, rot), ch);
    map_con_to_rot(ps, rot)
}

fn piston_cons(meta: i32) -> u32 {
    let md = meta & 7;
    if md == 7 {
        0
    } else {
        ALL_CONNECTIONS ^ con_dir_mask(md)
    }
}

/// `getConSides(p, side, pcl)`: connection bits the block at `p` offers toward `side`.
pub fn con_sides<W: RuleWorld>(w: &W, p: WorldCoord, side: i32, pcl: i32) -> u32 {
    let kind = w.kind(p);
    if kind == BlockKind::Air {
        return 0;
    }
    if let Some(rpa) = w.connectable(p) {
        let pc = rpa.connect_class[side as usize];
        return if w.classes().is_compatible(pc, pcl) {
            rpa.connectable_mask
        } else {
            0
        };
    }
    if !w.classes().is_compatible(0, pcl) {
        return 0;
    }
    match kind {
        BlockKind::Piston { meta } => piston_cons(meta),
        BlockKind::PistonExtension {
            base_is_piston,
            meta,
        } => {
            if base_is_piston {
                piston_cons(meta)
            } else {
                0
            }
        }
        BlockKind::ConnectsEverywhere => ALL_CONNECTIONS,
        BlockKind::Repeater { meta } => {
            if meta & 1 > 0 {
                12
            } else {
                3
            }
        }
        _ => {
            if w.can_connect_redstone(p, dir_to_redstone(side)) {
                con_dir_mask(side)
            } else {
                0
            }
        }
    }
}

fn ext_side_one<W: RuleWorld>(w: &W, p: WorldCoord, side: i32, pcl: i32, cc: i32) -> u32 {
    if !exists(w, p) {
        return 0;
    }
    let Some(rpa) = w.connectable(p) else {
        return 0;
    };
    let cc2 = rpa.corner_mode;
    if cc == 0 || cc2 == 0 || (cc == 2 && cc2 == 2) || (cc == 3 && cc2 == 1) {
        return 0;
    }
    let pc = rpa.connect_class[side as usize];
    if w.classes().is_compatible(pc, pcl) {
        rpa.connectable_mask
    } else {
        0
    }
}

/// `getExtConSides(irp, p, dir, cc)`: around-the-corner connections of the tile `irp` into
/// the block at `p`, which is one step from the tile through `dir`.
pub fn ext_con_sides<W: RuleWorld>(
    w: &W,
    irp: &Connectable,
    p: WorldCoord,
    dir: i32,
    cc: i32,
) -> u32 {
    let mut cons = irp.connectable_mask & con_dir_mask(dir) & FACE_CONNECTIONS;
    if cons == 0 {
        return 0;
    }
    match w.kind(p) {
        BlockKind::CoverPlate { meta, cover_mask } => {
            if meta != 0 {
                return 0;
            }
            let Some(mut m) = cover_mask else { return 0 };
            if m & (1 << (dir ^ 1)) > 0 {
                return 0;
            }
            m |= m << 12;
            m |= m << 6;
            m &= 197_379;
            m |= m << 3;
            m &= 1_118_481;
            m |= m << 2;
            m |= m << 1;
            cons &= !m;
        }
        BlockKind::Air | BlockKind::Fluid => {}
        _ => return 0,
    }
    let pcl = irp.connect_class[dir as usize];
    let mut isv = 0;
    // (cons bits tested, neighbour side, face the neighbour is read from, result mask)
    const PROBES: [(u32, i32, i32, u32); 6] = [
        (15, 0, 1, 2_236_928),
        (240, 1, 0, 1_118_464),
        (3840, 2, 3, 8_912_930),
        (61440, 3, 2, 4_456_465),
        (983_040, 4, 5, 34_952),
        (15_728_640, 5, 4, 17_476),
    ];
    for (test, step, face, keep) in PROBES {
        if cons & test > 0 {
            isv |= ext_side_one(w, offset(p, step), face, pcl, cc) & keep;
        }
    }
    isv >>= ((dir ^ 1) << 2) as u32;
    isv = ((isv & 10) >> 1) | ((isv & 5) << 1);
    isv |= isv << 6;
    isv |= isv << 3;
    isv &= 4369;
    isv <<= (dir & 1) as u32;
    match dir {
        0 | 1 => isv << 8,
        2 | 3 => ((isv << 10) & 0x00FF_0000) | (isv & 0xFF),
        _ => isv << 2,
    }
}

/// `getConnections(irp, p)`: direct connections of the tile `irp` standing at `p`.
pub fn connections<W: RuleWorld>(w: &W, irp: &Connectable, p: WorldCoord) -> u32 {
    let cons = irp.connectable_mask;
    // (neighbour side, face the neighbour is read from)
    const SIDES: [(i32, i32); 6] = [(0, 1), (1, 0), (2, 3), (3, 2), (4, 5), (5, 4)];
    let mut cs = 0;
    for (d, from) in SIDES {
        if cons & CON_DIR_MASKS[d as usize] != 0 {
            let pcl = irp.connect_class[d as usize];
            cs |= con_sides(w, offset(p, d), from, pcl) & CON_DIR_MASKS[from as usize];
        }
    }
    cs = ((cs << 1) & 715_827_882) | ((cs >> 1) & 357_913_941);
    cs & cons
}

/// `getExtConnections(irp, p)`.
pub fn ext_connections<W: RuleWorld>(w: &W, irp: &Connectable, p: WorldCoord) -> u32 {
    (0..6).fold(0, |cs, d| {
        cs | ext_con_sides(w, irp, offset(p, d), d, irp.corner_mode)
    })
}

/// `getExtConnectionExtras(irp, p)`: as above with corner mode 3.
pub fn ext_connection_extras<W: RuleWorld>(w: &W, irp: &Connectable, p: WorldCoord) -> u32 {
    (0..6).fold(0, |cs, d| cs | ext_con_sides(w, irp, offset(p, d), d, 3))
}

/// `updateIndirectNeighbors`: every position within Manhattan distance 3 that gets a neighbour
/// notification (343 cells minus the corners: 63 for distance <= 3 including `p` itself).
pub fn indirect_neighbors(p: WorldCoord) -> Vec<WorldCoord> {
    let mut out = Vec::new();
    for a in -3i32..=3 {
        for b in -3i32..=3 {
            for c in -3i32..=3 {
                if a.abs() + b.abs() + c.abs() <= 3 {
                    out.push(WorldCoord::new(p.x + a, p.y + b, p.z + c));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct W {
        kinds: HashMap<WorldCoord, BlockKind>,
        conn: HashMap<WorldCoord, Connectable>,
        opaque: HashSet<WorldCoord>,
        weak: HashSet<(WorldCoord, i32)>,
        strong: HashSet<(WorldCoord, i32)>,
        classes: PowerClasses,
    }

    impl RuleWorld for W {
        fn kind(&self, p: WorldCoord) -> BlockKind {
            self.kinds.get(&p).copied().unwrap_or(BlockKind::Air)
        }
        fn connectable(&self, p: WorldCoord) -> Option<Connectable> {
            self.conn.get(&p).cloned()
        }
        fn classes(&self) -> &PowerClasses {
            &self.classes
        }
        fn is_opaque_cube(&self, p: WorldCoord) -> bool {
            self.opaque.contains(&p)
        }
        fn is_normal_cube(&self, p: WorldCoord) -> bool {
            self.opaque.contains(&p)
        }
        fn is_solid_on_side(&self, p: WorldCoord, _: i32) -> bool {
            self.opaque.contains(&p)
        }
        fn multipart_side_normal(&self, _: WorldCoord, _: i32) -> Option<bool> {
            None
        }
        fn weak_power(&self, p: WorldCoord, s: i32) -> bool {
            self.weak.contains(&(p, s))
        }
        fn strong_power(&self, p: WorldCoord, s: i32) -> bool {
            self.strong.contains(&(p, s))
        }
        fn can_connect_redstone(&self, _: WorldCoord, _: i32) -> bool {
            false
        }
        fn searching(&self) -> bool {
            false
        }
    }

    const O: WorldCoord = WorldCoord::new(0, 0, 0);

    #[test]
    fn con_local_roundtrip() {
        for face in 0..6 {
            for loc in 0..16 {
                let con = map_local_to_con(loc, face);
                assert_eq!(map_con_to_local(con, face), loc, "face {face} loc {loc}");
            }
        }
    }

    #[test]
    fn rot_roundtrip() {
        for rot in 0..4 {
            for rm in 0..16 {
                assert_eq!(map_local_to_rot(map_rot_to_local(rm, rot), rot), rm);
            }
        }
        for rot in 0..24 {
            for rm in 0..16 {
                let con = map_rot_to_con(rm, rot);
                assert_eq!(map_con_to_rot(con, rot), rm);
            }
        }
    }

    #[test]
    fn redstone_direction_table() {
        assert_eq!([2, 3, 4, 5, 0].map(dir_to_redstone), [0, 2, 3, 1, 0]);
    }

    #[test]
    fn indirect_neighbors_count() {
        assert_eq!(indirect_neighbors(O).len(), 63);
    }

    #[test]
    fn weak_power_reaches_powered_flag() {
        let mut w = W::default();
        let n = offset(O, 4);
        w.kinds.insert(n, BlockKind::Other);
        w.weak.insert((n, 4));
        assert!(is_powered(&w, O, CON_DIR_MASKS[4], 0));
        assert!(!is_powered(&w, O, CON_DIR_MASKS[5], 0));
    }

    #[test]
    fn strong_power_through_opaque_block_is_indirect() {
        let mut w = W::default();
        let n = offset(O, 5);
        w.kinds.insert(n, BlockKind::Other);
        w.opaque.insert(n);
        let far = offset(n, 5);
        w.kinds.insert(far, BlockKind::Other);
        w.strong.insert((far, 5));
        assert!(is_strong_powered(&w, n, 5));
        assert!(is_powering_to(&w, n, 5));
        assert!(is_powered(&w, O, 0, 1 << 5));
        assert!(!is_powered(&w, O, 0, 1 << 4));
        // A direct (weak) check does not see strong power through the block.
        assert!(!is_powered(&w, O, CON_DIR_MASKS[5], 0));
    }

    #[test]
    fn side_power_mask_from_vanilla_block() {
        let mut w = W::default();
        let n = offset(O, 0);
        w.kinds.insert(n, BlockKind::Other);
        w.weak.insert((n, 0));
        assert_eq!(
            side_power_mask(&w, n, 0, 0),
            con_dir_mask(0) & FACE_CONNECTIONS
        );
        assert_eq!(side_power_mask(&w, n, 3, 0), 0);
        assert_eq!(
            power_state(&w, O, CON_DIR_MASKS[0] | CON_DIR_MASKS[1], 0),
            con_dir_mask(0) & FACE_CONNECTIONS
        );
    }

    #[test]
    fn connectable_power_mask_swaps_neighbour_bits() {
        let mut w = W::default();
        let n = offset(O, 1);
        w.kinds.insert(n, BlockKind::Other);
        w.conn.insert(
            n,
            Connectable {
                powering_mask: vec![0b01 << 0],
                ..Default::default()
            },
        );
        // bit 0 becomes bit 1 and is then masked by the +Y direction mask.
        assert_eq!(side_power_mask(&w, n, 0, 1), 2 & con_dir_mask(1));
    }

    #[test]
    fn connections_between_two_wires() {
        let mut w = W::default();
        let east = offset(O, 5);
        let mask = CON_DIR_MASKS[4] | CON_DIR_MASKS[5];
        let c = Connectable {
            connectable_mask: mask,
            ..Default::default()
        };
        w.kinds.insert(east, BlockKind::Other);
        w.conn.insert(east, c.clone());
        let got = connections(&w, &c, O);
        assert_ne!(got, 0);
        assert_eq!(got & !mask, 0);
        assert_eq!(got & CON_DIR_MASKS[4], 0, "nothing on -X");
    }

    #[test]
    fn incompatible_class_blocks_connection() {
        let mut w = W::default();
        let east = offset(O, 5);
        let mut c = Connectable {
            connectable_mask: CON_DIR_MASKS[5],
            ..Default::default()
        };
        let mut other = c.clone();
        other.connectable_mask = CON_DIR_MASKS[4];
        other.connect_class = [3; 6];
        w.kinds.insert(east, BlockKind::Other);
        w.conn.insert(east, other);
        assert_eq!(connections(&w, &c, O), 0);
        w.classes.add_compatible_mapping(0, 3);
        assert!(w.classes.is_compatible(3, 0));
        assert_ne!(connections(&w, &c, O), 0);
        c.connect_class = [3; 6];
        assert_ne!(connections(&w, &c, O), 0);
    }

    #[test]
    fn repeater_and_piston_connections() {
        let mut w = W::default();
        w.kinds.insert(O, BlockKind::Repeater { meta: 1 });
        assert_eq!(con_sides(&w, O, 0, 0), 12);
        w.kinds.insert(O, BlockKind::Repeater { meta: 0 });
        assert_eq!(con_sides(&w, O, 0, 0), 3);
        w.kinds.insert(O, BlockKind::Piston { meta: 7 });
        assert_eq!(con_sides(&w, O, 0, 0), 0);
        w.kinds.insert(O, BlockKind::Piston { meta: 1 });
        assert_eq!(con_sides(&w, O, 0, 0), ALL_CONNECTIONS ^ con_dir_mask(1));
    }

    #[test]
    fn ext_connection_needs_empty_corner_block() {
        let mut w = W::default();
        let c = Connectable {
            connectable_mask: 0xF,
            corner_mode: 1,
            ..Default::default()
        };
        // The corner position is solid: no external connection.
        let below = offset(O, 0);
        w.kinds.insert(below, BlockKind::Other);
        assert_eq!(ext_connections(&w, &c, O), 0);
    }

    #[test]
    fn can_support_wire_on_opaque_block() {
        let mut w = W::default();
        w.opaque.insert(offset(O, 0));
        w.kinds.insert(offset(O, 0), BlockKind::Other);
        assert!(can_support_wire(&w, O, 0));
        assert!(is_side_normal(&w, O, 0));
        assert!(!is_side_normal(&w, O, 1));
    }
}
