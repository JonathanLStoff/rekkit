//! Athame ceremonial silver dagger (`redpower_2:athame`).
//!
//! Java reference: `eloraam/world/ItemAthame.java`.
//!
//! Properties:
//! - Base damage: 1 to ordinary mobs.
//! - Massive damage: 25 to Endermen and Ender Dragon.
//! - Durability: 100 uses.
//! - High enchantability (30).
//! - Repair material: Silver Ingot (`redpower_2:silver_ingot`).

pub use crate::athame::{
    athame_attack_damage, can_enchant_disjunction, is_ender_mob, ATHAME_BASE_DAMAGE,
    ATHAME_DURABILITY, ATHAME_EFFICIENCY, ATHAME_ENCHANTABILITY, ATHAME_ENDER_DAMAGE,
    ATHAME_HARVEST_LEVEL, ATHAME_ITEM_ID, ATHAME_REPAIR_MATERIAL, ATHAME_TOOL_CLASS,
};

/// Calculates total damage dealt by an Athame against a target entity name.
pub fn calculate_athame_damage(victim_id: &str, disjunction_level: u32) -> i32 {
    let is_ender = is_ender_mob(victim_id);
    athame_attack_damage(is_ender, disjunction_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_athame_damage_scaling() {
        // Normal mobs take 1 damage, unaffected by Disjunction
        assert_eq!(calculate_athame_damage("minecraft:zombie", 0), 1);
        assert_eq!(calculate_athame_damage("minecraft:skeleton", 5), 1);
        assert_eq!(calculate_athame_damage("minecraft:cow", 3), 1);

        // Ender mobs take 25 base damage + 6 per level of Disjunction
        assert_eq!(calculate_athame_damage("minecraft:enderman", 0), 25);
        assert_eq!(calculate_athame_damage("minecraft:enderman", 1), 31);
        assert_eq!(calculate_athame_damage("minecraft:enderman", 5), 55);

        assert_eq!(calculate_athame_damage("minecraft:ender_dragon", 0), 25);
        assert_eq!(calculate_athame_damage("minecraft:ender_dragon", 3), 43);
    }

    #[test]
    fn test_athame_properties() {
        assert_eq!(ATHAME_DURABILITY, 100);
        assert_eq!(ATHAME_ENCHANTABILITY, 30);
        assert_eq!(ATHAME_REPAIR_MATERIAL, "redpower_2:silver_ingot");
        assert!(can_enchant_disjunction("redpower_2:athame"));
        assert!(!can_enchant_disjunction("minecraft:diamond_sword"));
    }
}
