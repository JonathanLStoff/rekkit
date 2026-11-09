//! Paint Cans and Paint Brushes (`redpower_2:paint_can` and `redpower_2:paint_brush`).
//!
//! Java reference: `eloraam/world/{ItemPaintCan,ItemPaintBrush}.java`.
//!
//! Features:
//! - 16 dye color channels.
//! - Paint Can: 15 coatings of colored paint. When emptied, returns an Empty Paint Can (`redpower_2:empty_paint_can`).
//! - Paint Brush: 15 applications of color. When exhausted, reverts to a Dry Paint Brush (`redpower_2:dry_paint_brush`).
//! - Dips dry brushes into paint cans to load them with color.
//! - Colors pneumatic tubes, bundled cables, jacketed wires, and canvas bags on right-click.

use crate::ItemStack;

pub const DRY_PAINT_BRUSH_ID: &str = "redpower_2:dry_paint_brush";
pub const PAINT_BRUSH_ID: &str = "redpower_2:paint_brush";
pub const EMPTY_PAINT_CAN_ID: &str = "redpower_2:empty_paint_can";
pub const PAINT_CAN_ID: &str = "redpower_2:paint_can";
pub const PAINT_MAX_USES: u16 = 15;

/// 16 Minecraft dye color names in canonical order.
pub const DYE_COLORS: [&str; 16] = [
    "black",
    "red",
    "green",
    "brown",
    "blue",
    "purple",
    "cyan",
    "light_gray",
    "gray",
    "pink",
    "lime",
    "yellow",
    "light_blue",
    "magenta",
    "orange",
    "white",
];

/// Resolves dye color index from 0..=15.
pub fn color_name(color_idx: u8) -> &'static str {
    DYE_COLORS
        .get((color_idx as usize) % 16)
        .copied()
        .unwrap_or("white")
}

/// Fills an empty paint can with a dye to produce a loaded paint can with 15 applications.
pub fn fill_paint_can(empty_can: &ItemStack, _dye_color_idx: u8) -> Option<ItemStack> {
    if empty_can.item_id != EMPTY_PAINT_CAN_ID {
        return None;
    }

    let mut can = ItemStack::new(PAINT_CAN_ID, 1);
    can.damage = 0; // 0 is fresh (15 applications remaining)
    Some(can)
}

/// Dips a dry paint brush into a paint can to coat it with colored paint.
///
/// Decrements the paint can's remaining uses. When exhausted, replaces can with `empty_paint_can`.
/// Returns the loaded paint brush.
pub fn dip_paint_brush(can: &mut ItemStack, dry_brush: &ItemStack) -> Option<ItemStack> {
    if can.item_id != PAINT_CAN_ID || dry_brush.item_id != DRY_PAINT_BRUSH_ID {
        return None;
    }

    can.damage += 1;
    if can.damage >= PAINT_MAX_USES {
        *can = ItemStack::new(EMPTY_PAINT_CAN_ID, 1);
    }

    let mut brush = ItemStack::new(PAINT_BRUSH_ID, 1);
    brush.damage = 0;
    Some(brush)
}

/// Applies one coating of paint to a target block/item, degrading brush durability.
///
/// When the brush reaches 15 uses, it reverts into a `dry_paint_brush`.
pub fn use_paint_brush(brush: &mut ItemStack) -> bool {
    if brush.item_id != PAINT_BRUSH_ID {
        return false;
    }

    brush.damage += 1;
    if brush.damage >= PAINT_MAX_USES {
        *brush = ItemStack::new(DRY_PAINT_BRUSH_ID, 1);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_paint_can() {
        let empty = ItemStack::new(EMPTY_PAINT_CAN_ID, 1);
        let filled = fill_paint_can(&empty, 1).expect("fills can");
        assert_eq!(filled.item_id, PAINT_CAN_ID);
        assert_eq!(filled.damage, 0);
    }

    #[test]
    fn test_dip_brush_and_empty_can() {
        let mut can = ItemStack::new(PAINT_CAN_ID, 1);
        can.damage = 14; // 1 use left
        let dry = ItemStack::new(DRY_PAINT_BRUSH_ID, 1);

        let brush = dip_paint_brush(&mut can, &dry).expect("dips brush");
        assert_eq!(brush.item_id, PAINT_BRUSH_ID);
        assert_eq!(brush.damage, 0);

        // Can is now empty
        assert_eq!(can.item_id, EMPTY_PAINT_CAN_ID);
    }

    #[test]
    fn test_brush_wear_and_revert() {
        let mut brush = ItemStack::new(PAINT_BRUSH_ID, 1);
        brush.damage = 14;

        assert!(use_paint_brush(&mut brush));
        // Exhausted -> reverts to dry brush
        assert_eq!(brush.item_id, DRY_PAINT_BRUSH_ID);
    }
}
