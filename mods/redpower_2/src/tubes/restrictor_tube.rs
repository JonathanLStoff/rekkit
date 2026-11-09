//! Restrictor Tube (`TileRestrictTube`): pneumatic tube with high traversal routing weight.
//!
//! Subtype of pneumatic tube that imposes +1,000,000 Dijkstra traversal weight
//! (`RESTRICTOR_TUBE_WEIGHT`), designating routes across it as overflow paths of last resort.
//!
//! Matches original Java `com.eloraam.redpower.machine.TileRestrictTube`.

use crate::machine::magtube::TubeItem;
use crate::machine::tube::TubeAnswer;
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

    /// `canRouteItems`: returns true for restrictor tubes.
    pub fn can_route_items(&self) -> bool {
        true
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

    /// `tubeItemCanEnter(side, state, ti)`: checks color filtering and idle state (Java signature).
    pub fn tube_item_can_enter_side(&self, _side: u8, state: u8, ti: &TubeItem) -> bool {
        self.tube.tube_item_can_enter(state, ti)
    }

    /// Returns a [`TubeAnswer`] for routing query integration.
    pub fn tube_answer(&self, ti: &TubeItem, state: u8) -> TubeAnswer {
        TubeAnswer {
            can_enter_state: self.tube_item_can_enter(state, ti),
            can_enter_idle: self.tube_item_can_enter(0, ti),
            can_route: self.can_route_items(),
            weight: self.tube_weight(0, state),
        }
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
    use crate::machine::tube::{find_route_full, OutRules, Pos};
    use std::collections::HashMap;

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
        assert!(rt.can_route_items());
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
        assert!(rt.tube_item_can_enter_side(1, 0, &red_item));

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

        let ans = rt.tube_answer(&red_item, 0);
        assert!(ans.can_enter_state);
        assert!(ans.can_route);
        assert_eq!(ans.weight, RESTRICTOR_TUBE_WEIGHT);
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

    struct TubeNetwork {
        tubes: HashMap<Pos, TubeAnswer>,
        chest: Pos,
    }

    impl OutRules for TubeNetwork {
        fn inventory_accepts(&self, pos: Pos, _s: i32) -> bool {
            pos == self.chest
        }
        fn tube_at(&self, pos: Pos, _s: i32) -> Option<TubeAnswer> {
            self.tubes.get(&pos).copied()
        }
        fn connections(&self, pos: Pos) -> u8 {
            let mut cons = 0;
            for s in 0..6 {
                let p = crate::machine::tube::step(pos, s);
                if self.tubes.contains_key(&p) || p == self.chest {
                    cons |= 1 << s;
                }
            }
            cons
        }
    }

    #[test]
    fn test_restrictor_tube_routing_priority() {
        let test_item = dummy_item(1, 0);
        let normal_tube = Tube::default();
        let normal_ans = TubeAnswer {
            can_enter_state: true,
            can_enter_idle: true,
            can_route: true,
            weight: normal_tube.tube_weight(),
        };

        let restrictor = RestrictorTube::new();
        let restrictor_ans = restrictor.tube_answer(&test_item, 0);
        assert_eq!(restrictor_ans.weight, 1_000_000);

        // Network topology:
        // Source is at (0, 0, 0).
        // Destination chest is at (3, 0, 0).
        // Path A: through (1, 0, 0) which is a RestrictorTube, then (2, 0, 0) normal tube to chest. (2 hops, weight 1_000_004)
        // Path B: detour through (0, 1, 0) -> (1, 1, 0) -> (2, 1, 0) -> (3, 1, 0) -> (3, 0, 0) chest. (5 hops, weight ~10)
        let mut tubes = HashMap::new();
        // Path A (short with restrictor)
        tubes.insert((1, 0, 0), restrictor_ans);
        tubes.insert((2, 0, 0), normal_ans);

        // Path B (long normal detour)
        tubes.insert((0, 1, 0), normal_ans);
        tubes.insert((1, 1, 0), normal_ans);
        tubes.insert((2, 1, 0), normal_ans);
        tubes.insert((3, 1, 0), normal_ans);

        let net = TubeNetwork {
            tubes: tubes.clone(),
            chest: (3, 0, 0),
        };

        // Router at (0, 0, 0) has open sides +X (side 5) and +Y (side 1)
        let (dir, route) = find_route_full(&net, (0, 0, 0), (1 << 5) | (1 << 1), 0).unwrap();
        // Dijkstra must choose Path B (+Y, side 1), avoiding the restrictor tube!
        assert_eq!(dir, 1);
        assert!(route.weight < 1_000_000);

        // Now remove Path B to simulate overflow / path blocked
        let mut overflow_tubes = HashMap::new();
        overflow_tubes.insert((1, 0, 0), restrictor_ans);
        overflow_tubes.insert((2, 0, 0), normal_ans);

        let overflow_net = TubeNetwork {
            tubes: overflow_tubes,
            chest: (3, 0, 0),
        };

        // When only restrictor path exists, router must traverse it as overflow
        let (overflow_dir, overflow_route) =
            find_route_full(&overflow_net, (0, 0, 0), 1 << 5, 5).unwrap();
        assert_eq!(overflow_dir, 5);
        assert!(overflow_route.weight >= 1_000_000);
    }
}
