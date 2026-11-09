//! World and player interaction event handlers (`WorldEvents.java`).
//!
//! Handlers:
//! - Bonemeal fertilization on Flax crops and Rubber saplings.
//! - Athame bonus drops against Endermen and Ender Dragon.
//! - Seed bag automated seed collection on entity item pickup.

use crate::crops::{flax_fertilize, FLAX_CROP_ID};
use crate::trees::RUBBER_SAPLING_ID;
use crate::ItemStack;

/// Result of bonemeal fertilization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonemealResult {
    /// Flax crop accelerated straight to ripe pair (meta 4 + upper half 5).
    FlaxRipe,
    /// Rubber sapling triggered to grow into rubber tree.
    RubberTree,
}

/// Handles bonemeal right-click event (`WorldEvents.onBonemeal`).
pub fn handle_bonemeal(
    block_id: &str,
    meta: u8,
    light_above: u8,
    above_is_air: bool,
) -> Option<BonemealResult> {
    match block_id {
        FLAX_CROP_ID => {
            if flax_fertilize(meta, light_above, true, above_is_air) {
                Some(BonemealResult::FlaxRipe)
            } else {
                None
            }
        }
        RUBBER_SAPLING_ID => {
            if light_above >= 9 {
                Some(BonemealResult::RubberTree)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extra drops rolled when killing an entity with Athame (`WorldEvents.onDeath`).
pub fn handle_athame_kill(weapon_id: &str, victim_id: &str) -> Vec<ItemStack> {
    if weapon_id != "redpower_2:athame" {
        return Vec::new();
    }

    match victim_id.strip_prefix("minecraft:").unwrap_or(victim_id) {
        "enderman" => vec![ItemStack::new("minecraft:ender_pearl", 1)],
        "ender_dragon" => vec![ItemStack::new("minecraft:dragon_egg", 1)],
        _ => Vec::new(),
    }
}

/// Automatic item insertion into held Seed Bag upon pickup (`WorldEvents.onPickupItem`).
pub fn handle_seed_bag_pickup(held_item_id: &str, pickup_item: &ItemStack) -> bool {
    if held_item_id != "redpower_2:seed_bag" {
        return false;
    }

    let id = pickup_item.item_id.as_str();
    id == "redpower_2:flax_seeds"
        || id == "minecraft:wheat_seeds"
        || id == "minecraft:pumpkin_seeds"
        || id == "minecraft:melon_seeds"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonemeal_on_flax_and_sapling() {
        assert_eq!(
            handle_bonemeal("redpower_2:flax_crop", 0, 15, true),
            Some(BonemealResult::FlaxRipe)
        );
        // Already ripe
        assert_eq!(handle_bonemeal("redpower_2:flax_crop", 4, 15, true), None);
        // Darkness
        assert_eq!(handle_bonemeal("redpower_2:flax_crop", 0, 8, true), None);

        // Rubber sapling
        assert_eq!(
            handle_bonemeal("redpower_2:rubber_sapling", 1, 15, true),
            Some(BonemealResult::RubberTree)
        );
    }

    #[test]
    fn test_athame_bonus_drops() {
        let drops = handle_athame_kill("redpower_2:athame", "minecraft:enderman");
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item_id, "minecraft:ender_pearl");

        let normal_drops = handle_athame_kill("minecraft:diamond_sword", "minecraft:enderman");
        assert!(normal_drops.is_empty());
    }

    #[test]
    fn test_seed_bag_pickup() {
        let seeds = ItemStack::new("redpower_2:flax_seeds", 1);
        assert!(handle_seed_bag_pickup("redpower_2:seed_bag", &seeds));

        let dirt = ItemStack::new("minecraft:dirt", 1);
        assert!(!handle_seed_bag_pickup("redpower_2:seed_bag", &dirt));
    }
}
