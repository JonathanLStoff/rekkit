//! Flora: flax crop, indigo flower, rubber sapling, rubber leaves and the rubber tree.
//!
//! Specification reference: `planning/mods/redpower_2.md` §5 Plants & Trees.
//! Java reference: `eloraam/world/{BlockCustomCrops,BlockCustomFlower,BlockCustomLeaves,
//! WorldGenRubberTree,ItemCustomSeeds}.java` and `RedPowerWorld#GenerateSurface`.
//!
//! All logic is pure: randomness is passed in as already-rolled values or a closure
//! `rng(bound) -> 0..bound`, so the engine layer owns world access and the RNG.

pub const FLAX_CROP_ID: &str = "redpower_2:flax_crop";
pub const INDIGO_FLOWER_ID: &str = "redpower_2:indigo_flower";
pub const RUBBER_SAPLING_ID: &str = "redpower_2:rubber_sapling";
pub const RUBBER_LEAVES_ID: &str = "redpower_2:rubber_leaves";
pub const RUBBER_LOG_ID: &str = "redpower_2:rubber_log";
pub const FLAX_SEEDS_ID: &str = "redpower_2:flax_seeds";

/// Flax stage at which the lower half is ripe (`md == 4`); `5` is the upper half.
pub const FLAX_RIPE_LOWER: u8 = 4;
pub const FLAX_UPPER: u8 = 5;
/// Minimum light at the block above for flax to grow or be fertilized (9).
pub const FLAX_MIN_LIGHT: u8 = 9;
/// Growth chance per random tick is 1 in this many (30).
pub const FLAX_GROWTH_ODDS: u32 = 30;
/// Sapling light and odds (`BlockCustomFlower#a`): light >= 9, 1 in 300 per random tick.
pub const SAPLING_MIN_LIGHT: u8 = 9;
pub const SAPLING_GROWTH_ODDS: u32 = 300;
/// Leaves drop a sapling 1 time in 20.
pub const LEAF_SAPLING_ODDS: u32 = 20;
/// Leaf decay: a flagged leaf looks for a log within this many leaf steps (4).
pub const LEAF_LOG_RANGE: u8 = 4;
/// Rubber trees generated per chunk in jungle biomes.
pub const RUBBER_TREES_PER_CHUNK: u32 = 6;

/// Block metadata of the flower block (`BlockCustomFlower`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowerKind {
    Indigo,
    Sapling,
    /// Sapling in its second growth stage (meta 2, drops/pick as a sapling).
    SaplingGrown,
}

impl FlowerKind {
    pub fn from_meta(meta: u8) -> Self {
        match meta {
            1 => Self::Sapling,
            2 => Self::SaplingGrown,
            _ => Self::Indigo,
        }
    }
    pub fn meta(self) -> u8 {
        match self {
            Self::Indigo => 0,
            Self::Sapling => 1,
            Self::SaplingGrown => 2,
        }
    }
    /// Dropped damage value: meta 2 drops as 1 (`BlockCustomFlower#c`).
    pub fn drop_meta(self) -> u8 {
        match self {
            Self::SaplingGrown => 1,
            other => other.meta(),
        }
    }
}

/// Outcome of a flower-block random tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaplingTick {
    None,
    /// meta 1 -> meta 2
    Advance,
    /// meta 2 -> replace by a tree (restored to meta 1 sapling if generation fails)
    GrowTree,
}

/// `BlockCustomFlower#a` random tick. `roll_300` is `random.nextInt(300)`.
pub fn sapling_tick(meta: u8, light_above: u8, roll_300: u32) -> SaplingTick {
    if (meta != 1 && meta != 2) || light_above < SAPLING_MIN_LIGHT || roll_300 != 0 {
        return SaplingTick::None;
    }
    if meta == 1 {
        SaplingTick::Advance
    } else {
        SaplingTick::GrowTree
    }
}

/// Flax can stay planted (`BlockCustomCrops#g`). `below_is_tilled_soil` is
/// "farmland with nonzero moisture/meta"-agnostic; `below_flax_meta` is the flax meta
/// of the block underneath when it is flax.
pub fn flax_can_stay(
    meta: u8,
    below_is_farmland: bool,
    below_flax_meta: Option<u8>,
    above_air: bool,
) -> bool {
    if meta == FLAX_UPPER {
        return below_flax_meta == Some(FLAX_RIPE_LOWER);
    }
    if !below_is_farmland {
        return false;
    }
    meta == FLAX_RIPE_LOWER || above_air
}

/// Hitbox height of a flax stage (`BlockCustomCrops#a`): `min(1, 0.1 + 0.25 * meta)`.
pub fn flax_height(meta: u8) -> f32 {
    (0.1 + 0.25 * meta as f32).min(1.0)
}

/// Result of a flax random tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlaxGrowth {
    pub new_meta: u8,
    /// Place the upper half (meta 5) above when the lower half reaches 4.
    pub place_upper: bool,
}

/// `BlockCustomCrops#a` random tick. `roll_30` is `random.nextInt(30)`.
pub fn flax_tick(
    meta: u8,
    light_above: u8,
    on_tilled_farmland: bool,
    above_air: bool,
    roll_30: u32,
) -> Option<FlaxGrowth> {
    if light_above < FLAX_MIN_LIGHT || meta == FLAX_RIPE_LOWER || meta == FLAX_UPPER {
        return None;
    }
    if !on_tilled_farmland || !above_air || roll_30 != 0 {
        return None;
    }
    let new_meta = meta + 1;
    Some(FlaxGrowth {
        new_meta,
        place_upper: meta == 3,
    })
}

/// Bonemeal on flax (`fertilize`): jumps straight to the ripe pair.
pub fn flax_fertilize(
    meta: u8,
    light_above: u8,
    on_tilled_farmland: bool,
    above_air: bool,
) -> bool {
    light_above >= FLAX_MIN_LIGHT
        && meta != FLAX_RIPE_LOWER
        && meta != FLAX_UPPER
        && on_tilled_farmland
        && above_air
}

/// Drops of a broken flax block (`getBlockDropped`). `rng(n)` returns `0..n`.
/// Returns `(string_count, seed_count)`.
pub fn flax_drops(meta: u8, fortune: u32, rng: &mut dyn FnMut(u32) -> u32) -> (u32, u32) {
    let mut strings = 0;
    if meta == FLAX_RIPE_LOWER || meta == FLAX_UPPER {
        strings = 1 + rng(3) + rng(1 + fortune);
    }
    let mut md = meta;
    let mut seeds = 0;
    for _ in 0..(3 + fortune) {
        if md == FLAX_UPPER {
            md = FLAX_RIPE_LOWER;
        }
        if rng(8) <= md as u32 {
            seeds += 1;
        }
    }
    (strings, seeds)
}

/// Flax seeds are used on top of tilled farmland (face 1) with two air blocks above.
pub fn seeds_can_plant(
    face: u8,
    target_is_farmland: bool,
    farmland_meta: u8,
    air1: bool,
    air2: bool,
) -> bool {
    face == 1 && target_is_farmland && air1 && air2 && farmland_meta != 0
}

/// Leaf drop (`quantityDropped`): a sapling with `roll_20 == 0`.
pub fn leaf_sapling_drop(roll_20: u32) -> u32 {
    u32::from(roll_20 == 0)
}

/// Leaf meta bit that flags a leaf for decay checking.
pub const LEAF_DECAY_FLAG: u8 = 8;
pub const LEAF_PERSIST_MASK: u8 = 4;

/// Whether a leaf block should run the decay search on a random tick.
pub fn leaf_needs_decay_check(meta: u8) -> bool {
    meta & LEAF_DECAY_FLAG != 0 && meta & LEAF_PERSIST_MASK == 0
}

/// Breadth-first log search for leaf decay. `is_log(dx,dy,dz)` and `is_leaf(dx,dy,dz)` take
/// offsets from the checked leaf. Returns true if a log is reachable (leaf survives).
pub fn leaf_connected_to_log(
    is_log: &dyn Fn(i32, i32, i32) -> bool,
    is_leaf: &dyn Fn(i32, i32, i32) -> bool,
) -> bool {
    use std::collections::{HashMap, VecDeque};
    const DIRS: [(i32, i32, i32); 6] = [
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
        (-1, 0, 0),
        (1, 0, 0),
    ];
    let mut seen: HashMap<(i32, i32, i32), u8> = HashMap::new();
    let mut queue = VecDeque::new();
    seen.insert((0, 0, 0), LEAF_LOG_RANGE);
    queue.push_back((0, 0, 0));
    while let Some(c) = queue.pop_front() {
        let steps = seen[&c];
        for d in DIRS {
            let p = (c.0 + d.0, c.1 + d.1, c.2 + d.2);
            if seen.contains_key(&p) {
                continue;
            }
            if is_log(p.0, p.1, p.2) {
                return true;
            }
            if steps != 0 && is_leaf(p.0, p.1, p.2) {
                seen.insert(p, steps - 1);
                queue.push_back(p);
            }
        }
    }
    false
}

/// Tree height: `random.nextInt(6) + 25`.
pub fn tree_height(roll_6: u32) -> i32 {
    roll_6 as i32 + 25
}

/// Whether a rubber tree fits: the 3x3 footprint must sit on grass/dirt and the
/// column is clear (air, leaves, snow layer, vanilla leaves/log). `max_y` is the world height.
pub fn tree_can_grow(y: i32, height: i32, max_y: i32) -> bool {
    y >= 1 && y + height + 2 <= max_y
}

/// Radius of the clear check at height `dy` above the base: 1 for the first four layers, then 5.
pub fn tree_clear_radius(dy: i32) -> i32 {
    if dy > 3 {
        5
    } else {
        1
    }
}

/// Trunk is a 3x3 of rubber logs (meta 1) for the bottom 7 layers (`y = 0..=6`).
pub const TRUNK_LAYERS: i32 = 7;

/// Jungle surface count for the biome: rubber trees only in Jungle / JungleHills.
pub fn rubber_trees_for_biome(biome: &str) -> u32 {
    match biome {
        "jungle" | "jungle_hills" | "minecraft:jungle" | "minecraft:jungle_hills" => {
            RUBBER_TREES_PER_CHUNK
        }
        _ => 0,
    }
}

/// Indigo flower patch attempts per chunk by biome (`GenerateSurface`):
/// plains / forest / forest hills 1, swamp 4.
pub fn indigo_patches_for_biome(biome: &str) -> u32 {
    match biome.trim_start_matches("minecraft:") {
        "plains" | "forest" | "forest_hills" | "wooded_hills" => 1,
        "swamp" | "swampland" => 4,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flax_growth_and_height() {
        assert_eq!(
            flax_tick(3, 15, true, true, 0),
            Some(FlaxGrowth {
                new_meta: 4,
                place_upper: true
            })
        );
        assert_eq!(flax_tick(1, 15, true, true, 0).unwrap().new_meta, 2);
        assert!(flax_tick(1, 8, true, true, 0).is_none());
        assert!(flax_tick(1, 15, true, true, 5).is_none());
        assert!(flax_tick(4, 15, true, true, 0).is_none());
        assert_eq!(flax_height(0), 0.1);
        assert_eq!(flax_height(5), 1.0);
    }

    #[test]
    fn flax_support() {
        assert!(flax_can_stay(5, false, Some(4), false));
        assert!(!flax_can_stay(5, true, Some(3), false));
        assert!(flax_can_stay(4, true, None, false));
        assert!(!flax_can_stay(2, true, None, false));
        assert!(flax_can_stay(2, true, None, true));
    }

    #[test]
    fn flax_drop_counts() {
        let mut max = |n: u32| n - 1;
        assert_eq!(flax_drops(4, 0, &mut max), (3, 0));
        let mut zero = |_n: u32| 0;
        assert_eq!(flax_drops(0, 0, &mut zero), (0, 3));
    }

    #[test]
    fn sapling_cycle() {
        assert_eq!(sapling_tick(1, 9, 0), SaplingTick::Advance);
        assert_eq!(sapling_tick(2, 9, 0), SaplingTick::GrowTree);
        assert_eq!(sapling_tick(2, 8, 0), SaplingTick::None);
        assert_eq!(sapling_tick(0, 15, 0), SaplingTick::None);
        assert_eq!(FlowerKind::SaplingGrown.drop_meta(), 1);
    }

    #[test]
    fn leaves() {
        assert_eq!(leaf_sapling_drop(0), 1);
        assert_eq!(leaf_sapling_drop(7), 0);
        assert!(leaf_needs_decay_check(8));
        assert!(!leaf_needs_decay_check(12));
        // a log 3 leaf-steps away is within range; 6 is not
        let log_at = |n: i32| move |x: i32, y: i32, z: i32| y == 0 && z == 0 && x == n;
        let leaf = |x: i32, y: i32, z: i32| y == 0 && z == 0 && (0..8).contains(&x);
        assert!(leaf_connected_to_log(&log_at(4), &leaf));
        assert!(!leaf_connected_to_log(&log_at(7), &leaf));
    }

    #[test]
    fn tree_rules() {
        assert_eq!(tree_height(5), 30);
        assert!(tree_can_grow(64, 30, 256));
        assert!(!tree_can_grow(0, 30, 256));
        assert_eq!(tree_clear_radius(3), 1);
        assert_eq!(tree_clear_radius(4), 5);
        assert_eq!(rubber_trees_for_biome("jungle"), 6);
        assert_eq!(rubber_trees_for_biome("plains"), 0);
        assert_eq!(indigo_patches_for_biome("swamp"), 4);
        assert!(seeds_can_plant(1, true, 7, true, true));
        assert!(!seeds_can_plant(0, true, 7, true, true));
    }
}
