//! Wool Card (`redpower_2:wool_card`): port of `ItemWoolCard`.
//!
//! Features:
//! - Teases wool blocks back into 4 string (`minecraft:string`).
//! - Retained as a crafting container item, losing 1 point of durability per craft.
//! - Max durability: 63 uses.

use crate::ItemStack;

/// Canonical item identifier for the Wool Card.
pub const WOOL_CARD_ID: &str = "redpower_2:wool_card";
/// Durability limit of the Wool Card (63 uses).
pub const WOOL_CARD_DURABILITY: u16 = 63;
/// String yield per teased wool block.
pub const STRING_PER_WOOL: u32 = 4;

/// Result of crafting a wool block with a wool card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WoolCardCraftResult {
    /// 4 string produced from teasing.
    pub string_output: ItemStack,
    /// Damaged wool card returned to crafting grid, or None if broken.
    pub remainder_card: Option<ItemStack>,
}

/// Checks if an item ID represents a wool block.
pub fn is_wool_block(item_id: &str) -> bool {
    let lower = item_id
        .strip_prefix("minecraft:")
        .unwrap_or(item_id)
        .to_ascii_lowercase();
    lower == "wool" || lower.ends_with("_wool")
}

/// Crafts 1 wool block using a Wool Card.
pub fn tease_wool(card: &ItemStack, wool: &ItemStack) -> Option<WoolCardCraftResult> {
    if card.item_id != WOOL_CARD_ID || !is_wool_block(&wool.item_id) {
        return None;
    }

    let next_damage = card.damage + 1;
    let remainder = if next_damage < WOOL_CARD_DURABILITY {
        let mut damaged_card = card.clone();
        damaged_card.damage = next_damage;
        damaged_card.count = 1;
        Some(damaged_card)
    } else {
        None
    };

    Some(WoolCardCraftResult {
        string_output: ItemStack::new("minecraft:string", STRING_PER_WOOL),
        remainder_card: remainder,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tease_wool_crafting() {
        let card = ItemStack::new(WOOL_CARD_ID, 1);
        let white_wool = ItemStack::new("minecraft:white_wool", 1);

        let res = tease_wool(&card, &white_wool).expect("should craft");
        assert_eq!(res.string_output.item_id, "minecraft:string");
        assert_eq!(res.string_output.count, 4);

        let rem = res.remainder_card.expect("card survives");
        assert_eq!(rem.damage, 1);
    }

    #[test]
    fn test_wool_card_breaks_at_max_durability() {
        let mut card = ItemStack::new(WOOL_CARD_ID, 1);
        card.damage = 62;
        let wool = ItemStack::new("minecraft:red_wool", 1);

        let res = tease_wool(&card, &wool).expect("should craft");
        assert_eq!(res.string_output.count, 4);
        assert!(res.remainder_card.is_none()); // Broken at 63 uses
    }

    #[test]
    fn test_non_wool_rejected() {
        let card = ItemStack::new(WOOL_CARD_ID, 1);
        let dirt = ItemStack::new("minecraft:dirt", 1);
        assert!(tease_wool(&card, &dirt).is_none());
    }
}
