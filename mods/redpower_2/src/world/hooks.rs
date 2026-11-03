//! Engine-facing decisions for the world hooks (random tick, chunk generation, melee hit,
//! ribbon placement). Pure: a native hook feeds in what it read and applies what comes back.

use super::{athame, flora, ores};
use crate::wiring::ribbon::{RibbonCable, RIBBON_ITEM_ID};
use rekkit_core::side::Direction;

/// What a random tick on a flora block should do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloraAction {
    /// Not ours / nothing happens this tick.
    None,
    /// Set the block's meta to `new_meta`; place the flax upper half above when `place_upper`.
    SetMeta { new_meta: u8, place_upper: bool },
    /// Replace the sapling by a rubber tree (restore meta 1 if it does not fit).
    GrowTree,
}

/// Random tick of `block` at meta `meta`. `roll(n)` is `random.nextInt(n)`.
pub fn flora_random_tick(
    block: &str,
    meta: u8,
    light_above: u8,
    on_tilled_farmland: bool,
    above_air: bool,
    roll: &mut dyn FnMut(u32) -> u32,
) -> FloraAction {
    match block {
        flora::FLAX_CROP_ID => {
            let r = roll(flora::FLAX_GROWTH_ODDS);
            match flora::flax_tick(meta, light_above, on_tilled_farmland, above_air, r) {
                Some(g) => FloraAction::SetMeta {
                    new_meta: g.new_meta,
                    place_upper: g.place_upper,
                },
                None => FloraAction::None,
            }
        }
        flora::RUBBER_SAPLING_ID => {
            match flora::sapling_tick(meta, light_above, roll(flora::SAPLING_GROWTH_ODDS)) {
                flora::SaplingTick::None => FloraAction::None,
                flora::SaplingTick::Advance => FloraAction::SetMeta {
                    new_meta: 2,
                    place_upper: false,
                },
                flora::SaplingTick::GrowTree => FloraAction::GrowTree,
            }
        }
        _ => FloraAction::None,
    }
}

/// One block a generated chunk should place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Feature {
    /// Ore block `block` (meta `meta`) replacing stone at the position.
    Ore { block: &'static str, meta: u8, pos: (i32, i32, i32) },
    /// A rubber tree attempt at column `(x, z)`.
    RubberTree { x: i32, z: i32 },
    /// An indigo flower patch attempt at column `(x, z)`.
    IndigoPatch { x: i32, z: i32 },
}

/// Everything RedPower generates in the chunk whose block origin is `(chunk_x, chunk_z)`.
/// `rng_f` is a float in `[0,1)`, `rng_n(n)` an int in `0..n`. Ore positions are returned for
/// the caller to keep only where the world has stone.
pub fn chunk_features(
    chunk_x: i32,
    chunk_z: i32,
    biome: &str,
    rng_f: &mut dyn FnMut() -> f64,
    rng_n: &mut dyn FnMut(u32) -> u32,
) -> Vec<Feature> {
    let mut out = Vec::new();
    for ore in ores::ALL_ORES {
        let g = ore.gen();
        for (x, y, z) in ores::vein_origins(ore, chunk_x, chunk_z, rng_n) {
            for pos in ores::vein_positions(g.vein_size, x, y, z, rng_f, rng_n) {
                out.push(Feature::Ore {
                    block: ore.block_id(),
                    meta: ore.meta(),
                    pos,
                });
            }
        }
    }
    for _ in 0..flora::rubber_trees_for_biome(biome) {
        let (x, z) = (chunk_x + rng_n(16) as i32, chunk_z + rng_n(16) as i32);
        out.push(Feature::RubberTree { x, z });
    }
    for _ in 0..flora::indigo_patches_for_biome(biome) {
        let (x, z) = (chunk_x + rng_n(16) as i32, chunk_z + rng_n(16) as i32);
        out.push(Feature::IndigoPatch { x, z });
    }
    out
}

/// Damage the held item deals to `victim_kind`, or `None` when the item is not ours.
pub fn melee_damage(held_item: &str, victim_kind: &str, disjunction_level: u32) -> Option<i32> {
    (held_item == athame::ATHAME_ITEM_ID).then(|| {
        athame::athame_attack_damage(athame::is_ender_mob(victim_kind), disjunction_level)
    })
}

/// Using `item` on the `clicked_face` of a block places a ribbon cable on that face.
pub fn ribbon_placement(item: &str, clicked_face: Direction) -> Option<RibbonCable> {
    (item == RIBBON_ITEM_ID).then(|| RibbonCable::place_on_clicked_face(clicked_face))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flax_grows_on_zero_roll() {
        let mut z = |_n: u32| 0;
        assert_eq!(
            flora_random_tick(flora::FLAX_CROP_ID, 3, 12, true, true, &mut z),
            FloraAction::SetMeta { new_meta: 4, place_upper: true }
        );
        assert_eq!(
            flora_random_tick("redpower_2:other", 3, 12, true, true, &mut z),
            FloraAction::None
        );
        assert_eq!(
            flora_random_tick(flora::RUBBER_SAPLING_ID, 2, 12, false, false, &mut z),
            FloraAction::GrowTree
        );
    }

    #[test]
    fn chunk_has_ores_and_jungle_trees() {
        let mut s = 7u64;
        let mut t = 11u64;
        let mut rf = move || {
            t ^= t << 13;
            t ^= t >> 7;
            t ^= t << 17;
            (t >> 11) as f64 / (1u64 << 53) as f64
        };
        let mut rn = move |n: u32| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s % n as u64) as u32
        };
        let f = chunk_features(0, 0, "jungle", &mut rf, &mut rn);
        assert!(f.iter().any(|x| matches!(x, Feature::Ore { .. })));
        assert_eq!(
            f.iter().filter(|x| matches!(x, Feature::RubberTree { .. })).count(),
            6
        );
    }

    #[test]
    fn athame_and_ribbon() {
        assert_eq!(melee_damage(athame::ATHAME_ITEM_ID, "minecraft:enderman", 2), Some(37));
        assert_eq!(melee_damage(athame::ATHAME_ITEM_ID, "minecraft:zombie", 2), Some(1));
        assert_eq!(melee_damage("minecraft:stick", "x", 0), None);
        let r = ribbon_placement(RIBBON_ITEM_ID, Direction::Up).unwrap();
        assert_eq!(r.face, Direction::Down);
    }
}
