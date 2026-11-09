//! Flax crop and seeds mechanics (`redpower_2:flax_crop` and `redpower_2:flax_seeds`).
//!
//! Java reference: `eloraam/world/{BlockCustomCrops,ItemCustomSeeds}.java`.

pub use crate::flora::{
    flax_can_stay, flax_drops, flax_fertilize, flax_height, flax_tick, seeds_can_plant, FlaxGrowth,
    FLAX_CROP_ID, FLAX_GROWTH_ODDS, FLAX_MIN_LIGHT, FLAX_RIPE_LOWER, FLAX_SEEDS_ID, FLAX_UPPER,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flax_crop_lifecycle() {
        // Ticking through stages 0 to 4
        for stage in 0..3 {
            let growth = flax_tick(stage, 15, true, true, 0).expect("should advance");
            assert_eq!(growth.new_meta, stage + 1);
            assert!(!growth.place_upper);
        }

        // Stage 3 -> 4 places upper half (meta 5)
        let ripe_growth = flax_tick(3, 15, true, true, 0).expect("should produce upper half");
        assert_eq!(ripe_growth.new_meta, 4);
        assert!(ripe_growth.place_upper);

        // Stage 4 (ripe) does not advance further
        assert!(flax_tick(4, 15, true, true, 0).is_none());
    }

    #[test]
    fn test_flax_bonemeal_acceleration() {
        assert!(flax_fertilize(0, 15, true, true));
        assert!(flax_fertilize(3, 15, true, true));
        // Already ripe or dark
        assert!(!flax_fertilize(4, 15, true, true));
        assert!(!flax_fertilize(0, 8, true, true));
    }

    #[test]
    fn test_flax_harvest_drops() {
        let mut rng = |_bound| 0;
        let (strings, seeds) = flax_drops(4, 0, &mut rng);
        assert_eq!(strings, 1);
        assert!(seeds > 0);
    }
}
