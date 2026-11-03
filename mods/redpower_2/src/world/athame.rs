//! Athame ceremonial silver dagger (`redpower_2:athame`) and Disjunction enchantment.
//!
//! Specification reference: `planning/mods/redpower_2.md` §4.2 Mining & Combat Equipment.
//! Java reference:
//! - `ItemAthame.java` (`eloraam/world/ItemAthame.java` & `com/eloraam/redpower/world/ItemAthame.java`)
//! - `EnchantmentDisjunction.java` (`eloraam/world/EnchantmentDisjunction.java` & `com/eloraam/redpower/world/EnchantmentDisjunction.java`)

use rekkit_core::inventory::ItemStack;
use serde::{Deserialize, Serialize};

/// Canonical item identifier for Athame.
pub const ATHAME_ITEM_ID: &str = "redpower_2:athame";

/// Durability of Athame (100 uses, `ItemAthame.java:8`).
pub const ATHAME_DURABILITY: u32 = 100;

/// Base damage against normal mobs (1 damage, `ItemAthame.java:18`).
pub const ATHAME_BASE_DAMAGE: i32 = 1;

/// Base damage against Endermen and Ender Dragon (25 damage, `ItemAthame.java:18`).
pub const ATHAME_ENDER_DAMAGE: i32 = 25;

/// Enchantability rating of Athame (30, `ItemAthame.java:30`).
pub const ATHAME_ENCHANTABILITY: u32 = 30;

/// Mining efficiency against blocks (1.0F, `ItemAthame.java:14`).
pub const ATHAME_EFFICIENCY: f32 = 1.0;

/// Harvest level of Athame as a sword (0, `MinecraftForge.setToolClass(itemAthame, "sword", 0)`).
pub const ATHAME_HARVEST_LEVEL: u32 = 0;

/// Tool class string for Athame.
pub const ATHAME_TOOL_CLASS: &str = "sword";

/// Repair item ID for Athame (`redpower_2:silver_ingot`, `ItemAthame.java:26`).
pub const ATHAME_REPAIR_MATERIAL: &str = "redpower_2:silver_ingot";

/// Canonical identifier for Disjunction enchantment.
pub const DISJUNCTION_ENCHANTMENT_ID: &str = "redpower_2:disjunction";

/// Localization key for Disjunction enchantment.
pub const DISJUNCTION_NAME: &str = "enchantment.damage.disjunction";

/// Rarity weight of Disjunction enchantment (10, `RedPowerWorld.java:395`).
pub const DISJUNCTION_WEIGHT: u32 = 10;

/// Minimum enchantment level for Disjunction (1).
pub const DISJUNCTION_MIN_LEVEL: u32 = 1;

/// Maximum enchantment level for Disjunction (5, `EnchantmentDisjunction.java:19`).
pub const DISJUNCTION_MAX_LEVEL: u32 = 5;

/// Additional damage dealt per Disjunction level to Ender mobs (+6, `EnchantmentDisjunction.java:23`).
pub const DISJUNCTION_PER_LEVEL_DAMAGE: i32 = 6;

/// Minimum enchantability base constant (5, `EnchantmentDisjunction.java:11`).
pub const DISJUNCTION_ENCHANTABILITY_BASE: u32 = 5;

/// Minimum enchantability slope per level (8, `EnchantmentDisjunction.java:11`).
pub const DISJUNCTION_ENCHANTABILITY_PER_LEVEL: u32 = 8;

/// Enchantability span window above minimum (20, `EnchantmentDisjunction.java:15`).
pub const DISJUNCTION_ENCHANTABILITY_SPAN: u32 = 20;

/// Check if a victim entity kind represents an Enderman or the Ender Dragon (§4.2).
///
/// Matches against standard Minecraft names (`"minecraft:enderman"`, `"minecraft:ender_dragon"`),
/// internal aliases (`"enderman"`, `"ender_dragon"`, `"enderdragon"`), and case variations.
pub fn is_ender_mob(entity_kind: &str) -> bool {
    let lower = entity_kind.to_ascii_lowercase();
    lower.contains("enderman") || lower.contains("ender_dragon") || lower.contains("enderdragon")
}

/// Calculate attack damage for Athame against a target entity, accounting for Disjunction enchantment (§4.2).
///
/// Formulas:
/// - Normal mob: 1 base damage. Disjunction does NOT apply (+0).
/// - Ender mob: 25 base damage + (6 * disjunction_level).
pub fn athame_attack_damage(is_ender: bool, disjunction_level: u32) -> i32 {
    let base = if is_ender {
        ATHAME_ENDER_DAMAGE
    } else {
        ATHAME_BASE_DAMAGE
    };
    if is_ender {
        base + (disjunction_level as i32) * DISJUNCTION_PER_LEVEL_DAMAGE
    } else {
        base
    }
}

/// Calculate minimum enchantability required for Disjunction at a given level: `5 + 8 * level` (`EnchantmentDisjunction.java:11`).
pub const fn disjunction_min_enchantability(level: u32) -> u32 {
    DISJUNCTION_ENCHANTABILITY_BASE + DISJUNCTION_ENCHANTABILITY_PER_LEVEL * level
}

/// Calculate maximum enchantability for Disjunction at a given level: `min_enchantability + 20` (`EnchantmentDisjunction.java:15`).
pub const fn disjunction_max_enchantability(level: u32) -> u32 {
    disjunction_min_enchantability(level) + DISJUNCTION_ENCHANTABILITY_SPAN
}

/// Check if Disjunction can be applied to an item (`EnchantmentDisjunction.java:31`).
///
/// Disjunction is exclusive to Athame. It cannot be applied to swords, sickles, or other tools.
pub fn can_enchant_disjunction(item_id: &str) -> bool {
    item_id == ATHAME_ITEM_ID
}

/// Check if Disjunction can be combined with another enchantment (`EnchantmentDisjunction.java:35`).
///
/// In the original Java, Disjunction is incompatible with itself and all damage enchantments
/// (Sharpness, Smite, Bane of Arthropods). It is compatible with non-damage enchantments
/// like Unbreaking, Looting, Fire Aspect, and Knockback.
pub fn disjunction_can_apply_together(other_enchantment: &str) -> bool {
    let lower = other_enchantment.to_ascii_lowercase();
    if lower == DISJUNCTION_ENCHANTMENT_ID
        || lower == "disjunction"
        || lower.contains("disjunction")
    {
        return false;
    }
    // Incompatible with damage enchantments (instances of EnchantmentDamage)
    if lower.contains("sharpness")
        || lower.contains("smite")
        || lower.contains("bane_of_arthropods")
        || lower.contains("damage")
    {
        return false;
    }
    true
}

/// Check if an item is a valid repair material for Athame (`ItemAthame.java:26`).
///
/// Athame is repaired on an anvil using Silver Ingots (`redpower_2:silver_ingot`).
pub fn athame_is_repairable_with(material_item_id: &str) -> bool {
    material_item_id == ATHAME_REPAIR_MATERIAL || material_item_id == "silver_ingot"
}

/// Represents the Athame silver ritual dagger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Athame {
    pub damage: u16,
    pub disjunction_level: u32,
}

impl Default for Athame {
    fn default() -> Self {
        Self::new()
    }
}

impl Athame {
    /// Create a pristine Athame dagger.
    pub const fn new() -> Self {
        Self {
            damage: 0,
            disjunction_level: 0,
        }
    }

    /// Create an Athame dagger enchanted with Disjunction.
    pub const fn with_disjunction(level: u32) -> Self {
        Self {
            damage: 0,
            disjunction_level: if level > DISJUNCTION_MAX_LEVEL {
                DISJUNCTION_MAX_LEVEL
            } else {
                level
            },
        }
    }

    /// Returns the maximum durability of the dagger.
    pub const fn max_durability() -> u32 {
        ATHAME_DURABILITY
    }

    /// Returns the enchantability rating of the dagger.
    pub const fn enchantability() -> u32 {
        ATHAME_ENCHANTABILITY
    }

    /// Check if the dagger is broken (exhausted durability).
    pub const fn is_broken(&self) -> bool {
        self.damage as u32 >= ATHAME_DURABILITY
    }

    /// Remaining durability uses before breaking.
    pub const fn remaining_uses(&self) -> u32 {
        if self.damage as u32 >= ATHAME_DURABILITY {
            0
        } else {
            ATHAME_DURABILITY - (self.damage as u32)
        }
    }

    /// Deal damage against an entity, calculating total damage dealt and incrementing tool wear.
    ///
    /// Returns `(damage_dealt, is_now_broken)`.
    pub fn attack(&mut self, entity_kind: &str) -> (i32, bool) {
        if self.is_broken() {
            return (0, true);
        }
        let is_ender = is_ender_mob(entity_kind);
        let dmg = athame_attack_damage(is_ender, self.disjunction_level);
        self.damage = self.damage.saturating_add(1);
        (dmg, self.is_broken())
    }

    /// Repair the dagger using a repair material, restoring durability.
    ///
    /// Each silver ingot repairs 25% of maximum durability (standard vanilla/Forge repair ratio).
    pub fn repair_with(&mut self, material_item_id: &str) -> bool {
        if !athame_is_repairable_with(material_item_id) {
            return false;
        }
        let repair_amount = (ATHAME_DURABILITY / 4) as u16;
        self.damage = self.damage.saturating_sub(repair_amount);
        true
    }

    /// Build an [`ItemStack`] representing this Athame.
    pub fn to_item_stack(&self) -> ItemStack {
        let mut stack = ItemStack::new(ATHAME_ITEM_ID, 1).with_max_stack(1);
        stack.damage = self.damage;
        stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_athame_constants_and_properties() {
        assert_eq!(ATHAME_ITEM_ID, "redpower_2:athame");
        assert_eq!(ATHAME_DURABILITY, 100);
        assert_eq!(ATHAME_BASE_DAMAGE, 1);
        assert_eq!(ATHAME_ENDER_DAMAGE, 25);
        assert_eq!(ATHAME_ENCHANTABILITY, 30);
        assert_eq!(ATHAME_EFFICIENCY, 1.0);
        assert_eq!(ATHAME_HARVEST_LEVEL, 0);
        assert_eq!(ATHAME_TOOL_CLASS, "sword");
        assert_eq!(ATHAME_REPAIR_MATERIAL, "redpower_2:silver_ingot");
    }

    #[test]
    fn test_is_ender_mob_detection() {
        assert!(is_ender_mob("minecraft:enderman"));
        assert!(is_ender_mob("minecraft:ender_dragon"));
        assert!(is_ender_mob("enderman"));
        assert!(is_ender_mob("enderdragon"));
        assert!(is_ender_mob("Ender_Dragon"));
        assert!(is_ender_mob("ENDERMAN"));

        assert!(!is_ender_mob("minecraft:zombie"));
        assert!(!is_ender_mob("minecraft:skeleton"));
        assert!(!is_ender_mob("minecraft:creeper"));
        assert!(!is_ender_mob("minecraft:spider"));
        assert!(!is_ender_mob("minecraft:player"));
        assert!(!is_ender_mob("cow"));
    }

    #[test]
    fn test_athame_attack_damage_formulas() {
        // Normal mobs: base 1 damage, unaffected by Disjunction
        assert_eq!(athame_attack_damage(false, 0), 1);
        assert_eq!(athame_attack_damage(false, 1), 1);
        assert_eq!(athame_attack_damage(false, 3), 1);
        assert_eq!(athame_attack_damage(false, 5), 1);

        // Ender mobs: 25 base + 6 * level
        assert_eq!(athame_attack_damage(true, 0), 25);
        assert_eq!(athame_attack_damage(true, 1), 31);
        assert_eq!(athame_attack_damage(true, 2), 37);
        assert_eq!(athame_attack_damage(true, 3), 43);
        assert_eq!(athame_attack_damage(true, 4), 49);
        assert_eq!(athame_attack_damage(true, 5), 55);
    }

    #[test]
    fn test_disjunction_enchantability_formula() {
        // min = 5 + 8 * level, max = min + 20
        assert_eq!(disjunction_min_enchantability(1), 13);
        assert_eq!(disjunction_max_enchantability(1), 33);

        assert_eq!(disjunction_min_enchantability(2), 21);
        assert_eq!(disjunction_max_enchantability(2), 41);

        assert_eq!(disjunction_min_enchantability(3), 29);
        assert_eq!(disjunction_max_enchantability(3), 49);

        assert_eq!(disjunction_min_enchantability(4), 37);
        assert_eq!(disjunction_max_enchantability(4), 57);

        assert_eq!(disjunction_min_enchantability(5), 45);
        assert_eq!(disjunction_max_enchantability(5), 65);
    }

    #[test]
    fn test_disjunction_item_exclusivity() {
        assert!(can_enchant_disjunction("redpower_2:athame"));
        assert!(!can_enchant_disjunction("minecraft:diamond_sword"));
        assert!(!can_enchant_disjunction("minecraft:iron_sword"));
        assert!(!can_enchant_disjunction("redpower_2:ruby_sword"));
        assert!(!can_enchant_disjunction("redpower_2:iron_sickle"));
    }

    #[test]
    fn test_disjunction_compatibility() {
        // Incompatible with self
        assert!(!disjunction_can_apply_together("redpower_2:disjunction"));
        assert!(!disjunction_can_apply_together("disjunction"));

        // Incompatible with vanilla damage enchantments
        assert!(!disjunction_can_apply_together("minecraft:sharpness"));
        assert!(!disjunction_can_apply_together("minecraft:smite"));
        assert!(!disjunction_can_apply_together(
            "minecraft:bane_of_arthropods"
        ));

        // Compatible with utility / non-damage enchantments
        assert!(disjunction_can_apply_together("minecraft:unbreaking"));
        assert!(disjunction_can_apply_together("minecraft:looting"));
        assert!(disjunction_can_apply_together("minecraft:fire_aspect"));
        assert!(disjunction_can_apply_together("minecraft:knockback"));
    }

    #[test]
    fn test_athame_struct_lifecycle_and_repair() {
        let mut athame = Athame::with_disjunction(3);
        assert_eq!(athame.remaining_uses(), 100);
        assert!(!athame.is_broken());

        // Attack a normal zombie
        let (dmg, broken) = athame.attack("minecraft:zombie");
        assert_eq!(dmg, 1);
        assert!(!broken);
        assert_eq!(athame.damage, 1);
        assert_eq!(athame.remaining_uses(), 99);

        // Attack an Enderman: 25 + 6 * 3 = 43 damage
        let (dmg, broken) = athame.attack("minecraft:enderman");
        assert_eq!(dmg, 43);
        assert!(!broken);
        assert_eq!(athame.damage, 2);

        // Wear it down
        athame.damage = 99;
        let (_, broken) = athame.attack("minecraft:ender_dragon");
        assert!(broken);
        assert!(athame.is_broken());
        assert_eq!(athame.remaining_uses(), 0);

        // Broken athame deals 0 damage
        let (dmg, broken) = athame.attack("minecraft:enderman");
        assert_eq!(dmg, 0);
        assert!(broken);

        // Repair with silver ingot
        assert!(!athame.repair_with("minecraft:iron_ingot"));
        assert!(athame.repair_with("redpower_2:silver_ingot"));
        assert_eq!(athame.damage, 75);
        assert!(!athame.is_broken());
        assert_eq!(athame.remaining_uses(), 25);
    }
}
