//! Decorative stones: marble, basalt and their brick variants, plus the marble and
//! volcano generators' pure fill logic.
//!
//! Java reference: `eloraam/world/{BlockCustomStone,WorldGenMarble,WorldGenVolcano}.java`.
//! World access is passed as closures; randomness as a closure `rng(bound) -> 0..bound`.

use std::collections::{HashMap, HashSet, VecDeque};

/// The seven metadata values of `BlockCustomStone`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoneKind {
    Marble = 0,
    Basalt = 1,
    MarbleBrick = 2,
    BasaltCobblestone = 3,
    BasaltBrick = 4,
    BasaltChiseled = 5,
    BasaltPaver = 6,
}

pub const STONE_KINDS: [StoneKind; 7] = [
    StoneKind::Marble,
    StoneKind::Basalt,
    StoneKind::MarbleBrick,
    StoneKind::BasaltCobblestone,
    StoneKind::BasaltBrick,
    StoneKind::BasaltChiseled,
    StoneKind::BasaltPaver,
];

impl StoneKind {
    pub fn from_meta(md: u8) -> Option<Self> {
        STONE_KINDS.get(md as usize).copied()
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Marble => "redpower_2:marble",
            Self::Basalt => "redpower_2:basalt",
            Self::MarbleBrick => "redpower_2:marble_brick",
            Self::BasaltCobblestone => "redpower_2:basalt_cobblestone",
            Self::BasaltBrick => "redpower_2:basalt_brick",
            Self::BasaltChiseled => "redpower_2:basalt_chiseled",
            Self::BasaltPaver => "redpower_2:basalt_paver",
        }
    }

    /// Hardness (`getPrototypicalHardness`): marble kinds 1.0, basalt kinds 2.5.
    pub fn hardness(self) -> f32 {
        match self {
            Self::Marble | Self::MarbleBrick => 1.0,
            _ => 2.5,
        }
    }

    /// Blast resistance (`getExplosionResistance`): 12.0 for basalt kinds, 6.0 for marble.
    pub fn blast_resistance(self) -> f32 {
        match self {
            Self::Marble | Self::MarbleBrick => 6.0,
            _ => 12.0,
        }
    }

    /// Kind dropped when mined (`b(int)`): basalt and paver drop cobblestone.
    pub fn drop_kind(self) -> StoneKind {
        match self {
            Self::Basalt | Self::BasaltPaver => Self::BasaltCobblestone,
            k => k,
        }
    }
}

/// Hardness for a raw metadata value; unknown values use 3.0.
pub fn hardness_for_meta(md: u8) -> f32 {
    StoneKind::from_meta(md).map_or(3.0, StoneKind::hardness)
}

/// Blast resistance for a raw metadata value; unknown values use 6.0.
pub fn blast_resistance_for_meta(md: u8) -> f32 {
    StoneKind::from_meta(md).map_or(6.0, StoneKind::blast_resistance)
}

/// Marble pockets per chunk and their Y band.
pub const MARBLE_POCKETS_PER_CHUNK: u32 = 4;
pub const MARBLE_MIN_Y: i32 = 32;
pub const MARBLE_MAX_Y: i32 = 63;
/// Marble search ceiling when looking up for stone.
pub const MARBLE_SEARCH_CEILING: i32 = 96;

/// Marble pocket fill (`WorldGenMarble#a`). From `(x,y,z)` (which must be air) go up until
/// stone is found, then breadth-first fill stone with marble, up to `count` blocks. A stone
/// block touching air resets the spread budget to 6. Returns the positions to turn into
/// marble, or `None` when the start is not air or no stone is within the ceiling.
pub fn marble_fill(
    start: (i32, i32, i32),
    count: u32,
    block_at: impl Fn(i32, i32, i32) -> BlockClass,
) -> Option<Vec<(i32, i32, i32)>> {
    let (x, y, z) = start;
    if block_at(x, y, z) != BlockClass::Air {
        return None;
    }
    let mut l = y;
    while block_at(x, l, z) != BlockClass::Stone {
        l += 1;
        if l > MARBLE_SEARCH_CEILING {
            return None;
        }
    }
    let mut queue: VecDeque<(i32, i32, i32, i32)> = VecDeque::new();
    let mut seen: HashSet<(i32, i32, i32)> = HashSet::new();
    let mut out = Vec::new();
    let mut remaining = count;
    seen.insert((x, l, z));
    queue.push_back((x, l, z, 6));
    while remaining > 0 {
        let Some((px, py, pz, p)) = queue.pop_front() else {
            break;
        };
        if block_at(px, py, pz) != BlockClass::Stone {
            continue;
        }
        out.push((px, py, pz));
        let neighbours = [
            (px - 1, py, pz),
            (px + 1, py, pz),
            (px, py - 1, pz),
            (px, py + 1, pz),
            (px, py, pz - 1),
            (px, py, pz + 1),
        ];
        if p > 0 {
            let touches_air = neighbours
                .iter()
                .any(|&(a, b, c)| block_at(a, b, c) == BlockClass::Air);
            let np = if touches_air { 6 } else { p - 1 };
            for n in neighbours {
                if seen.insert(n) {
                    queue.push_back((n.0, n.1, n.2, np));
                }
            }
        }
        remaining -= 1;
    }
    Some(out)
}

/// Coarse block classes the generators care about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockClass {
    Air,
    Stone,
    Other,
}

/// Volcano fill-queue bookkeeping (`addBlock`/`searchBlock`): a column is queued only when
/// its power exceeds the best seen so far.
#[derive(Default)]
pub struct VolcanoSpread {
    pub queue: VecDeque<(i32, i32, i32)>,
    pub power: HashMap<(i32, i32), i32>,
}

impl VolcanoSpread {
    pub fn add(&mut self, x: i32, y: i32, z: i32, p: i32) {
        if p > 0 && self.power.get(&(x, z)).is_none_or(|&o| p > o) {
            self.queue.push_back((x, y, z));
            self.power.insert((x, z), p);
        }
    }

    /// Spread to the four horizontal neighbours; each loses one power when its bit in a
    /// 4-bit roll (`rng(16)`) is set.
    pub fn search(&mut self, x: i32, y: i32, z: i32, p: i32, rng: &mut impl FnMut(u32) -> u32) {
        let rp = rng(16);
        let q = |bit: u32| if rp & bit > 0 { p - 1 } else { p };
        self.add(x - 1, y, z, q(1));
        self.add(x + 1, y, z, q(2));
        self.add(x, y, z - 1, q(4));
        self.add(x, y, z + 1, q(8));
    }
}

/// Volcano cone parameters: head power 3..=6, spread 0..=2.
pub fn volcano_params(rng: &mut impl FnMut(u32) -> u32) -> (i32, i32) {
    let head = 3 + rng(4) as i32;
    let spread = rng(3) as i32;
    (head, spread)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stone_numbers() {
        assert_eq!(hardness_for_meta(0), 1.0);
        assert_eq!(hardness_for_meta(2), 1.0);
        assert_eq!(hardness_for_meta(6), 2.5);
        assert_eq!(hardness_for_meta(9), 3.0);
        assert_eq!(blast_resistance_for_meta(1), 12.0);
        assert_eq!(blast_resistance_for_meta(2), 6.0);
        assert_eq!(StoneKind::Basalt.drop_kind(), StoneKind::BasaltCobblestone);
        assert_eq!(
            StoneKind::BasaltPaver.drop_kind(),
            StoneKind::BasaltCobblestone
        );
        assert_eq!(StoneKind::BasaltBrick.drop_kind(), StoneKind::BasaltBrick);
    }

    #[test]
    fn marble_fills_stone_above_air() {
        let f = |_x: i32, y: i32, _z: i32| {
            if y < 10 {
                BlockClass::Air
            } else {
                BlockClass::Stone
            }
        };
        let r = marble_fill((0, 5, 0), 20, f).unwrap();
        assert_eq!(r.len(), 20);
        assert_eq!(r[0], (0, 10, 0));
        assert!(marble_fill((0, 12, 0), 5, f).is_none());
        assert!(marble_fill((0, 0, 0), 5, |_, _, _| BlockClass::Air).is_none());
    }

    #[test]
    fn volcano_spread_keeps_best_power() {
        let mut v = VolcanoSpread::default();
        v.add(1, 5, 1, 2);
        v.add(1, 5, 1, 2);
        v.add(1, 6, 1, 3);
        v.add(2, 5, 2, 0);
        assert_eq!(v.queue.len(), 2);
        let mut rng = |_b: u32| 15;
        let mut s = VolcanoSpread::default();
        s.search(0, 0, 0, 3, &mut rng);
        assert_eq!(s.power[&(-1, 0)], 2);
    }
}
