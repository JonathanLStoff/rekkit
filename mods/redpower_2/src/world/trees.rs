//! Rubber trees, saplings, leaves, and wild flowers.
//!
//! Java reference: `eloraam/world/{BlockCustomFlower,BlockCustomLeaves,BlockCustomLog,WorldGenRubberTree}.java`.

pub use crate::flora::{
    indigo_patches_for_biome, leaf_connected_to_log, leaf_needs_decay_check, leaf_sapling_drop,
    rubber_trees_for_biome, sapling_tick, tree_can_grow, tree_clear_radius, tree_height,
    FlowerKind, SaplingTick, INDIGO_FLOWER_ID, LEAF_DECAY_FLAG, LEAF_LOG_RANGE, LEAF_PERSIST_MASK,
    LEAF_SAPLING_ODDS, RUBBER_LEAVES_ID, RUBBER_LOG_ID, RUBBER_SAPLING_ID, RUBBER_TREES_PER_CHUNK,
    SAPLING_GROWTH_ODDS, SAPLING_MIN_LIGHT, TRUNK_LAYERS,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rubber_sapling_growth() {
        // Stage 1 sapling advances to grown stage 2
        assert_eq!(sapling_tick(1, 15, 0), SaplingTick::Advance);
        // Stage 2 grown sapling triggers tree growth
        assert_eq!(sapling_tick(2, 15, 0), SaplingTick::GrowTree);
        // Low light fails
        assert_eq!(sapling_tick(1, 8, 0), SaplingTick::None);
    }

    #[test]
    fn test_rubber_leaves_decay_and_drops() {
        // Drop rate: 1 in 20
        assert_eq!(leaf_sapling_drop(0), 1);
        assert_eq!(leaf_sapling_drop(1), 0);

        // Connected to log check
        let is_log = |x: i32, y: i32, z: i32| (x, y, z) == (2, 0, 0);
        let is_leaf = |_x: i32, _y: i32, _z: i32| true;
        assert!(leaf_connected_to_log(&is_log, &is_leaf));

        let no_log = |_x: i32, _y: i32, _z: i32| false;
        assert!(!leaf_connected_to_log(&no_log, &is_leaf));
    }

    #[test]
    fn test_indigo_flower_and_rubber_biome_generation() {
        assert_eq!(rubber_trees_for_biome("jungle"), 6);
        assert_eq!(rubber_trees_for_biome("forest"), 0);

        assert_eq!(indigo_patches_for_biome("swamp"), 4);
        assert_eq!(indigo_patches_for_biome("plains"), 1);
        assert_eq!(indigo_patches_for_biome("desert"), 0);
    }
}
