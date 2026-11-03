//! Magtube (`TileMagTube`) and Accelerator (`TileAccel`) with the `TubeFlow` item stepping they share.
//!
//! Pure logic. A tube item advances `power + 16` progress per tick and crosses into the next block at
//! 128. A plain item has power 0 (8 ticks per block); the accelerator gives it power 255, which
//! decays by one per block, so a capsule crosses a block per tick at first. The engine layer hands
//! finished items to the neighbour tube and reads the voltage from the blutricity grid.

use crate::wiring::blutricity::BlueConductor;
use crate::ItemStack;
use serde::{Deserialize, Serialize};

/// Progress at which an item leaves the block (`TubeFlow.update`).
pub const TUBE_PROGRESS_DONE: i32 = 128;
/// Progress every item gains per tick regardless of power.
pub const TUBE_BASE_STEP: i32 = 16;
/// `TileMagTube.getSpeed()`.
pub const MAGTUBE_SPEED: i32 = 128;
/// Power given to an accelerated capsule.
pub const ACCEL_CAPSULE_POWER: i16 = 255;
/// Minimum voltage for the accelerator to work.
pub const ACCEL_MIN_VOLTAGE: f64 = 60.0;
/// W-t drawn per item in the stack.
pub const ACCEL_POWER_PER_ITEM: f64 = 100.0;
/// `getTubeConClass` of the accelerator; plain tubes are below, magtubes `18 + paint`.
pub const ACCEL_CON_CLASS: i32 = 17;
/// `getTubeConClass` base of a magtube.
pub const MAGTUBE_CON_CLASS_BASE: i32 = 18;
/// Cover id class (`Covers[i] >> 8`) below which a cover blocks the tube side.
pub const COVER_BLOCKING_CLASS: u16 = 3;
/// Extended id of `TileMagTube` / `TileAccel`.
pub const MAGTUBE_EXTENDED_ID: u8 = 11;
pub const ACCEL_EXTENDED_ID: u8 = 2;

/// Magtube centre part bounds, in sixteenths (`setPartBounds`, part 29).
pub const MAGTUBE_CORE_BOUNDS: [f32; 6] = [2.0, 2.0, 2.0, 14.0, 14.0, 14.0];

/// `TubeItem`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TubeItem {
    pub item: ItemStack,
    pub progress: i32,
    pub mode: u8,
    /// Side 0..5 (opposite is `side ^ 1`).
    pub side: u8,
    pub color: u8,
    pub power: i16,
    pub scheduled: bool,
    /// Routing priority (`TubeItem.priority`); 0 once the preferred pass failed.
    #[serde(default)]
    pub priority: i32,
}

impl TubeItem {
    pub fn new(side: u8, item: ItemStack) -> Self {
        Self {
            item,
            progress: 0,
            mode: 1,
            side,
            color: 0,
            power: 0,
            scheduled: false,
            priority: 0,
        }
    }
}

/// Magtube tile state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MagTube {
    /// Per side, the cover id when one is fitted.
    pub covers: [Option<u16>; 6],
    pub paint_color: u8,
}

impl MagTube {
    /// `getTubeConnectableSides`: all six, minus sides sealed by a low-class cover.
    pub fn connectable_sides(&self) -> u8 {
        let mut tr = 63u8;
        for (i, c) in self.covers.iter().enumerate() {
            if matches!(c, Some(id) if id >> 8 < COVER_BLOCKING_CLASS) {
                tr &= !(1 << i);
            }
        }
        tr
    }
    pub fn speed(&self) -> i32 {
        MAGTUBE_SPEED
    }
    pub fn con_class(&self) -> i32 {
        MAGTUBE_CON_CLASS_BASE + self.paint_color as i32
    }
}

/// `recache` bits: forward plain, forward magnetic, back plain, back magnetic.
pub const CON_FWD_TUBE: u8 = 1;
pub const CON_FWD_MAG: u8 = 2;
pub const CON_BACK_TUBE: u8 = 4;
pub const CON_BACK_MAG: u8 = 8;

/// Classify one neighbour by its tube connection class (`None` = no tube there).
pub fn neighbour_bits(class: Option<i32>, plain: u8, mag: u8) -> u8 {
    match class {
        Some(c) if c < ACCEL_CON_CLASS => plain,
        Some(_) => mag,
        None => 0,
    }
}

/// Accelerator tile state (`TileAccel`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accelerator {
    /// Facing side 0..5; it connects tubes on this side and the opposite.
    pub rotation: u8,
    pub contents: Vec<TubeItem>,
    pub charged: bool,
    pub has_changed: bool,
    /// Cached neighbour bits; `None` = needs `recache`.
    pub con_cache: Option<u8>,
    /// Not serialised with the tile; saved with [`Accelerator::save_cond`].
    #[serde(skip, default = "BlueConductor::panel")]
    pub cond: BlueConductor,
}

impl Accelerator {
    pub fn new(rotation: u8) -> Self {
        Self {
            rotation,
            contents: Vec::new(),
            charged: false,
            has_changed: false,
            con_cache: None,
            cond: BlueConductor::panel(),
        }
    }

    /// `getTubeConnectableSides`: `3 << (Rotation & 6)`.
    pub fn connectable_sides(&self) -> u8 {
        3 << (self.rotation & 6)
    }
    pub fn con_class(&self) -> i32 {
        ACCEL_CON_CLASS
    }
    pub fn light_value(&self) -> u8 {
        if self.charged {
            6
        } else {
            0
        }
    }

    /// `recache`, given the connection classes of the forward and backward neighbours.
    pub fn recache(&mut self, forward: Option<i32>, backward: Option<i32>) {
        self.con_cache = Some(
            neighbour_bits(forward, CON_FWD_TUBE, CON_FWD_MAG)
                | neighbour_bits(backward, CON_BACK_TUBE, CON_BACK_MAG),
        );
    }
    /// A neighbour changed: the cache is rebuilt on the next use.
    pub fn on_neighbor_change(&mut self) {
        self.con_cache = None;
    }

    /// `tubeItemEnter`: only in state 0 and through the two axis faces.
    pub fn tube_item_enter(&mut self, side: u8, state: u8, mut ti: TubeItem) -> bool {
        if state != 0 || (side != self.rotation && side != (self.rotation ^ 1)) {
            return false;
        }
        ti.side = side;
        self.add(ti);
        true
    }
    pub fn tube_item_can_enter(&self, state: u8) -> bool {
        state == 0
    }
    pub fn tube_weight(&self) -> u32 {
        0
    }
    /// `addTubeItem`: an item handed over by a neighbouring flow.
    pub fn add_tube_item(&mut self, mut ti: TubeItem) {
        ti.side ^= 1;
        self.add(ti);
    }
    fn add(&mut self, mut ti: TubeItem) {
        ti.progress = 0;
        ti.scheduled = false;
        self.contents.push(ti);
        self.has_changed = true;
    }

    /// `flow.schedule`: turn the item round and, if the exit leads to a magtube and the grid holds at
    /// least 60 V, draw 100 W-t per item and give it capsule power.
    fn schedule(&mut self, ti: &mut TubeItem, tick: u64) {
        ti.scheduled = true;
        ti.progress = 0;
        ti.side ^= 1;
        if self.con_cache.is_none() {
            self.recache(None, None);
        }
        let cc = self.con_cache.unwrap_or(0);
        ti.power = 0;
        let leads_mag = (ti.side == self.rotation && cc & CON_FWD_MAG != 0)
            || (ti.side == (self.rotation ^ 1) && cc & CON_BACK_MAG != 0);
        if leads_mag && self.cond.voltage(tick) >= ACCEL_MIN_VOLTAGE {
            self.cond
                .draw_power(tick, ACCEL_POWER_PER_ITEM * ti.item.count as f64);
            ti.power = ACCEL_CAPSULE_POWER;
        }
    }

    /// `TubeFlow.update` + the charge light. Returns items that finished and must go to the neighbour
    /// on `ti.side`.
    pub fn tick(&mut self, tick: u64) -> Vec<TubeItem> {
        let mut out = Vec::new();
        let mut keep = Vec::with_capacity(self.contents.len());
        let items = std::mem::take(&mut self.contents);
        for mut ti in items {
            ti.progress += ti.power as i32 + TUBE_BASE_STEP;
            if ti.progress >= TUBE_PROGRESS_DONE {
                if ti.power > 0 {
                    ti.power -= 1;
                }
                self.has_changed = true;
                if !ti.scheduled {
                    self.schedule(&mut ti, tick);
                    keep.push(ti);
                } else {
                    out.push(ti);
                }
            } else {
                keep.push(ti);
            }
        }
        self.contents = keep;
        self.cond.endpoint_update(tick);
        let flowing = self.cond.flow != 0;
        if flowing != self.charged {
            self.charged = flowing;
        }
        out
    }

    /// Save the conductor's charge (NBT-style key/value map).
    pub fn save_cond(&self, nbt: &mut std::collections::HashMap<String, String>) {
        self.cond.save(nbt);
    }
    pub fn load_cond(&mut self, nbt: &std::collections::HashMap<String, String>) {
        self.cond.load(nbt);
    }

    /// `onRemove`: everything inside drops.
    pub fn take_all_for_drop(&mut self) -> Vec<ItemStack> {
        self.contents.drain(..).map(|t| t.item).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(n: u32) -> ItemStack {
        ItemStack {
            item_id: "minecraft:stone".into(),
            count: n,
            damage: 0,
            max_stack_size: 64,
        }
    }

    #[test]
    fn magtube_sides_and_class() {
        let mut m = MagTube::default();
        assert_eq!(m.connectable_sides(), 63);
        m.covers[1] = Some(0x0100);
        m.covers[2] = Some(0x0400);
        assert_eq!(m.connectable_sides(), 63 & !0b10);
        m.paint_color = 5;
        assert_eq!(m.con_class(), 23);
        assert_eq!(m.speed(), 128);
    }

    #[test]
    fn accelerator_axis() {
        assert_eq!(Accelerator::new(0).connectable_sides(), 0b11);
        assert_eq!(Accelerator::new(3).connectable_sides(), 0b1100);
        assert_eq!(Accelerator::new(5).connectable_sides(), 0b110000);
    }

    #[test]
    fn enter_rules() {
        let mut a = Accelerator::new(2);
        assert!(!a.tube_item_enter(0, 0, TubeItem::new(0, stack(1))));
        assert!(!a.tube_item_enter(2, 1, TubeItem::new(2, stack(1))));
        assert!(a.tube_item_enter(3, 0, TubeItem::new(3, stack(1))));
        assert_eq!(a.contents.len(), 1);
    }

    #[test]
    fn unpowered_item_passes_at_plain_speed() {
        let mut a = Accelerator::new(2);
        a.recache(Some(18), Some(3));
        a.tube_item_enter(3, 0, TubeItem::new(3, stack(4)));
        // first leg: 128/16 = 8 ticks to schedule
        for t in 0..7 {
            assert!(a.tick(t).is_empty());
        }
        assert!(a.tick(7).is_empty());
        assert!(a.contents[0].scheduled);
        assert_eq!(a.contents[0].side, 2);
        assert_eq!(a.contents[0].power, 0);
    }

    #[test]
    fn powered_item_is_accelerated_and_draws_power() {
        let mut a = Accelerator::new(2);
        a.recache(Some(18), Some(3));
        a.cond.vcap = 100.0;
        a.cond.last_tick = 1;
        a.tube_item_enter(3, 0, TubeItem::new(3, stack(2)));
        let mut done = Vec::new();
        for t in 0..20 {
            done.extend(a.tick(t));
        }
        assert_eq!(done.len(), 1);
        // first leg 8 ticks, second leg 1 tick at power 255 -> done well within 20 ticks
        assert_eq!(done[0].power, ACCEL_CAPSULE_POWER - 1);
        assert!(a.cond.vcap < 100.0 || a.cond.icap != 0.0);
    }

    #[test]
    fn low_voltage_or_plain_exit_gives_no_power() {
        let mut a = Accelerator::new(2);
        a.recache(Some(3), None); // forward is a plain tube
        a.cond.vcap = 100.0;
        a.cond.last_tick = 1;
        a.tube_item_enter(3, 0, TubeItem::new(3, stack(1)));
        for t in 0..8 {
            a.tick(t);
        }
        assert_eq!(a.contents[0].power, 0);

        let mut b = Accelerator::new(2);
        b.recache(Some(18), None);
        b.cond.vcap = 59.0;
        b.cond.last_tick = 1;
        b.tube_item_enter(3, 0, TubeItem::new(3, stack(1)));
        for t in 0..8 {
            b.tick(t);
        }
        assert_eq!(b.contents[0].power, 0);
    }

    #[test]
    fn state_round_trips() {
        let mut a = Accelerator::new(4);
        a.charged = true;
        a.tube_item_enter(4, 0, TubeItem::new(4, stack(3)));
        let s = serde_json::to_string(&a).unwrap();
        let b: Accelerator = serde_json::from_str(&s).unwrap();
        assert!(b.charged);
        assert_eq!(b.contents, a.contents);
    }
}
