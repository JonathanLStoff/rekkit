//! Pneumatic tube (`TileTube`) with the in-flight item simulation of `TubeFlow`.
//!
//! Pure logic. Every tick each item gains `power + 16` progress; at 128 it is either scheduled
//! (its exit side chosen by the router) or, when already scheduled, handed to the neighbour. The
//! engine answers the questions in [`FlowContext`]. Loose ejection happens only when the tube has no
//! connection at all, otherwise an item that nobody takes bounces back inside the tube
//! (mode 2), so items never leak into the world through normal operation.

use super::magtube::{TubeItem, TUBE_BASE_STEP, TUBE_PROGRESS_DONE};
use crate::ItemStack;
use serde::{Deserialize, Serialize};

/// Normal routing mode (to an inventory or a tube that accepts in state 0).
pub const MODE_NORMAL: u8 = 1;
/// Bounce/overflow mode: the item found no taker and may only travel to state-2 acceptors.
pub const MODE_BOUNCE: u8 = 2;
/// Retrieved item: first try mode 3 routing, falling back to normal.
pub const MODE_RETRIEVE: u8 = 3;

/// What the world around a tube answers.
pub trait FlowContext {
    /// `TubeLib.findRoute(world, wc, item, sides, mode, lastDir)`: the exit side or -1.
    fn find_route(&mut self, ti: &TubeItem, sides: u8, mode: u8, last_dir: u8) -> i32;
    /// Hand the item to the neighbour across `ti.side`: a flow tube always takes it
    /// (`addTubeItem`), another connectable via `tubeItemEnter((side ^ 1), mode, ti)`.
    fn pass_to_tube(&mut self, ti: &TubeItem) -> bool;
    /// `handleItem`: `MachineLib.addToInventory` into the neighbour inventory.
    fn add_to_inventory(&mut self, ti: &TubeItem) -> bool;
    /// `MachineLib.ejectItem`: no connection at all, the item leaves as an entity.
    fn eject(&mut self, item: ItemStack);
}

/// Tube tile state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Tube {
    pub contents: Vec<TubeItem>,
    pub last_dir: u8,
    /// 0 unpainted, 1..16 dye.
    pub paint_color: u8,
    /// Per side, a cover id when fitted.
    pub covers: [Option<u16>; 6],
    /// `TileRestrictTube`: a tube whose `tubeWeight` is 1,000,000 (overflow route of last resort).
    #[serde(default)]
    pub restrictor: bool,
    #[serde(skip)]
    pub has_changed: bool,
}

fn lowest_bit(m: u8) -> u8 {
    m.trailing_zeros() as u8
}

impl Tube {
    /// `getTubeConnectableSides`.
    pub fn connectable_sides(&self) -> u8 {
        let mut tr = 63u8;
        for (i, c) in self.covers.iter().enumerate() {
            if matches!(c, Some(id) if id >> 8 < 3) {
                tr &= !(1 << i);
            }
        }
        tr
    }
    pub fn con_class(&self) -> i32 {
        self.paint_color as i32
    }
    /// A restrictor tube (`getExtendedID` 9 in the original) instead of a plain tube.
    pub fn new_restrictor() -> Self {
        Tube {
            restrictor: true,
            ..Tube::default()
        }
    }
    /// `tubeWeight`: 0 for a plain tube, 1,000,000 for `TileRestrictTube`.
    pub fn tube_weight(&self) -> i32 {
        if self.restrictor {
            1_000_000
        } else {
            0
        }
    }

    /// `TileTube.tryPaint`: paint the tube core (part 29) with dye `color` (1..16).
    pub fn try_paint(&mut self, part: i32, _side: u8, color: u8) -> bool {
        if part == 29 {
            if self.paint_color == color {
                return false;
            }
            self.paint_color = color;
            self.has_changed = true;
            true
        } else {
            false
        }
    }

    fn colour_ok(&self, ti: &TubeItem) -> bool {
        !(ti.color != 0 && self.paint_color != 0 && ti.color != self.paint_color)
    }
    /// `tubeItemCanEnter`.
    pub fn tube_item_can_enter(&self, state: u8, ti: &TubeItem) -> bool {
        self.colour_ok(ti) && state == 0
    }
    /// `tubeItemEnter`: only in state 0 and when the colours agree.
    pub fn tube_item_enter(&mut self, side: u8, state: u8, mut ti: TubeItem) -> bool {
        if state != 0 || !self.colour_ok(&ti) {
            return false;
        }
        ti.side = side;
        self.add(ti);
        true
    }
    /// `addTubeItem`: handed over by a neighbouring flow, which was travelling the other way.
    pub fn add_tube_item(&mut self, mut ti: TubeItem) {
        ti.side ^= 1;
        self.add(ti);
    }
    /// `TubeFlow.add`.
    pub fn add(&mut self, mut ti: TubeItem) {
        ti.progress = 0;
        ti.scheduled = false;
        self.contents.push(ti);
        self.has_changed = true;
    }

    fn advance_last_dir(&mut self, open: u8) {
        let m = (open as u32 & !((2u32 << self.last_dir) - 1)) as u8;
        let m = if m == 0 { open } else { m };
        self.last_dir = if m == 0 { 0 } else { lowest_bit(m) };
    }

    /// `flow.schedule`: choose the exit side. `cons` is `getConnections`.
    fn schedule<C: FlowContext>(&mut self, ti: &mut TubeItem, cons: u8, ctx: &mut C) {
        ti.scheduled = true;
        ti.progress = 0;
        let open = cons & !(1u8 << ti.side);
        if open == 0 {
            return;
        }
        if open.count_ones() == 1 {
            ti.side = lowest_bit(open);
            return;
        }
        if ti.mode != MODE_RETRIEVE {
            ti.mode = MODE_NORMAL;
        }
        let r = ctx.find_route(ti, open, ti.mode, self.last_dir);
        if r >= 0 {
            ti.side = r as u8;
            self.advance_last_dir(open);
            return;
        }
        if ti.mode == MODE_NORMAL && ti.priority > 0 {
            ti.priority = 0;
            let r = ctx.find_route(ti, cons, MODE_NORMAL, self.last_dir);
            if r >= 0 {
                ti.side = r as u8;
                return;
            }
        }
        let r = ctx.find_route(ti, cons, MODE_BOUNCE, self.last_dir);
        if r >= 0 {
            ti.side = r as u8;
            ti.mode = MODE_BOUNCE;
            return;
        }
        let mut r = -1;
        if ti.mode == MODE_RETRIEVE {
            r = ctx.find_route(ti, cons, MODE_NORMAL, self.last_dir);
            ti.mode = MODE_NORMAL;
        }
        if r >= 0 {
            ti.side = r as u8;
        } else {
            ti.side = self.last_dir;
            self.advance_last_dir(open);
        }
    }

    /// `TubeFlow.update` (server side). Returns whether anything changed.
    pub fn update<C: FlowContext>(&mut self, cons: u8, ctx: &mut C) -> bool {
        if self.contents.is_empty() {
            return false;
        }
        let mut changed = false;
        let mut done: Vec<TubeItem> = Vec::new();
        let items = std::mem::take(&mut self.contents);
        let mut keep = Vec::with_capacity(items.len());
        for mut ti in items {
            ti.progress += ti.power as i32 + TUBE_BASE_STEP;
            if ti.progress >= TUBE_PROGRESS_DONE {
                if ti.power > 0 {
                    ti.power -= 1;
                }
                changed = true;
                if !ti.scheduled {
                    self.schedule(&mut ti, cons, ctx);
                    keep.push(ti);
                } else {
                    done.push(ti);
                }
            } else {
                keep.push(ti);
            }
        }
        self.contents = keep;

        for mut ti in done {
            if ti.side < 6 && cons & (1 << ti.side) != 0 {
                if ctx.pass_to_tube(&ti) || ctx.add_to_inventory(&ti) {
                    continue;
                }
                ti.progress = 0;
                ti.scheduled = false;
                ti.mode = MODE_BOUNCE;
                self.contents.push(ti);
            } else if cons == 0 {
                ctx.eject(ti.item);
            } else {
                ti.side = lowest_bit(cons);
                ti.progress = TUBE_PROGRESS_DONE;
                ti.scheduled = false;
                self.contents.push(ti);
                changed = true;
            }
        }
        if changed {
            self.has_changed = true;
        }
        changed
    }

    /// `onRemove`: everything inside drops.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        self.contents
            .drain(..)
            .filter(|t| t.item.count > 0)
            .map(|t| t.item)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restrictor_weight() {
        assert_eq!(Tube::default().tube_weight(), 0);
        assert_eq!(Tube::new_restrictor().tube_weight(), 1_000_000);
    }

    fn stack(n: u32) -> ItemStack {
        ItemStack {
            item_id: "minecraft:stone".into(),
            count: n,
            damage: 0,
            max_stack_size: 64,
        }
    }

    #[derive(Default)]
    struct Ctx {
        routes: Vec<i32>,
        modes: Vec<u8>,
        tube_takes: bool,
        inv_takes: bool,
        passed: Vec<u8>,
        ejected: Vec<ItemStack>,
    }
    impl FlowContext for Ctx {
        fn find_route(&mut self, _: &TubeItem, _: u8, mode: u8, _: u8) -> i32 {
            self.modes.push(mode);
            if self.routes.is_empty() {
                -1
            } else {
                self.routes.remove(0)
            }
        }
        fn pass_to_tube(&mut self, ti: &TubeItem) -> bool {
            self.passed.push(ti.side);
            self.tube_takes
        }
        fn add_to_inventory(&mut self, _: &TubeItem) -> bool {
            self.inv_takes
        }
        fn eject(&mut self, item: ItemStack) {
            self.ejected.push(item);
        }
    }

    fn run(t: &mut Tube, cons: u8, c: &mut Ctx, ticks: usize) {
        for _ in 0..ticks {
            t.update(cons, c);
        }
    }

    #[test]
    fn straight_tube_takes_8_ticks_per_block() {
        let mut t = Tube::default();
        let mut c = Ctx {
            tube_takes: true,
            ..Default::default()
        };
        // entered from side 2 (north face), exits side 3; two connections only
        let cons = (1 << 2) | (1 << 3);
        assert!(t.tube_item_enter(2, 0, TubeItem::new(2, stack(1))));
        run(&mut t, cons, &mut c, 7);
        assert!(t.contents[0].progress < 128 && !t.contents[0].scheduled);
        run(&mut t, cons, &mut c, 1);
        assert!(t.contents[0].scheduled);
        assert_eq!(t.contents[0].side, 3);
        run(&mut t, cons, &mut c, 7);
        assert_eq!(t.contents.len(), 1);
        run(&mut t, cons, &mut c, 1);
        assert!(t.contents.is_empty());
        assert_eq!(c.passed, vec![3]);
        assert!(c.modes.is_empty(), "no routing needed with one exit");
    }

    #[test]
    fn junction_uses_router_and_rotates_last_dir() {
        let mut t = Tube::default();
        let mut c = Ctx {
            routes: vec![4],
            ..Default::default()
        };
        let cons = 0b111100; // sides 2..5
        t.tube_item_enter(2, 0, TubeItem::new(2, stack(1)));
        run(&mut t, cons, &mut c, 8);
        assert_eq!(t.contents[0].side, 4);
        assert_eq!(c.modes, vec![MODE_NORMAL]);
        assert_eq!(t.last_dir, 3, "next open side after 0 among 3,4,5");
    }

    #[test]
    fn no_taker_bounces_in_mode_two_without_ejecting() {
        let mut t = Tube::default();
        let mut c = Ctx::default();
        let cons = (1 << 2) | (1 << 3);
        t.tube_item_enter(2, 0, TubeItem::new(2, stack(1)));
        run(&mut t, cons, &mut c, 16);
        assert_eq!(t.contents.len(), 1);
        let ti = &t.contents[0];
        assert_eq!(ti.mode, MODE_BOUNCE);
        assert!(!ti.scheduled);
        assert_eq!(ti.progress, 0);
        assert!(c.ejected.is_empty());
    }

    #[test]
    fn failed_normal_route_falls_back_to_bounce_route() {
        let mut t = Tube::default();
        let mut c = Ctx {
            routes: vec![-1, 5],
            ..Default::default()
        };
        let cons = 0b111100;
        let mut ti = TubeItem::new(2, stack(1));
        ti.priority = 0;
        t.tube_item_enter(2, 0, ti);
        run(&mut t, cons, &mut c, 8);
        assert_eq!(c.modes, vec![MODE_NORMAL, MODE_BOUNCE]);
        assert_eq!(t.contents[0].side, 5);
        assert_eq!(t.contents[0].mode, MODE_BOUNCE);
    }

    #[test]
    fn priority_item_retries_over_all_connections() {
        let mut t = Tube::default();
        let mut c = Ctx {
            routes: vec![-1, 2],
            ..Default::default()
        };
        let mut ti = TubeItem::new(2, stack(1));
        ti.priority = 1;
        t.tube_item_enter(2, 0, ti);
        run(&mut t, 0b111100, &mut c, 8);
        assert_eq!(c.modes, vec![MODE_NORMAL, MODE_NORMAL]);
        assert_eq!(t.contents[0].side, 2);
        assert_eq!(t.contents[0].priority, 0);
    }

    #[test]
    fn dead_end_item_turns_round() {
        let mut t = Tube::default();
        let mut c = Ctx::default();
        t.tube_item_enter(2, 0, TubeItem::new(2, stack(1)));
        run(&mut t, 1 << 2, &mut c, 8);
        assert_eq!(t.contents[0].side, 2);
        assert!(t.contents[0].scheduled);
    }

    #[test]
    fn inventory_takes_item_at_the_end() {
        let mut t = Tube::default();
        let mut c = Ctx {
            inv_takes: true,
            ..Default::default()
        };
        let cons = (1 << 2) | (1 << 3);
        t.tube_item_enter(2, 0, TubeItem::new(2, stack(5)));
        run(&mut t, cons, &mut c, 16);
        assert!(t.contents.is_empty());
        assert!(c.ejected.is_empty());
    }

    #[test]
    fn disconnected_tube_ejects_and_lost_side_reroutes() {
        let mut t = Tube::default();
        let mut c = Ctx::default();
        let mut ti = TubeItem::new(3, stack(2));
        ti.scheduled = true;
        ti.progress = 127;
        ti.side = 3;
        t.contents.push(ti.clone());
        t.update(0, &mut c);
        assert_eq!(c.ejected.len(), 1);

        // exit side no longer connected but another is: turn to the lowest connection
        t.contents.push(ti);
        t.update(1 << 4, &mut c);
        assert_eq!(t.contents[0].side, 4);
        assert_eq!(t.contents[0].progress, 128);
        assert!(!t.contents[0].scheduled);
    }

    #[test]
    fn colour_rules() {
        let mut t = Tube::default();
        let mut red = TubeItem::new(0, stack(1));
        red.color = 14;
        assert!(t.tube_item_can_enter(0, &red));
        t.paint_color = 14;
        assert!(t.tube_item_can_enter(0, &red));
        t.paint_color = 3;
        assert!(!t.tube_item_can_enter(0, &red));
        assert!(!t.tube_item_enter(0, 0, red.clone()));
        let plain = TubeItem::new(0, stack(1));
        assert!(t.tube_item_enter(0, 0, plain));
        assert!(!t.tube_item_enter(0, 2, TubeItem::new(0, stack(1))));
    }

    #[test]
    fn power_speeds_items_and_decays() {
        let mut t = Tube::default();
        let mut c = Ctx {
            tube_takes: true,
            ..Default::default()
        };
        let mut ti = TubeItem::new(2, stack(1));
        ti.power = 112; // 112+16 = 128 per tick
        t.tube_item_enter(2, 0, ti);
        let cons = (1 << 2) | (1 << 3);
        t.update(cons, &mut c);
        assert_eq!(t.contents[0].power, 111);
        assert!(t.contents[0].scheduled);
    }

    #[test]
    fn add_tube_item_flips_side_and_removal_drops() {
        let mut t = Tube::default();
        t.add_tube_item(TubeItem::new(3, stack(4)));
        assert_eq!(t.contents[0].side, 2);
        assert_eq!(t.take_all_for_drop().len(), 1);
        assert!(t.contents.is_empty());
    }

    #[test]
    fn covers_seal_sides_and_state_round_trips() {
        let mut t = Tube::default();
        t.covers[1] = Some(0x0100);
        assert_eq!(t.connectable_sides(), 63 & !0b10);
        t.tube_item_enter(1, 0, TubeItem::new(1, stack(3)));
        let s = serde_json::to_string(&t).unwrap();
        let u: Tube = serde_json::from_str(&s).unwrap();
        assert_eq!(u.contents, t.contents);
    }
}
