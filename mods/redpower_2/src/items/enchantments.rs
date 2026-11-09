//! RedPower 2 combat enchantments: Disjunction and Vorpal.
//!
//! Java reference: `eloraam/world/{EnchantmentDisjunction,EnchantmentVorpal,WorldEvents}.java`.

use crate::athame::is_ender_mob;
use crate::ItemStack;

/// Canonical enchantment ID for Disjunction.
pub const DISJUNCTION_ID: &str = "redpower_2:disjunction";
/// Canonical enchantment ID for Vorpal.
pub const VORPAL_ID: &str = "redpower_2:vorpal";

/// Bonus damage per level for Disjunction against Ender mobs (+6 per level).
pub const DISJUNCTION_BONUS_PER_LEVEL: i32 = 6;
/// Vorpal critical bonus damage (+100).
pub const VORPAL_CRITICAL_BONUS: i32 = 100;
/// Health threshold for Vorpal decapitation (overkill <= -20 HP).
pub const VORPAL_DECAPITATION_THRESHOLD: i32 = -20;

/// Calculates Disjunction damage bonus against an entity.
pub fn disjunction_bonus_damage(level: u32, victim_id: &str) -> i32 {
    if is_ender_mob(victim_id) {
        (level as i32) * DISJUNCTION_BONUS_PER_LEVEL
    } else {
        0
    }
}

/// Evaluates Vorpal critical strike bonus given level and a random roll 0..100.
///
/// Java: `ent.p.t.nextInt(100) < 2 * i * i ? 100 : 0`
pub fn vorpal_critical_damage(level: u32, roll_100: u32) -> i32 {
    if level == 0 {
        return 0;
    }
    let threshold = 2 * level * level;
    if roll_100 < threshold {
        VORPAL_CRITICAL_BONUS
    } else {
        0
    }
}

/// Evaluates whether a Vorpal strike decapitates the victim based on fatal overkill damage.
///
/// Java: `ev.entityLiving.getHealth() <= -20`
pub fn vorpal_decapitates(level: u32, final_health: i32) -> bool {
    level > 0 && final_health <= VORPAL_DECAPITATION_THRESHOLD
}

/// Determines the dropped skull item upon successful Vorpal decapitation.
///
/// Java:
/// - Skeleton (non-wither): skull 0 (`minecraft:skeleton_skull`)
/// - Zombie: head 2 (`minecraft:zombie_head`)
/// - Player: head 3 (`minecraft:player_head`)
/// - Creeper: head 4 (`minecraft:creeper_head`)
pub fn vorpal_head_drop(victim_type: &str, is_wither_skeleton: bool) -> Option<ItemStack> {
    let lower = victim_type
        .strip_prefix("minecraft:")
        .unwrap_or(victim_type)
        .to_ascii_lowercase();

    match lower.as_str() {
        "skeleton" => {
            if is_wither_skeleton {
                None // Wither skeletons do not drop normal skeleton skulls from vorpal
            } else {
                Some(ItemStack::new("minecraft:skeleton_skull", 1))
            }
        }
        "zombie" => Some(ItemStack::new("minecraft:zombie_head", 1)),
        "player" => Some(ItemStack::new("minecraft:player_head", 1)),
        "creeper" => Some(ItemStack::new("minecraft:creeper_head", 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disjunction_bonus() {
        assert_eq!(disjunction_bonus_damage(1, "minecraft:enderman"), 6);
        assert_eq!(disjunction_bonus_damage(5, "minecraft:enderman"), 30);
        assert_eq!(disjunction_bonus_damage(3, "minecraft:ender_dragon"), 18);
        assert_eq!(disjunction_bonus_damage(5, "minecraft:zombie"), 0);
    }

    #[test]
    fn test_vorpal_critical_chances() {
        // Level 1: 2 * 1^2 = 2% chance
        assert_eq!(vorpal_critical_damage(1, 0), 100);
        assert_eq!(vorpal_critical_damage(1, 1), 100);
        assert_eq!(vorpal_critical_damage(1, 2), 0);

        // Level 2: 2 * 2^2 = 8% chance
        assert_eq!(vorpal_critical_damage(2, 7), 100);
        assert_eq!(vorpal_critical_damage(2, 8), 0);

        // Level 3: 2 * 3^2 = 18% chance
        assert_eq!(vorpal_critical_damage(3, 17), 100);
        assert_eq!(vorpal_critical_damage(3, 18), 0);

        // Level 0: 0% chance
        assert_eq!(vorpal_critical_damage(0, 0), 0);
    }

    #[test]
    fn test_vorpal_decapitation_thresholds() {
        // Must be <= -20 HP
        assert!(vorpal_decapitates(1, -20));
        assert!(vorpal_decapitates(1, -25));
        assert!(!vorpal_decapitates(1, -19));
        assert!(!vorpal_decapitates(1, 0));
        assert!(!vorpal_decapitates(0, -50)); // No vorpal
    }

    #[test]
    fn test_vorpal_head_drops() {
        let skel = vorpal_head_drop("minecraft:skeleton", false).expect("skeleton drop");
        assert_eq!(skel.item_id, "minecraft:skeleton_skull");

        // Wither skeleton does not drop normal skull
        assert!(vorpal_head_drop("minecraft:skeleton", true).is_none());

        let zombie = vorpal_head_drop("minecraft:zombie", false).expect("zombie drop");
        assert_eq!(zombie.item_id, "minecraft:zombie_head");

        let creeper = vorpal_head_drop("minecraft:creeper", false).expect("creeper drop");
        assert_eq!(creeper.item_id, "minecraft:creeper_head");

        let player = vorpal_head_drop("minecraft:player", false).expect("player drop");
        assert_eq!(player.item_id, "minecraft:player_head");

        // Cows do not drop heads
        assert!(vorpal_head_drop("minecraft:cow", false).is_none());
    }
}
