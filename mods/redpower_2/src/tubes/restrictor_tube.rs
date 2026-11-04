//! Restrictor Tube (`TileRestrictTube`): pneumatic tube with high traversal routing weight.
//!
//! Subtype of pneumatic tube that imposes +1,000,000 Dijkstra traversal weight
//! (`RESTRICTOR_TUBE_WEIGHT`), designating routes across it as overflow paths of last resort.
//!
//! Matches original Java `com.eloraam.redpower.machine.TileRestrictTube`.

use crate::machine::magtube::TubeItem;
use crate::machine::tube_flow::{FlowContext, Tube};
use crate::ItemStack;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Extended ID of `TileRestrictTube` in RedPower (`getExtendedID() == 10`).
pub const RESTRICTOR_TUBE_EXTENDED_ID: u8 = 10;

/// Routing weight added by a Restrictor Tube (+1,000,000).
pub const RESTRICTOR_TUBE_WEIGHT: i32 = 1_000_000;

/// Restrictor Tube tile state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestrictorTube {
    /// Inner pneumatic tube state with `restrictor = true`.
    pub tube: Tube,
}

impl Default for RestrictorTube {
    fn default() -> Self {
        Self {
            tube: Tube::new_restrictor(),
        }
    }
}

impl Deref for RestrictorTube {
    type Target = Tube;

    fn deref(&self) -> &Self::Target {
        &self.tube
    }
}

impl DerefMut for RestrictorTube {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tube
    }
}

impl RestrictorTube {
    /// Creates a new unpainted Restrictor Tube.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Restrictor Tube with an initial paint dye color (1..=16).
    pub fn new_painted(paint_color: u8) -> Self {
        let mut t = Tube::new_restrictor();
        t.paint_color = paint_color;
        Self { tube: t }
    }

    /// `getExtendedID`: returns 10 for `TileRestrictTube`.
    pub fn extended_id(&self) -> u8 {
        RESTRICTOR_TUBE_EXTENDED_ID
    }

    /// `tubeWeight(side, state)`: returns 1,000,000 for Restrictor Tube.
    pub fn tube_weight(&self, _side: u8, _state: u8) -> i32 {
        RESTRICTOR_TUBE_WEIGHT
    }

    /// `getTubeConnectableSides`: bitmask of sides that can connect (sides without blocking covers).
    pub fn connectable_sides(&self) -> u8 {
        self.tube.connectable_sides()
    }

    /// `getTubeConClass`: returns the dye color (0 unpainted, 1..=16 painted).
    pub fn con_class(&self) -> i32 {
        self.tube.con_class()
    }

    /// `tubeItemCanEnter`: verifies dye color matching and idle state.
    pub fn tube_item_can_enter(&self, state: u8, ti: &TubeItem) -> bool {
        self.tube.tube_item_can_enter(state, ti)
    }

    /// `tubeItemEnter`: enters item if state is 0 and dye colors match.
    pub fn tube_item_enter(&mut self, side: u8, state: u8, ti: TubeItem) -> bool {
        self.tube.tube_item_enter(side, state, ti)
    }

    /// `addTubeItem`: passes item across boundary (flips side).
    pub fn add_tube_item(&mut self, ti: TubeItem) {
        self.tube.add_tube_item(ti);
    }

    /// `tryPaint`: paints part 29 (tube core) with dye `color` (1..=16).
    pub fn try_paint(&mut self, part: i32, side: u8, color: u8) -> bool {
        self.tube.try_paint(part, side, color)
    }

    /// Advances tube flow simulation for one tick.
    pub fn update<C: FlowContext>(&mut self, cons: u8, ctx: &mut C) -> bool {
        self.tube.update(cons, ctx)
    }

    /// Ejects contents on block break.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        self.tube.take_all_for_drop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_item(count: u32, color: u8) -> TubeItem {
        let mut ti = TubeItem::new(
            0,
            ItemStack {
                item_id: "minecraft:iron_ingot".into(),
                count,
                damage: 0,
                max_stack_size: 64,
            },
        );
        ti.color = color;
        ti
    }

    #[test]
    fn test_restrictor_tube_weight_and_id() {
        let rt = RestrictorTube::new();
        assert_eq!(rt.extended_id(), 10);
        assert_eq!(rt.tube_weight(0, 0), 1_000_000);
        assert!(rt.restrictor);
    }

    #[test]
    fn test_paint_and_color_filtering() {
        let mut rt = RestrictorTube::new();
        assert_eq!(rt.paint_color, 0);
        assert_eq!(rt.con_class(), 0);

        // Can enter unpainted tube with any color
        let uncolored = dummy_item(1, 0);
        let red_item = dummy_item(1, 14); // 14 = red
        assert!(rt.tube_item_can_enter(0, &uncolored));
        assert!(rt.tube_item_can_enter(0, &red_item));

        // Painting invalid part fails
        assert!(!rt.try_paint(0, 0, 14));
        assert_eq!(rt.paint_color, 0);

        // Painting part 29 succeeds
        assert!(rt.try_paint(29, 0, 14));
        assert_eq!(rt.paint_color, 14);
        assert_eq!(rt.con_class(), 14);

        // Painting identical color returns false
        assert!(!rt.try_paint(29, 0, 14));

        // Matching or uncolored items can enter; different color cannot
        let blue_item = dummy_item(1, 4); // 4 = blue
        assert!(rt.tube_item_can_enter(0, &uncolored));
        assert!(rt.tube_item_can_enter(0, &red_item));
        assert!(!rt.tube_item_can_enter(0, &blue_item));
    }

    #[test]
    fn test_item_entry_and_drop() {
        let mut rt = RestrictorTube::new();
        let item = dummy_item(5, 0);
        assert!(rt.tube_item_enter(2, 0, item));
        assert_eq!(rt.contents.len(), 1);
        assert_eq!(rt.contents[0].side, 2);

        let drops = rt.take_all_for_drop();
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].count, 5);
        assert!(rt.contents.is_empty());
    }
}
