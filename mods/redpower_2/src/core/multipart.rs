//! Sub-voxel multipart infrastructure: `CoverLib` tables and placement validator, `TileCoverable`
//! bounds/strength/harvest rules and the `TileCovered` cover store (29 slots: 6 faces, 8 corners,
//! 12 strips, 3 posts). Pure data; no engine calls.

use super::foundation::PacketError;
use super::traits::{Coverable, Multipart};
use rekkit_core::inventory::ItemStack;
use std::collections::HashMap;

/// Number of cover slots on a tile.
pub const COVER_SLOTS: usize = 29;
const SLOT_MASK: u32 = (1 << 29) - 1;

/// Slot ranges: faces 0..6, corners 6..14, strips 14..26, posts 26..29.
pub fn is_face(side: usize) -> bool {
    side < 6
}
pub fn is_post(side: usize) -> bool {
    (26..29).contains(&side)
}

/// Item damage -> packed cover data (material in low byte, shape class in higher bytes).
pub fn damage_to_cover_data(dmg: i32) -> i32 {
    let hd = dmg >> 8;
    let mut cn = dmg & 0xFF;
    cn |= match hd {
        0 => 65536,
        16 => 131328,
        17 => 262656,
        18 => 33619968,
        19 => 33685760,
        20 => 33817088,
        21 => 16842752,
        22 => 16908544,
        23 => 17039872,
        24 => 1114880,
        25 => 1180672,
        26 => 1312000,
        27 => 198144,
        28 => 329472,
        29 => 395264,
        30 => 461056,
        31 => 1247744,
        32 => 1379072,
        33 => 1444864,
        34 => 1510656,
        35 => 33751808,
        36 => 33883136,
        37 => 33948928,
        38 => 34014720,
        39 => 16974592,
        40 => 17105920,
        41 => 17171712,
        42 => 17237504,
        43 => 50462720,
        44 => 50594048,
        45 => 50725376,
        _ => 0,
    };
    cn
}

/// Packed cover data -> item damage of the cover plate item for the slot.
pub fn cover_value_to_damage(side: usize, cov: i32) -> i32 {
    let hd = cov >> 8;
    let mut cn = cov & 0xFF;
    let add = if side < 6 {
        match hd {
            1 => 4096,
            2 => 4352,
            3 => 6144,
            4 => 6400,
            5 => 6656,
            6 => 6912,
            7 => 7168,
            8 => 7424,
            9 => 7680,
            10 => 7936,
            11 => 8192,
            12 => 8448,
            13 => 8704,
            _ => 0,
        }
    } else if side < 14 {
        match hd {
            0 => 4608,
            1 => 4864,
            2 => 5120,
            3 => 8960,
            4 => 9216,
            5 => 9472,
            6 => 9728,
            _ => 0,
        }
    } else if side < 26 {
        match hd {
            0 => 5376,
            1 => 5632,
            2 => 5888,
            3 => 9984,
            4 => 10240,
            5 => 10496,
            6 => 10752,
            _ => 0,
        }
    } else if side < 29 {
        match hd {
            0 => 11008,
            1 => 11264,
            2 => 11520,
            _ => 0,
        }
    } else {
        0
    };
    cn |= add;
    cn
}

/// Face mask (bits 0..6) touching corner `cn` (0..8).
pub fn corner_to_cover_mask(cn: usize) -> u32 {
    [21, 25, 37, 41, 22, 26, 38, 42][cn.min(7)]
}
/// Corner mask (8 bits) touched by face `cn`.
pub fn cover_to_corner_mask(cn: usize) -> u32 {
    [15, 240, 85, 170, 51, 204][cn.min(5)]
}
/// Strip mask (12 bits) touched by face `cn`.
pub fn cover_to_strip_mask(cn: usize) -> u32 {
    [15, 3840, 337, 674, 1076, 2248][cn.min(5)]
}
/// Corner mask touched by strip `sn` (0..12).
pub fn strip_to_corner_mask(sn: usize) -> u32 {
    [5, 10, 3, 12, 17, 34, 68, 136, 80, 160, 48, 192][sn.min(11)]
}
/// Face mask touched by strip `sn`.
pub fn strip_to_cover_mask(sn: usize) -> u32 {
    [5, 9, 17, 33, 20, 24, 36, 40, 6, 10, 18, 34][sn.min(11)]
}

/// Thickness in blocks of the cover in `side` with packed value `cov`.
pub fn thickness(side: usize, cov: i32) -> f32 {
    match thickness_quanta(side, cov) {
        0 => 1.0,
        q => q as f32 / 8.0,
    }
}

/// Thickness in 1/8 units; 0 means a full block (the Java default case).
pub fn thickness_quanta(side: usize, cov: i32) -> i32 {
    let h = cov >> 8;
    if side < 6 {
        match h {
            0 | 3 => 1,
            1 | 4 => 2,
            2 | 5 => 4,
            6 | 10 => 3,
            7 | 11 => 5,
            8 | 12 => 6,
            9 | 13 => 7,
            _ => 0,
        }
    } else {
        if is_post(side) {
            match h {
                0 => return 1,
                1 => return 2,
                2 => return 3,
                _ => {}
            }
        }
        match h {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 3,
            4 => 5,
            5 => 6,
            6 => 7,
            _ => 0,
        }
    }
}

/// Selection bounds `[x1,y1,z1,x2,y2,z2]` of cover slot `part` (`TileCoverable.setPartBounds`).
pub fn part_bounds(part: usize, cov: i32) -> [f32; 6] {
    let th = if thickness_quanta(part, cov) == 0 {
        1.0
    } else {
        thickness_quanta(part, cov) as f32 / 8.0
    };
    let o = 1.0 - th;
    match part {
        0 => [0.0, 0.0, 0.0, 1.0, th, 1.0],
        1 => [0.0, o, 0.0, 1.0, 1.0, 1.0],
        2 => [0.0, 0.0, 0.0, 1.0, 1.0, th],
        3 => [0.0, 0.0, o, 1.0, 1.0, 1.0],
        4 => [0.0, 0.0, 0.0, th, 1.0, 1.0],
        5 => [o, 0.0, 0.0, 1.0, 1.0, 1.0],
        6 => [0.0, 0.0, 0.0, th, th, th],
        7 => [0.0, 0.0, o, th, th, 1.0],
        8 => [o, 0.0, 0.0, 1.0, th, th],
        9 => [o, 0.0, o, 1.0, th, 1.0],
        10 => [0.0, o, 0.0, th, 1.0, th],
        11 => [0.0, o, o, th, 1.0, 1.0],
        12 => [o, o, 0.0, 1.0, 1.0, th],
        13 => [o, o, o, 1.0, 1.0, 1.0],
        14 => [0.0, 0.0, 0.0, 1.0, th, th],
        15 => [0.0, 0.0, o, 1.0, th, 1.0],
        16 => [0.0, 0.0, 0.0, th, th, 1.0],
        17 => [o, 0.0, 0.0, 1.0, th, 1.0],
        18 => [0.0, 0.0, 0.0, th, 1.0, th],
        19 => [0.0, 0.0, o, th, 1.0, 1.0],
        20 => [o, 0.0, 0.0, 1.0, 1.0, th],
        21 => [o, 0.0, o, 1.0, 1.0, 1.0],
        22 => [0.0, o, 0.0, 1.0, 1.0, th],
        23 => [0.0, o, o, 1.0, 1.0, 1.0],
        24 => [0.0, o, 0.0, th, 1.0, 1.0],
        25 => [o, o, 0.0, 1.0, 1.0, 1.0],
        26 => [0.5 - th, 0.0, 0.5 - th, 0.5 + th, 1.0, 0.5 + th],
        27 => [0.5 - th, 0.5 - th, 0.0, 0.5 + th, 0.5 + th, 1.0],
        _ => [0.0, 0.5 - th, 0.5 - th, 1.0, 0.5 + th, 0.5 + th],
    }
}

/// Port of `CoverLib.PlacementValidator`; `covm` is the occupied-slot mask, `covs` the packed values.
struct Validator<'a> {
    sidemask: u32,
    cornermask: u32,
    fillcornermask: u32,
    hollowcornermask: u32,
    covm: u32,
    covs: &'a [i16; COVER_SLOTS],
    q: [i32; COVER_SLOTS],
}

impl<'a> Validator<'a> {
    fn has(&self, slot: usize, ty: i32) -> bool {
        self.covm & (1 << slot) != 0 && (self.covs[slot] as i32) >> 8 == ty
    }
    fn thick_face(&mut self, ty: i32) -> bool {
        for i in 0..6 {
            if self.has(i, ty) {
                let t = cover_to_corner_mask(i);
                if self.fillcornermask & t > 0 {
                    return false;
                }
                self.fillcornermask |= t;
                self.sidemask |= cover_to_strip_mask(i);
            }
        }
        true
    }
    fn thick_side(&mut self, ty: i32) -> bool {
        for i in 0..12 {
            if self.has(i + 14, ty) {
                let t = strip_to_corner_mask(i);
                if self.fillcornermask & t > 0 {
                    return false;
                }
                self.fillcornermask |= t;
                self.sidemask |= 1 << i;
            }
        }
        true
    }
    fn thick_corner(&mut self, ty: i32) -> bool {
        for i in 0..8 {
            if self.has(i + 6, ty) {
                let t = 1 << i;
                if self.fillcornermask & t == t {
                    return false;
                }
                self.fillcornermask |= t;
            }
        }
        true
    }
    fn face(&mut self, ty: i32) -> bool {
        for i in 0..6 {
            if self.has(i, ty) {
                let t = cover_to_corner_mask(i);
                if self.fillcornermask & t == t {
                    return false;
                }
                self.cornermask |= t;
                self.sidemask |= cover_to_strip_mask(i);
            }
        }
        true
    }
    fn side(&mut self, ty: i32) -> bool {
        for i in 0..12 {
            if self.has(i + 14, ty) {
                let t = strip_to_corner_mask(i);
                if self.fillcornermask & t == t || self.sidemask & (1 << i) > 0 {
                    return false;
                }
                self.cornermask |= t;
                self.sidemask |= 1 << i;
            }
        }
        true
    }
    fn corner(&mut self, ty: i32) -> bool {
        for i in 0..8 {
            if self.has(i + 6, ty) {
                let t = 1 << i;
                if self.cornermask & t == t {
                    return false;
                }
                self.cornermask |= t;
            }
        }
        true
    }
    fn hollow(&mut self, ty: i32) -> bool {
        for i in 0..6 {
            if self.has(i, ty) {
                let t = cover_to_corner_mask(i);
                if self.cornermask & t > 0 {
                    return false;
                }
                self.cornermask |= t;
                self.hollowcornermask |= t;
                let s = cover_to_strip_mask(i);
                if self.sidemask & s > 0 {
                    return false;
                }
                self.sidemask |= s;
            }
        }
        true
    }
    fn hollow_cover(&mut self, ty: i32) -> bool {
        let (mut ocm, mut osm) = (0, 0);
        for i in 0..6 {
            if self.has(i, ty) {
                let t = cover_to_corner_mask(i);
                if self.cornermask & t > 0 {
                    return false;
                }
                ocm |= t;
                let s = cover_to_strip_mask(i);
                if self.sidemask & s > 0 {
                    return false;
                }
                osm |= s;
            }
        }
        self.cornermask |= ocm;
        self.sidemask |= osm;
        true
    }
    fn calc_quanta(&mut self) {
        for i in 0..COVER_SLOTS {
            self.q[i] = if self.covm & (1 << i) == 0 {
                0
            } else {
                thickness_quanta(i, self.covs[i] as i32)
            };
        }
    }
    fn overlap(&self, a: usize, b: usize, c: usize, d: usize) -> bool {
        let (a, b, c, d) = (self.q[a], self.q[b], self.q[c], self.q[d]);
        a + b > 8 || a + c > 8 || a + d > 8 || b + c > 8 || b + d > 8 || c + d > 8
    }
    fn impingement_ok(&self) -> bool {
        for i in (0..6).step_by(2) {
            if self.q[i] + self.q[i + 1] > 8 {
                return false;
            }
        }
        let groups = [
            (14, 15, 22, 23),
            (16, 17, 24, 25),
            (18, 19, 20, 22),
            (6, 7, 8, 9),
            (10, 11, 12, 13),
            (6, 8, 10, 12),
            (7, 9, 11, 13),
            (6, 7, 10, 11),
            (8, 9, 12, 13),
        ];
        for (a, b, c, d) in groups {
            if self.overlap(a, b, c, d) {
                return false;
            }
        }
        for i in 0..6 {
            let q1 = self.q[i];
            if q1 == 0 {
                continue;
            }
            let cm = cover_to_corner_mask(i);
            let sm1 = cover_to_strip_mask(i);
            let sm2 = cover_to_strip_mask(i ^ 1);
            for j in 0..8 {
                let q2 = self.q[6 + j];
                if cm & (1 << j) == 0 {
                    if q1 + q2 > 8 {
                        return false;
                    }
                } else if q2 > 0 && q2 < q1 {
                    return false;
                }
            }
            for j in 0..12 {
                let q2 = self.q[14 + j];
                if sm2 & (1 << j) > 0 {
                    if q1 + q2 > 8 {
                        return false;
                    }
                } else if sm1 & (1 << j) > 0 && q2 > 0 && q2 < q1 {
                    return false;
                }
            }
        }
        for i in 0..12 {
            let q1 = self.q[14 + i];
            if q1 == 0 {
                continue;
            }
            let cm = strip_to_corner_mask(i);
            for j in 0..8 {
                let q2 = self.q[6 + j];
                if cm & (1 << j) == 0 {
                    if q1 + q2 > 8 {
                        return false;
                    }
                } else if q2 > 0 && q2 < q1 {
                    return false;
                }
            }
        }
        for i in 0..3 {
            let q1 = self.q[26 + i];
            if q1 == 0 {
                continue;
            }
            for j in 0..8 {
                if q1 + self.q[6 + j] > 4 {
                    return false;
                }
            }
            for j in 0..12 {
                if q1 + self.q[14 + j] > 4 {
                    return false;
                }
            }
            for j in 0..6 {
                if j >> 1 != i && self.q[j] + q1 > 4 {
                    return false;
                }
            }
        }
        true
    }

    fn check(&mut self, cons: u32, jacket: bool) -> bool {
        self.calc_quanta();
        if !self.impingement_ok() {
            return false;
        }
        if !(self.thick_face(9) && self.thick_side(6) && self.thick_corner(6)) {
            return false;
        }
        if !(self.thick_face(8) && self.thick_side(5) && self.thick_corner(5)) {
            return false;
        }
        if !(self.thick_face(7) && self.thick_side(4) && self.thick_corner(4)) {
            return false;
        }
        if self.cornermask > 0 && cons > 0 {
            return false;
        }
        if !(self.thick_face(2) && self.thick_side(2) && self.thick_corner(2)) {
            return false;
        }
        self.cornermask = self.fillcornermask;
        if !(self.face(6) && self.side(3) && self.corner(3)) {
            return false;
        }
        if self.covm & 469762048 > 0 && (jacket || cons > 0) {
            return false;
        }
        for i in 0..6 {
            if cons & (1 << i) != 0 && self.cornermask & cover_to_corner_mask(i) > 0 {
                return false;
            }
        }
        if !(self.face(1) && self.side(1) && self.corner(1)) {
            return false;
        }
        if jacket && (self.cornermask > 0 || self.sidemask > 0) {
            return false;
        }
        for ty in [13, 12, 11, 10, 5] {
            if !self.hollow(ty) {
                return false;
            }
        }
        for i in 0..6 {
            if cons & (1 << i) != 0 && self.hollowcornermask & cover_to_corner_mask(i) > 0 {
                return false;
            }
        }
        if !(self.hollow(4)
            && self.hollow_cover(3)
            && self.face(0)
            && self.side(0)
            && self.corner(0))
        {
            return false;
        }
        for i in 0..12 {
            if self.covm & (1 << (i + 14)) != 0 {
                let t = strip_to_cover_mask(i);
                if cons & t == t {
                    return false;
                }
            }
        }
        true
    }
}

/// `CoverLib.checkPlacement`: may these covers coexist with connections `cons` (face mask)?
pub fn check_placement(covm: u32, covs: &[i16; COVER_SLOTS], cons: u32, jacket: bool) -> bool {
    Validator {
        sidemask: 0,
        cornermask: 0,
        fillcornermask: 0,
        hollowcornermask: 0,
        covm,
        covs,
        q: [0; COVER_SLOTS],
    }
    .check(cons, jacket)
}

/// Registered cover material (`CoverLib.addMaterial`).
#[derive(Clone, Debug, PartialEq)]
pub struct CoverMaterial {
    pub block: String,
    pub meta: u16,
    pub name: String,
    pub desc: String,
    /// Minimum saw sharpness needed to cut it.
    pub hardness: i32,
    pub transparent: bool,
    /// Mining hardness; negative means unbreakable.
    pub mining_hardness: f32,
}

/// Material table indexed by the low byte of a cover value.
#[derive(Default, Clone, Debug)]
pub struct CoverLib {
    materials: HashMap<u8, CoverMaterial>,
    index: HashMap<(String, u16), u8>,
}

impl CoverLib {
    pub fn add_material(&mut self, n: u8, m: CoverMaterial) {
        self.index.insert((m.block.clone(), m.meta), n);
        self.materials.insert(n, m);
    }
    pub fn material(&self, n: u8) -> Option<&CoverMaterial> {
        self.materials.get(&n)
    }
    /// Material number for a block/meta pair.
    pub fn material_for(&self, block: &str, meta: u16) -> Option<u8> {
        self.index.get(&(block.to_string(), meta)).copied()
    }
    pub fn hardness(&self, n: u8) -> i32 {
        self.material(n).map_or(0, |m| m.hardness)
    }
    pub fn is_transparent(&self, n: u8) -> bool {
        self.material(n).is_some_and(|m| m.transparent)
    }
    pub fn mining_hardness(&self, n: u8) -> f32 {
        self.material(n).map_or(0.0, |m| m.mining_hardness)
    }
}

/// `TileCoverable.isSideNormal` shape test: only full-face thickness classes count.
pub fn cover_is_normal(cov: i32, transparent: bool) -> bool {
    let n = cov >> 8;
    !transparent && (n < 3 || (6..=9).contains(&n))
}

/// `TileCoverable.getPartStrength`; `strength` and `can_harvest` come from the engine.
pub fn part_strength(mining_hardness: f32, can_harvest: bool, strength_vs_block: f32) -> f32 {
    if mining_hardness < 0.0 {
        0.0
    } else if !can_harvest {
        1.0 / mining_hardness / 100.0
    } else {
        strength_vs_block / mining_hardness / 30.0
    }
}

/// Cover store of a multipart block (`TileCovered`).
#[derive(Clone, Debug, PartialEq)]
pub struct TileCovered {
    pub cover_sides: u32,
    pub covers: [i16; COVER_SLOTS],
    /// True after a change; the engine layer clears it when it re-sends/re-renders the tile.
    pub dirty: bool,
}

impl Default for TileCovered {
    fn default() -> Self {
        Self {
            cover_sides: 0,
            covers: [0; COVER_SLOTS],
            dirty: false,
        }
    }
}

impl TileCovered {
    pub fn block_empty(&self) -> bool {
        self.cover_sides == 0
    }

    /// Item stacks dropped when the whole block is harvested; `convert` maps (slot, cover) to a plate item.
    pub fn harvest_contents(
        &self,
        convert: impl Fn(usize, i32) -> Option<ItemStack>,
    ) -> Vec<ItemStack> {
        (0..COVER_SLOTS)
            .filter(|&i| self.cover_sides & (1 << i) != 0)
            .filter_map(|i| convert(i, self.covers[i] as i32))
            .collect()
    }

    /// Bounds of an occupied part.
    pub fn part_bounds(&self, part: usize) -> Option<[f32; 6]> {
        (part < COVER_SLOTS && self.cover_sides & (1 << part) != 0)
            .then(|| part_bounds(part, self.covers[part] as i32))
    }

    /// NBT body: `cvm` and `cvs` (little-endian shorts for each set slot).
    pub fn to_nbt(&self) -> (i32, Vec<u8>) {
        let mut b = Vec::new();
        for i in 0..COVER_SLOTS {
            if self.cover_sides & (1 << i) != 0 {
                b.extend_from_slice(&(self.covers[i] as u16).to_le_bytes());
            }
        }
        (self.cover_sides as i32, b)
    }

    pub fn from_nbt(cvm: i32, cvs: &[u8]) -> Self {
        let mut t = Self::default();
        let m = cvm as u32 & SLOT_MASK;
        t.cover_sides = m;
        let mut sp = 0;
        for i in 0..COVER_SLOTS {
            if m & (1 << i) != 0 {
                if sp + 2 > cvs.len() {
                    break;
                }
                t.covers[i] = u16::from_le_bytes([cvs[sp], cvs[sp + 1]]) as i16;
                sp += 2;
            }
        }
        t
    }

    /// Tile description packet body (UVLC values; mask then each cover).
    pub fn write_packet(&self, out: &mut Vec<u8>) {
        super::foundation::write_uvlc(out, self.cover_sides as u64);
        for i in 0..COVER_SLOTS {
            if self.cover_sides & (1 << i) != 0 {
                super::foundation::write_uvlc(out, self.covers[i] as u16 as u64);
            }
        }
    }

    pub fn read_packet(r: &mut super::foundation::PacketReader) -> Result<Self, PacketError> {
        let mut t = Self {
            cover_sides: r.uvlc()? as u32 & SLOT_MASK,
            ..Self::default()
        };
        for i in 0..COVER_SLOTS {
            if t.cover_sides & (1 << i) != 0 {
                t.covers[i] = r.uvlc()? as u16 as i16;
            }
        }
        Ok(t)
    }
}

impl Coverable for TileCovered {
    fn can_add_cover(&self, side: i32, cover: i32) -> bool {
        if !(0..COVER_SLOTS as i32).contains(&side) || self.cover_sides & (1 << side) > 0 {
            return false;
        }
        let mut test = self.covers;
        test[side as usize] = cover as i16;
        check_placement(self.cover_sides | 1 << side, &test, 0, false)
    }
    fn try_add_cover(&mut self, side: i32, cover: i32) -> bool {
        if !self.can_add_cover(side, cover) {
            return false;
        }
        self.cover_sides |= 1 << side;
        self.covers[side as usize] = cover as i16;
        self.dirty = true;
        true
    }
    fn try_remove_cover(&mut self, side: i32) -> i32 {
        if !(0..COVER_SLOTS as i32).contains(&side) || self.cover_sides & (1 << side) == 0 {
            return -1;
        }
        self.cover_sides &= !(1 << side);
        let r = self.covers[side as usize] as i32;
        self.covers[side as usize] = 0;
        self.dirty = true;
        r
    }
    fn cover(&self, side: i32) -> i32 {
        if !(0..COVER_SLOTS as i32).contains(&side) || self.cover_sides & (1 << side) == 0 {
            -1
        } else {
            self.covers[side as usize] as i32
        }
    }
    fn cover_mask(&self) -> u32 {
        self.cover_sides
    }
}

impl Multipart for TileCovered {
    fn is_side_solid(&self, side: i32) -> bool {
        (0..32).contains(&side) && self.cover_sides & (1 << side) > 0
    }
    /// Transparency needs the material table, so this treats covers as opaque; use
    /// [`cover_is_normal`] with a [`CoverLib`] for the exact answer.
    fn is_side_normal(&self, side: i32) -> bool {
        self.is_side_solid(side) && cover_is_normal(self.cover(side), false)
    }
    fn harvest_multipart(&mut self) -> Vec<ItemStack> {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------------------------
// BlockExtended / BlockMultipart / BlockCoverable / TileAppliance
// ---------------------------------------------------------------------------------------------

/// `BlockExtended.tileEntityMap`: sub-tile metadata (0..16) to a tile kind.
#[derive(Clone, Debug, PartialEq)]
pub struct TileMap<K: Clone> {
    kinds: [Option<K>; 16],
}

impl<K: Clone> Default for TileMap<K> {
    fn default() -> Self {
        Self {
            kinds: std::array::from_fn(|_| None),
        }
    }
}

impl<K: Clone> TileMap<K> {
    /// `addTileEntityMapping(md, cl)`; out-of-range metadata is ignored.
    pub fn add_tile_entity_mapping(&mut self, md: usize, kind: K) {
        if md < 16 {
            self.kinds[md] = Some(kind);
        }
    }
    /// `createTileEntity(world, md)`: `None` when nothing is mapped.
    pub fn create_tile_entity(&self, md: usize) -> Option<K> {
        self.kinds.get(md).cloned().flatten()
    }
}

/// Axis-aligned box `[min_x, min_y, min_z, max_x, max_y, max_z]` in block-local or world space.
pub type Aabb = [f64; 6];

fn offset(b: [f32; 6], x: i32, y: i32, z: i32) -> Aabb {
    [
        b[0] as f64 + x as f64,
        b[1] as f64 + y as f64,
        b[2] as f64 + z as f64,
        b[3] as f64 + x as f64,
        b[4] as f64 + y as f64,
        b[5] as f64 + z as f64,
    ]
}

/// Result of a ray test against one box: hit point, hit face (0..6, MC side order) and distance^2.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub point: [f64; 3],
    pub side: usize,
    pub dist_sq: f64,
}

/// Segment `src -> dst` against a box (`Block.collisionRayTrace`); the nearest entry point.
pub fn ray_box(b: &Aabb, src: [f64; 3], dst: [f64; 3]) -> Option<RayHit> {
    let d = [dst[0] - src[0], dst[1] - src[1], dst[2] - src[2]];
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    let mut side = usize::MAX;
    for a in 0..3 {
        let (lo, hi) = (b[a], b[a + 3]);
        if d[a].abs() < 1e-12 {
            if src[a] < lo || src[a] > hi {
                return None;
            }
            continue;
        }
        let (mut ta, mut tb) = ((lo - src[a]) / d[a], (hi - src[a]) / d[a]);
        // faces: axis 0 -> 4 (-x) / 5 (+x); axis 1 -> 0 / 1; axis 2 -> 2 / 3
        let (mut fa, mut fb) = match a {
            0 => (4, 5),
            1 => (0, 1),
            _ => (2, 3),
        };
        if ta > tb {
            std::mem::swap(&mut ta, &mut tb);
            std::mem::swap(&mut fa, &mut fb);
        }
        if ta > t0 {
            t0 = ta;
            side = fa;
        }
        t1 = t1.min(tb);
        if t0 > t1 {
            return None;
        }
    }
    let point = [src[0] + d[0] * t0, src[1] + d[1] * t0, src[2] + d[2] * t0];
    let dist_sq = (0..3).map(|i| (point[i] - src[i]).powi(2)).sum();
    // A segment that starts inside the box has no entry face; vanilla reports no hit.
    (side != usize::MAX).then_some(RayHit {
        point,
        side,
        dist_sq,
    })
}

/// What a multipart tile exposes to `BlockMultipart` (`TileMultipart` / `TileCoverable`).
pub trait MultipartTile {
    /// Bit per part that can be hit / highlighted.
    fn parts_mask(&self) -> u32;
    /// Bit per part that collides.
    fn solid_parts_mask(&self) -> u32;
    /// Local bounds of a part.
    fn part_bounds(&self, part: usize) -> Option<[f32; 6]>;
    fn part_strength(&self, part: usize) -> f32;
    /// `onHarvestPart`: remove the part and return its drops; empties report through `block_empty`.
    fn harvest_part(&mut self, part: usize) -> Vec<ItemStack>;
    fn block_empty(&self) -> bool;
    /// `getBlockDropped` for the whole block.
    fn harvest_contents(&self) -> Vec<ItemStack>;
}

/// `BlockMultipart.a(world,x,y,z,ray)`: nearest part along the ray; ties keep the first part.
pub fn trace_parts<T: MultipartTile>(
    t: &T,
    pos: [i32; 3],
    src: [f64; 3],
    dst: [f64; 3],
) -> Option<(usize, RayHit)> {
    let mut best: Option<(usize, RayHit)> = None;
    let mut pm = t.parts_mask();
    while pm > 0 {
        let pt = pm.trailing_zeros() as usize;
        pm &= !(1 << pt);
        let Some(b) = t.part_bounds(pt) else { continue };
        let bx = offset(b, pos[0], pos[1], pos[2]);
        if let Some(h) = ray_box(&bx, src, dst) {
            if best.as_ref().is_none_or(|(_, p)| h.dist_sq < p.dist_sq) {
                best = Some((pt, h));
            }
        }
    }
    best
}

fn boxes_overlap(a: &Aabb, b: &Aabb) -> bool {
    (0..3).all(|i| a[i] < b[i + 3] && a[i + 3] > b[i])
}

/// `computeCollidingBoxes`: every solid part whose box overlaps `query` (world space).
pub fn colliding_boxes<T: MultipartTile>(t: &T, pos: [i32; 3], query: &Aabb) -> Vec<Aabb> {
    let mut out = Vec::new();
    let mut pm = t.solid_parts_mask();
    while pm > 0 {
        let pt = pm.trailing_zeros() as usize;
        pm &= !(1 << pt);
        if let Some(b) = t.part_bounds(pt) {
            let bx = offset(b, pos[0], pos[1], pos[2]);
            if boxes_overlap(&bx, query) {
                out.push(bx);
            }
        }
    }
    out
}

/// Outcome of `removeBlockByPlayer`.
#[derive(Clone, Debug, PartialEq)]
pub enum RemoveOutcome {
    /// Client side: report success, change nothing.
    Client,
    /// No part under the cursor: vanilla keeps the block.
    Nothing,
    /// One part came off; the block stays unless `block_removed`.
    Part {
        part: usize,
        drops: Vec<ItemStack>,
        block_removed: bool,
    },
}

/// `BlockMultipart.removeBlockByPlayer`: harvest only the part the player is looking at. The block
/// is never removed by the engine itself (Java returns false); `block_removed` is set when the
/// last part left and `TileCoverable.deleteBlock` clears it.
pub fn remove_block_by_player<T: MultipartTile>(
    t: &mut T,
    pos: [i32; 3],
    src: [f64; 3],
    dst: [f64; 3],
    is_client: bool,
) -> RemoveOutcome {
    if is_client {
        return RemoveOutcome::Client;
    }
    let Some((part, _)) = trace_parts(t, pos, src, dst) else {
        return RemoveOutcome::Nothing;
    };
    let drops = t.harvest_part(part);
    RemoveOutcome::Part {
        part,
        drops,
        block_removed: t.block_empty(),
    }
}

/// `BlockExtended.removeBlockByPlayer` (non-multipart tiles): drops only when the player can
/// harvest and is not in creative; the block is always removed.
pub fn extended_drops(
    contents: Vec<ItemStack>,
    can_harvest: bool,
    creative: bool,
) -> Vec<ItemStack> {
    if can_harvest && !creative {
        contents
    } else {
        Vec::new()
    }
}

/// `BlockMultipart.getBlockDropped` (via `BlockExtended`): all contents of the tile.
pub fn get_block_dropped<T: MultipartTile>(t: &T) -> Vec<ItemStack> {
    t.harvest_contents()
}

/// `BlockCoverable.isBlockSolidOnSide`.
pub fn is_block_solid_on_side(tile: Option<&dyn Multipart>, side: usize) -> bool {
    tile.is_some_and(|t| t.is_side_normal(side as i32))
}

/// `BlockCoverable.getExplosionResistance`: the struck cover's block resistance when the ray from
/// the explosion hits a part, else the block default.
pub fn explosion_resistance<T: MultipartTile>(
    t: Option<&T>,
    pos: [i32; 3],
    src: [f64; 3],
    block_default: f32,
    cover_resistance: impl Fn(usize) -> Option<f32>,
) -> f32 {
    let Some(t) = t else { return block_default };
    let end = [
        pos[0] as f64 + 0.5,
        pos[1] as f64 + 0.5,
        pos[2] as f64 + 0.5,
    ];
    match trace_parts(t, pos, src, end) {
        None => block_default,
        Some((part, _)) => cover_resistance(part).unwrap_or(block_default),
    }
}

impl TileCovered {
    /// `TileCoverable.getPartStrength`: `cover_hardness` is the material's mining hardness.
    pub fn part_strength_of(
        &self,
        part: usize,
        lib: &CoverLib,
        can_harvest: bool,
        strength_vs_block: f32,
    ) -> f32 {
        if part >= COVER_SLOTS || self.cover_sides & (1 << part) == 0 {
            return 0.0;
        }
        let m = (self.covers[part] & 0xFF) as u8;
        self.cover_strength(lib, m, can_harvest, strength_vs_block)
    }

    fn cover_strength(&self, lib: &CoverLib, m: u8, can_harvest: bool, svb: f32) -> f32 {
        part_strength(lib.mining_hardness(m), can_harvest, svb)
    }

    /// `onHarvestPart`: take the cover off, returning its (slot, cover) for the caller to convert
    /// into a plate item.
    pub fn harvest_part_raw(&mut self, part: usize) -> Option<(usize, i32)> {
        let c = self.try_remove_cover(part as i32);
        (c >= 0).then_some((part, c))
    }
}

/// Adapter binding a [`TileCovered`] to a [`CoverLib`] and plate conversion so it satisfies
/// [`MultipartTile`].
pub struct CoveredView<'a, F: Fn(usize, i32) -> Option<ItemStack>> {
    pub tile: &'a mut TileCovered,
    pub lib: &'a CoverLib,
    pub convert: F,
    pub can_harvest: bool,
    pub strength_vs_block: f32,
}

impl<F: Fn(usize, i32) -> Option<ItemStack>> MultipartTile for CoveredView<'_, F> {
    fn parts_mask(&self) -> u32 {
        self.tile.cover_sides
    }
    fn solid_parts_mask(&self) -> u32 {
        self.tile.cover_sides
    }
    fn part_bounds(&self, part: usize) -> Option<[f32; 6]> {
        self.tile.part_bounds(part)
    }
    fn part_strength(&self, part: usize) -> f32 {
        self.tile
            .part_strength_of(part, self.lib, self.can_harvest, self.strength_vs_block)
    }
    fn harvest_part(&mut self, part: usize) -> Vec<ItemStack> {
        match self.tile.harvest_part_raw(part) {
            Some((s, c)) => (self.convert)(s, c).into_iter().collect(),
            None => Vec::new(),
        }
    }
    fn block_empty(&self) -> bool {
        self.tile.block_empty()
    }
    fn harvest_contents(&self) -> Vec<ItemStack> {
        self.tile.harvest_contents(&self.convert)
    }
}

/// Sub id of the appliance tile description packet.
pub const APPLIANCE_SUB_ID: u8 = 7;

/// Light level of an active appliance.
pub const APPLIANCE_ACTIVE_LIGHT: u8 = 13;

/// `TileAppliance`: yaw-facing (0..3) and an active flag, carried by frames and packets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileAppliance {
    pub rotation: u8,
    pub active: bool,
}

impl TileAppliance {
    /// `onBlockPlaced`: `floor(yaw * 4 / 360 + 0.5) & 3`.
    pub fn on_block_placed(&mut self, player_yaw: f32) {
        self.rotation = ((player_yaw * 4.0 / 360.0 + 0.5).floor() as i32 & 3) as u8;
    }
    pub fn light_value(&self) -> u8 {
        if self.active {
            APPLIANCE_ACTIVE_LIGHT
        } else {
            0
        }
    }
    /// NBT bytes `(ps, rot)`.
    pub fn to_nbt(&self) -> (u8, u8) {
        (self.active as u8, self.rotation)
    }
    pub fn from_nbt(ps: u8, rot: u8) -> Self {
        Self {
            rotation: rot,
            active: ps > 0,
        }
    }
    /// `writeToPacket` body: rotation, state.
    pub fn write_body(&self) -> Vec<u8> {
        vec![self.rotation, self.active as u8]
    }
    pub fn read_body(&mut self, body: &[u8]) -> std::io::Result<()> {
        match body {
            [r, ps, ..] => {
                self.rotation = *r;
                self.active = *ps > 0;
                Ok(())
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "short appliance packet",
            )),
        }
    }
    /// Tile description packet (sub id 7) for the client.
    pub fn tile_packet(&self, x: i32, y: i32, z: i32) -> super::traits::Packet211TileDesc {
        super::traits::Packet211TileDesc {
            sub_id: APPLIANCE_SUB_ID,
            x,
            y,
            z,
            body: self.write_body(),
        }
    }
    /// `handlePacket`: ignores other sub ids; true when the tile needs a re-render.
    pub fn handle_packet(&mut self, p: &super::traits::Packet211TileDesc) -> bool {
        if p.sub_id != APPLIANCE_SUB_ID {
            return false;
        }
        let _ = self.read_body(&p.body);
        true
    }
}

impl super::traits::FrameSupport for TileAppliance {
    /// `getFramePacket`: sub id byte then the body.
    fn frame_packet(&self) -> Vec<u8> {
        let mut v = vec![APPLIANCE_SUB_ID];
        v.extend(self.write_body());
        v
    }
    fn handle_frame_packet(&mut self, data: &[u8]) -> std::io::Result<()> {
        match data.split_first() {
            Some((_, rest)) => self.read_body(rest),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "empty frame packet",
            )),
        }
    }
    fn on_frame_refresh(&mut self) {}
    fn on_frame_pickup(&mut self) {}
    fn on_frame_drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thickness_table() {
        assert_eq!(thickness_quanta(0, 0), 1);
        assert_eq!(thickness_quanta(0, 9 << 8), 7);
        assert_eq!(thickness_quanta(26, 2 << 8), 3);
        assert_eq!(thickness_quanta(10, 6 << 8), 7);
        assert_eq!(thickness(0, 2 << 8), 0.5);
        assert_eq!(thickness(0, 14 << 8), 1.0);
    }

    #[test]
    fn opposite_faces_fit_until_full() {
        let mut t = TileCovered::default();
        assert!(t.try_add_cover(0, 2 << 8)); // half slab bottom
        assert!(t.try_add_cover(1, 2 << 8)); // half slab top: 4+4 ok
        let mut u = TileCovered::default();
        assert!(u.try_add_cover(0, 2 << 8));
        assert!(u.can_add_cover(1, 6 << 8)); // 4+3 fits
    }

    #[test]
    fn impinging_faces_rejected() {
        let mut t = TileCovered::default();
        assert!(t.try_add_cover(0, 2 << 8));
        assert!(!t.can_add_cover(1, 7 << 8)); // 4+5 > 8
        assert!(!t.can_add_cover(0, 0)); // occupied
    }

    #[test]
    fn add_remove_round_trip() {
        let mut t = TileCovered::default();
        assert!(t.try_add_cover(3, 5));
        assert_eq!(t.cover(3), 5);
        assert_eq!(t.cover_mask(), 8);
        assert_eq!(t.try_remove_cover(3), 5);
        assert_eq!(t.try_remove_cover(3), -1);
        assert!(t.block_empty());
    }

    #[test]
    fn nbt_and_packet_round_trip() {
        let mut t = TileCovered::default();
        t.try_add_cover(0, 0x0103);
        t.try_add_cover(27, 1 << 8 | 7);
        let (m, b) = t.to_nbt();
        let u = TileCovered::from_nbt(m, &b);
        assert_eq!(u.covers, t.covers);
        let mut buf = Vec::new();
        t.write_packet(&mut buf);
        let v = TileCovered::read_packet(&mut super::super::foundation::PacketReader::new(&buf))
            .unwrap();
        assert_eq!(v.cover_sides, t.cover_sides);
        assert_eq!(v.covers, t.covers);
    }

    #[test]
    fn bounds() {
        assert_eq!(part_bounds(1, 2 << 8), [0.0, 0.5, 0.0, 1.0, 1.0, 1.0]);
        assert_eq!(part_bounds(26, 0), [0.375, 0.0, 0.375, 0.625, 1.0, 0.625]);
    }

    #[test]
    fn mask_tables_consistent() {
        for c in 0..8 {
            for f in 0..6 {
                let touches = corner_to_cover_mask(c) & (1 << f) != 0;
                assert_eq!(touches, cover_to_corner_mask(f) & (1 << c) != 0);
            }
        }
        assert_eq!(cover_value_to_damage(0, 2 << 8), 4352);
        assert_eq!(damage_to_cover_data(16 << 8 | 3), 131328 | 3);
    }

    fn slab_tile() -> TileCovered {
        let mut t = TileCovered::default();
        assert!(t.try_add_cover(0, 2 << 8)); // bottom half slab
        assert!(t.try_add_cover(1, 0)); // top thin plate
        t
    }

    #[test]
    fn tile_map_dispatch() {
        let mut m: TileMap<&str> = TileMap::default();
        m.add_tile_entity_mapping(3, "furnace");
        assert_eq!(m.create_tile_entity(3), Some("furnace"));
        assert_eq!(m.create_tile_entity(4), None);
        assert_eq!(m.create_tile_entity(99), None);
    }

    #[test]
    fn raytrace_picks_nearest_part() {
        let mut t = slab_tile();
        let lib = CoverLib::default();
        let v = CoveredView {
            tile: &mut t,
            lib: &lib,
            convert: |_, _| None,
            can_harvest: true,
            strength_vs_block: 1.0,
        };
        // from above, straight down: top plate (part 1) first
        let h = trace_parts(&v, [0, 0, 0], [0.5, 2.0, 0.5], [0.5, -1.0, 0.5]).unwrap();
        assert_eq!(h.0, 1);
        assert_eq!(h.1.side, 1);
        // from the side at mid height only the bottom slab (y<0.5) is hit
        let h = trace_parts(&v, [0, 0, 0], [-1.0, 0.25, 0.5], [2.0, 0.25, 0.5]).unwrap();
        assert_eq!(h.0, 0);
        assert_eq!(h.1.side, 4);
        assert!(trace_parts(&v, [0, 0, 0], [-1.0, 0.7, 0.5], [2.0, 0.7, 0.5]).is_none());
    }

    #[test]
    fn colliding_boxes_filter_and_offset() {
        let mut t = slab_tile();
        let lib = CoverLib::default();
        let v = CoveredView {
            tile: &mut t,
            lib: &lib,
            convert: |_, _| None,
            can_harvest: true,
            strength_vs_block: 1.0,
        };
        let q = [5.0, 3.0, 5.0, 6.0, 3.4, 6.0];
        let hits = colliding_boxes(&v, [5, 3, 5], &q);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0][4], 3.5);
        assert!(colliding_boxes(&v, [5, 3, 5], &[5.0, 3.6, 5.0, 6.0, 3.8, 6.0]).is_empty());
    }

    #[test]
    fn player_removal_takes_one_part() {
        let mut t = slab_tile();
        let lib = CoverLib::default();
        let conv = |s: usize, c: i32| Some(ItemStack::new(format!("cover_{s}_{c}"), 1));
        let mut v = CoveredView {
            tile: &mut t,
            lib: &lib,
            convert: conv,
            can_harvest: true,
            strength_vs_block: 1.0,
        };
        let (s, d) = ([0.5, 2.0, 0.5], [0.5, -1.0, 0.5]);
        assert_eq!(
            remove_block_by_player(&mut v, [0, 0, 0], s, d, true),
            RemoveOutcome::Client
        );
        match remove_block_by_player(&mut v, [0, 0, 0], s, d, false) {
            RemoveOutcome::Part {
                part,
                drops,
                block_removed,
            } => {
                assert_eq!(part, 1);
                assert_eq!(drops.len(), 1);
                assert!(!block_removed);
            }
            o => panic!("{o:?}"),
        }
        assert_eq!(get_block_dropped(&v).len(), 1);
        match remove_block_by_player(&mut v, [0, 0, 0], s, d, false) {
            RemoveOutcome::Part {
                part: 0,
                block_removed: true,
                ..
            } => {}
            o => panic!("{o:?}"),
        }
        assert_eq!(
            remove_block_by_player(&mut v, [0, 0, 0], s, d, false),
            RemoveOutcome::Nothing
        );
    }

    #[test]
    fn extended_drops_rules() {
        let s = vec![ItemStack::new("x".to_string(), 1)];
        assert_eq!(extended_drops(s.clone(), true, false).len(), 1);
        assert!(extended_drops(s.clone(), true, true).is_empty());
        assert!(extended_drops(s, false, false).is_empty());
    }

    #[test]
    fn solid_on_side_and_explosion() {
        let t = slab_tile();
        assert!(is_block_solid_on_side(Some(&t), 0));
        assert!(!is_block_solid_on_side(Some(&t), 2));
        assert!(!is_block_solid_on_side(None, 0));
        let mut t2 = slab_tile();
        let lib = CoverLib::default();
        let v = CoveredView {
            tile: &mut t2,
            lib: &lib,
            convert: |_, _| None,
            can_harvest: true,
            strength_vs_block: 1.0,
        };
        let r = explosion_resistance(Some(&v), [0, 0, 0], [0.5, 5.0, 0.5], 1.0, |p| {
            Some(10.0 + p as f32)
        });
        assert_eq!(r, 11.0);
        let mut e = TileCovered::default();
        let ev = CoveredView {
            tile: &mut e,
            lib: &lib,
            convert: |_, _| None,
            can_harvest: true,
            strength_vs_block: 1.0,
        };
        assert_eq!(
            explosion_resistance(Some(&ev), [0, 0, 0], [9.0, 9.0, 9.0], 1.0, |_| Some(5.0)),
            1.0
        );
        assert_eq!(
            explosion_resistance(
                None::<&CoveredView<fn(usize, i32) -> Option<ItemStack>>>,
                [0, 0, 0],
                [9.0; 3],
                2.0,
                |_| None
            ),
            2.0
        );
    }

    #[test]
    fn appliance_placement_packets_frames() {
        use super::super::traits::FrameSupport;
        let mut a = TileAppliance::default();
        a.on_block_placed(90.0);
        assert_eq!(a.rotation, 1);
        a.on_block_placed(-90.0);
        assert_eq!(a.rotation, 3);
        a.active = true;
        assert_eq!(a.light_value(), 13);
        let f = a.frame_packet();
        assert_eq!(f, vec![7, 3, 1]);
        let mut b = TileAppliance::default();
        b.handle_frame_packet(&f).unwrap();
        assert_eq!(b, a);
        assert!(b.handle_frame_packet(&[7]).is_err());
        let p = a.tile_packet(1, 2, 3);
        let mut c = TileAppliance::default();
        assert!(
            c.handle_packet(&super::super::traits::Packet211TileDesc::decode(&p.encode()).unwrap())
        );
        assert_eq!(c, a);
        let (ps, rot) = a.to_nbt();
        assert_eq!(TileAppliance::from_nbt(ps, rot), a);
    }
}
