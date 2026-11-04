//! Redstone Tube (`TileRedstoneTube`): pneumatic tube with integrated redstone signal conduction.
//!
//! Subtype of pneumatic tube that transmits redstone signals alongside items and powers adjacent
//! blocks, implementing `IRedPowerWiring` with Red Alloy wire connection class (1).
//!
//! Matches original Java `com.eloraam.redpower.machine.TileRedstoneTube`.

use crate::machine::magtube::TubeItem;
use crate::machine::tube_flow::{FlowContext, Tube};
use crate::wiring::network::{WireKind, WireNode, CHANNELS};
use crate::ItemStack;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Extended ID of `TileRedstoneTube` in RedPower (`getExtendedID() == 9`).
pub const REDSTONE_TUBE_EXTENDED_ID: u8 = 9;

/// Connection class for Red Alloy wire wiring (`getConnectClass == 1`).
pub const REDSTONE_TUBE_WIRING_CLASS: i32 = 1;

/// Central wire connection shift (bits 24..29 represent center-wire faces).
pub const CENTER_WIRE_SHIFT: u32 = 24;

/// Mask of all 6 central-wire connections (bits 24..29).
pub const CENTER_WIRE_MASK: u32 = 0x3F << CENTER_WIRE_SHIFT;

/// Redstone Tube tile state combining pneumatic item transport and red alloy signal conduction.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RedstoneTube {
    /// Inner pneumatic tube state (contents, paint_color, covers, last_dir).
    pub tube: Tube,
    /// Held redstone signal strength 0..=255 (`PowerState`).
    pub power_state: u8,
    /// Direct connection mask cache (`ConMask`), -1 when uncalculated/dirty.
    pub con_mask: i32,
}

impl Deref for RedstoneTube {
    type Target = Tube;

    fn deref(&self) -> &Self::Target {
        &self.tube
    }
}

impl DerefMut for RedstoneTube {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tube
    }
}

impl RedstoneTube {
    /// Creates a new unpainted, unpowered Redstone Tube.
    pub fn new() -> Self {
        Self {
            tube: Tube::default(),
            power_state: 0,
            con_mask: -1,
        }
    }

    /// Creates a Redstone Tube with an initial paint dye color (1..=16).
    pub fn new_painted(paint_color: u8) -> Self {
        let mut t = Tube::default();
        t.paint_color = paint_color;
        Self {
            tube: t,
            power_state: 0,
            con_mask: -1,
        }
    }

    /// `getExtendedID`: returns 9 for `TileRedstoneTube`.
    pub fn extended_id(&self) -> u8 {
        REDSTONE_TUBE_EXTENDED_ID
    }

    // --- Pneumatic Tube delegates ---

    /// `tubeWeight(side, state)`: 0 for Redstone Tube (standard routing).
    pub fn tube_weight(&self, _side: u8, _state: u8) -> i32 {
        0
    }

    /// `getTubeConnectableSides`: sides free of blocking covers (covers id >> 8 < 3).
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

    // --- IRedPowerWiring Implementation ---

    /// `getConnectableMask`: returns the 30-bit connectable mask for wiring.
    /// In RedPower, bits 24..29 represent central connections facing the 6 sides (0..=5).
    pub fn get_connectable_mask(&self) -> u32 {
        let mut tr = 63u32;
        for (i, c) in self.tube.covers.iter().enumerate() {
            if matches!(c, Some(id) if id >> 8 < 3) {
                tr &= !(1 << i);
            }
        }
        tr << CENTER_WIRE_SHIFT
    }

    /// `getConnectionMask`: direct connection mask; returns cached `con_mask` or computes from connectable mask.
    pub fn get_connection_mask(&self) -> u32 {
        if self.con_mask >= 0 {
            self.con_mask as u32
        } else {
            self.get_connectable_mask()
        }
    }

    /// Sets the cached connection mask (`ConMask`).
    pub fn set_connection_mask(&mut self, mask: i32) {
        self.con_mask = mask;
    }

    /// Invalidates the cached connection mask when neighbors change.
    pub fn invalidate_connection_mask(&mut self) {
        self.con_mask = -1;
    }

    /// `getExtConnectionMask`: always 0 (tubes do not wrap around external block corners).
    pub fn get_ext_connection_mask(&self) -> u32 {
        0
    }

    /// `getCornerPowerMode`: always 0.
    pub fn get_corner_power_mode(&self) -> i32 {
        0
    }

    /// `getConnectClass(side)`: returns 1 (Red Alloy Wire connection class).
    pub fn get_connect_class(&self, _side: u8) -> i32 {
        REDSTONE_TUBE_WIRING_CLASS
    }

    /// `getCurrentStrength(cons, ch)`: returns current strength on channel 0.
    /// Channel 0 is Red Alloy wire; other channels return -1.
    pub fn get_current_strength(&self, cons: u32, ch: usize) -> i32 {
        if ch != 0 {
            -1
        } else if (cons & self.get_connectable_mask()) == 0 {
            -1
        } else {
            self.power_state as i32
        }
    }

    /// `scanPoweringStrength(cons, ch)`: returns 255 if powered externally, else 0.
    pub fn scan_powering_strength(&self, is_powered: bool, ch: usize) -> i32 {
        if ch != 0 {
            0
        } else if is_powered {
            255
        } else {
            0
        }
    }

    /// `updateCurrentStrength`: updates the held power state (0..=255).
    pub fn update_current_strength(&mut self, strength: i32) {
        self.power_state = strength.clamp(0, 255) as u8;
    }

    /// `getPoweringMask(ch)`: returns `getConnectableMask()` on channel 0 when powered, else 0.
    pub fn get_powering_mask(&self, ch: usize) -> u32 {
        if ch == 0 && self.power_state != 0 {
            self.get_connectable_mask()
        } else {
            0
        }
    }

    /// `isBlockWeakPoweringTo(side)`: determines whether this tube powers an adjacent block on `side`.
    ///
    /// Following RedPowerLib:
    /// - If `is_searching`: returns false (avoids feedback loops during search).
    /// - If there is no connection towards `side ^ 1`: returns false.
    /// - If adjacent block is redstone wire/component: returns `power_state > 15`.
    /// - If adjacent block is another receiver (door, piston, lamp): returns `power_state > 0`.
    pub fn is_block_weak_powering_to(
        &self,
        side: u8,
        is_searching: bool,
        is_redstone_component: bool,
    ) -> bool {
        if is_searching {
            return false;
        }
        let opp_side = side ^ 1;
        let con_bit = 1u32 << (CENTER_WIRE_SHIFT + opp_side as u32);
        if (self.get_connection_mask() & con_bit) == 0 {
            return false;
        }
        if is_redstone_component {
            self.power_state > 15
        } else {
            self.power_state > 0
        }
    }

    /// Vanilla redstone signal output (0..=15).
    pub fn vanilla_power(&self, side: u8, is_redstone_component: bool) -> i32 {
        if self.is_block_weak_powering_to(side, false, is_redstone_component) {
            ((self.power_state as i32 + 16) / 17).clamp(1, 15)
        } else {
            0
        }
    }

    /// Converts this Redstone Tube into a `WireNode` for integration with `WireWorld`.
    pub fn as_wire_node(&self) -> WireNode {
        let mask = self.get_connectable_mask();
        let mut state = [0i32; CHANNELS];
        state[0] = self.power_state as i32;
        let mut node = WireNode::placed(WireKind::Red, mask);
        node.connection_mask = self.get_connection_mask();
        node.state = state;
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_item(count: u32, color: u8) -> TubeItem {
        let mut ti = TubeItem::new(
            0,
            ItemStack {
                item_id: "minecraft:redstone".into(),
                count,
                damage: 0,
                max_stack_size: 64,
            },
        );
        ti.color = color;
        ti
    }

    #[test]
    fn test_redstone_tube_constants_and_id() {
        let rt = RedstoneTube::new();
        assert_eq!(rt.extended_id(), 9);
        assert_eq!(rt.tube_weight(0, 0), 0);
        assert_eq!(rt.get_connect_class(0), 1);
        assert_eq!(rt.get_ext_connection_mask(), 0);
        assert_eq!(rt.get_corner_power_mode(), 0);
    }

    #[test]
    fn test_connectable_mask_and_covers() {
        let mut rt = RedstoneTube::new();
        // Default: all 6 central-wire directions open (63 << 24)
        assert_eq!(rt.get_connectable_mask(), 63 << 24);

        // Add a cover on side 0 that blocks (cover id 0 has id >> 8 == 0 < 3)
        rt.tube.covers[0] = Some(0);
        assert_eq!(rt.get_connectable_mask(), 62 << 24);

        // Add a hollow cover on side 1 (cover id 0x0300 has id >> 8 == 3 >= 3, hollow covers do not block tubes)
        rt.tube.covers[1] = Some(0x0300);
        assert_eq!(rt.get_connectable_mask(), 62 << 24);
    }

    #[test]
    fn test_power_strength_and_channels() {
        let mut rt = RedstoneTube::new();
        rt.update_current_strength(170);
        assert_eq!(rt.power_state, 170);

        let cons = rt.get_connectable_mask();
        // Channel 0 returns held power
        assert_eq!(rt.get_current_strength(cons, 0), 170);
        // Other channels return -1
        assert_eq!(rt.get_current_strength(cons, 1), -1);
        assert_eq!(rt.get_current_strength(cons, 16), -1);

        // Mask not intersecting connectable returns -1
        assert_eq!(rt.get_current_strength(0, 0), -1);

        // Powering mask
        assert_eq!(rt.get_powering_mask(0), cons);
        assert_eq!(rt.get_powering_mask(1), 0);

        // When unpowered, powering mask is 0
        rt.update_current_strength(0);
        assert_eq!(rt.get_powering_mask(0), 0);
    }

    #[test]
    fn test_is_block_weak_powering_to() {
        let mut rt = RedstoneTube::new();
        rt.update_current_strength(100);

        // When searching, power check is false
        assert!(!rt.is_block_weak_powering_to(0, true, false));

        // For non-redstone block (solid block / door / lamp), power > 0 powers it
        assert!(rt.is_block_weak_powering_to(0, false, false));
        assert!(rt.vanilla_power(0, false) > 0);

        // For redstone dust / repeater: power > 15 powers it
        assert!(rt.is_block_weak_powering_to(0, false, true));

        // If power <= 15:
        rt.update_current_strength(15);
        // Non-redstone receiver still receives weak power
        assert!(rt.is_block_weak_powering_to(0, false, false));
        // Redstone component does not receive power when strength <= 15
        assert!(!rt.is_block_weak_powering_to(0, false, true));

        // When completely unpowered:
        rt.update_current_strength(0);
        assert!(!rt.is_block_weak_powering_to(0, false, false));
        assert_eq!(rt.vanilla_power(0, false), 0);
    }

    #[test]
    fn test_paint_coloring_rules() {
        let mut rt = RedstoneTube::new();
        assert_eq!(rt.paint_color, 0);

        let uncolored = dummy_item(1, 0);
        let yellow = dummy_item(1, 12);

        // Can enter unpainted
        assert!(rt.tube_item_can_enter(0, &uncolored));
        assert!(rt.tube_item_can_enter(0, &yellow));

        // Paint with yellow (dye 12)
        assert!(rt.try_paint(29, 0, 12));
        assert_eq!(rt.paint_color, 12);
        assert_eq!(rt.con_class(), 12);

        // Matching items can enter; other colors cannot
        let red = dummy_item(1, 14);
        assert!(rt.tube_item_can_enter(0, &uncolored));
        assert!(rt.tube_item_can_enter(0, &yellow));
        assert!(!rt.tube_item_can_enter(0, &red));
    }

    #[test]
    fn test_wire_node_conversion() {
        let mut rt = RedstoneTube::new();
        rt.update_current_strength(255);
        let node = rt.as_wire_node();
        assert_eq!(node.kind, WireKind::Red);
        assert_eq!(node.state[0], 255);
        assert_eq!(node.connect_class(), 1);
        assert_eq!(node.connectable_mask, 63 << 24);
    }
}
