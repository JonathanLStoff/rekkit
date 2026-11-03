//! Grate (`redpower_2:grate`): port of `TileGrate` (machine panel meta 3) and its `GratePathfinder`.
//!
//! Pure logic. Under suction (pressure below -100) the grate maps the connected body of
//! liquid in front of it and drains it into its buffer; under pressure (above 100) it
//! pours the buffer into the nearest free spot of a connected body. The engine layer
//! implements [`GrateWorld`] for the real blocks.

use super::pipe::{FluidBuffer, FluidBufferTag, PIPE_MAX_LEVEL, PRESSURE_FLOW};
use crate::WorldCoord;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// Ticks between grate work steps (`scheduleTick(5)`).
pub const GRATE_TICK_DELAY: u32 = 5;
/// Pressure beyond which the grate acts.
pub const GRATE_THRESHOLD: i32 = 100;
/// Search steps per tick while mapping a body of liquid.
pub const MAP_TRIES: usize = 400;
/// Search steps per tick while looking for a spot to pour.
pub const DUMP_TRIES: usize = 2000;
/// Mapping stops growing past this many known source cells.
pub const MAP_LIMIT: usize = 32768;
/// Level of a full liquid block.
pub const FULL_BLOCK: i32 = 1000;

/// The world as the grate sees it (`PipeLib.getLiquidId` and `FluidClass`).
pub trait GrateWorld {
    /// `PipeLib.getLiquidId`: fluid id of the liquid block at `pos`, 0 if none.
    fn liquid_id(&self, pos: WorldCoord) -> i32;
    /// `FluidClass.getFluidId` for the class of `fluid_id`: `fluid_id` if the block at `pos`
    /// holds that fluid, otherwise something else (0).
    fn fluid_id_at(&self, fluid_id: i32, pos: WorldCoord) -> i32;
    /// `FluidClass.getFluidLevel` (0..=1000).
    fn fluid_level(&self, fluid_id: i32, pos: WorldCoord) -> i32;
    /// `FluidClass.setFluidLevel`; false if the block cannot take it.
    fn set_fluid_level(&mut self, fluid_id: i32, pos: WorldCoord, level: i32) -> bool;
    /// `FluidClass.getFluidQuanta`: buffer units one placed block costs; 0 for an unknown fluid.
    fn quanta(&self, fluid_id: i32) -> i32;
    /// The block at `pos` is air.
    fn is_air(&self, pos: WorldCoord) -> bool;
    /// Set the block at `pos` to air.
    fn remove_block(&mut self, pos: WorldCoord);
}

/// `FluidCoord`: a position and its step distance from the grate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidCoord {
    pub wc: WorldCoord,
    pub dist: i32,
}

impl FluidCoord {
    /// `compareTo`: lowest y first, then nearest (ties broken by position for determinism).
    fn natural_key(&self) -> (i32, i32, WorldCoord) {
        (self.wc.y, self.dist, self.wc)
    }
}

impl Ord for FluidCoord {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.natural_key().cmp(&o.natural_key())
    }
}
impl PartialOrd for FluidCoord {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

/// Which queue order the search work set uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkOrder {
    /// `SimpleComparator`: nearest first (suck mapping).
    ByDistance,
    /// `FluidCoord.compareTo`: lowest first (pouring, `checkVertical`).
    ByHeight,
}

/// `GratePathfinder`.
#[derive(Debug, Clone)]
pub struct GratePathfinder {
    pub start_pos: WorldCoord,
    pub fluid_id: i32,
    order: WorkOrder,
    backlink: HashMap<WorldCoord, WorldCoord>,
    work_dist: BinaryHeap<Reverse<(i32, WorldCoord)>>,
    work_height: BinaryHeap<Reverse<FluidCoord>>,
    /// `allset`: mapped source cells, highest (then farthest) first.
    all: BinaryHeap<FluidCoord>,
    /// Set when the original called `restartPath()`: the caller must drop this search.
    pub restart: bool,
}

impl GratePathfinder {
    pub fn new(check_vertical: bool) -> Self {
        Self {
            start_pos: WorldCoord::ZERO,
            fluid_id: 0,
            order: if check_vertical {
                WorkOrder::ByHeight
            } else {
                WorkOrder::ByDistance
            },
            backlink: HashMap::new(),
            work_dist: BinaryHeap::new(),
            work_height: BinaryHeap::new(),
            all: BinaryHeap::new(),
            restart: false,
        }
    }

    fn work_add(&mut self, wc: WorldCoord, dist: i32) {
        match self.order {
            WorkOrder::ByDistance => self.work_dist.push(Reverse((dist, wc))),
            WorkOrder::ByHeight => self.work_height.push(Reverse(FluidCoord { wc, dist })),
        }
    }

    fn work_poll(&mut self) -> Option<FluidCoord> {
        match self.order {
            WorkOrder::ByDistance => self
                .work_dist
                .pop()
                .map(|Reverse((dist, wc))| FluidCoord { wc, dist }),
            WorkOrder::ByHeight => self.work_height.pop().map(|Reverse(f)| f),
        }
    }

    /// Mapped source cells left to drain.
    pub fn mapped(&self) -> usize {
        self.all.len()
    }

    /// `start`: seeds the search from the sides in the `sides` mask with a known fluid.
    pub fn start(&mut self, wc: WorldCoord, fluid_id: i32, sides: u8) {
        self.fluid_id = fluid_id;
        self.start_pos = wc;
        for i in 0..6 {
            if sides & (1 << i) != 0 {
                let wc2 = wc.coord_step(i);
                self.backlink.insert(wc2, wc);
                self.work_add(wc2, 0);
            }
        }
    }

    /// `startSuck`: as `start`, but the fluid is whatever liquid sits on those sides.
    pub fn start_suck(&mut self, world: &dyn GrateWorld, wc: WorldCoord, sides: u8) -> bool {
        self.fluid_id = 0;
        self.start_pos = wc;
        for i in 0..6 {
            if sides & (1 << i) != 0 {
                let wc2 = wc.coord_step(i);
                self.backlink.insert(wc2, wc);
                self.work_add(wc2, 0);
                let fl = world.liquid_id(wc2);
                if fl != 0 {
                    self.fluid_id = fl;
                }
            }
        }
        self.fluid_id != 0
    }

    /// `isConnected`: the chain of back-links from `wc` reaches the grate through that fluid.
    pub fn is_connected(&self, world: &dyn GrateWorld, mut wc: WorldCoord) -> bool {
        if wc == self.start_pos {
            return true;
        }
        loop {
            match self.backlink.get(&wc) {
                None => return false,
                Some(&b) => wc = b,
            }
            if wc == self.start_pos {
                return true;
            }
            if world.fluid_id_at(self.fluid_id, wc) != self.fluid_id {
                return false;
            }
        }
    }

    fn step_add(&mut self, nc: FluidCoord) {
        for i in 0..6 {
            let wc2 = nc.wc.coord_step(i);
            if let std::collections::hash_map::Entry::Vacant(e) = self.backlink.entry(wc2) {
                e.insert(nc.wc);
                self.work_add(wc2, nc.dist + 1);
            }
        }
    }

    fn step_map(&mut self, world: &dyn GrateWorld, nc: FluidCoord) {
        for i in 0..6 {
            let wc2 = nc.wc.coord_step(i);
            if world.fluid_id_at(self.fluid_id, wc2) == self.fluid_id
                && !self.backlink.contains_key(&wc2)
            {
                self.backlink.insert(wc2, nc.wc);
                self.work_add(wc2, nc.dist + 1);
            }
        }
    }

    /// `tryDumpFluid`: returns the part of `level` still unplaced (0 = all placed).
    pub fn try_dump_fluid(&mut self, world: &mut dyn GrateWorld, mut level: i32, tries: usize) -> i32 {
        for _ in 0..tries {
            let Some(nc) = self.work_poll() else {
                self.restart = true;
                return level;
            };
            if !self.is_connected(world, nc.wc) {
                self.restart = true;
                return level;
            }
            if world.is_air(nc.wc) {
                if world.set_fluid_level(self.fluid_id, nc.wc, level) {
                    self.step_add(nc);
                    return 0;
                }
            } else if world.fluid_id_at(self.fluid_id, nc.wc) == self.fluid_id {
                self.step_add(nc);
                let lv1 = world.fluid_level(self.fluid_id, nc.wc);
                if lv1 < FULL_BLOCK {
                    let lv2 = (lv1 + level).min(FULL_BLOCK);
                    if world.set_fluid_level(self.fluid_id, nc.wc, lv2) {
                        level -= lv2 - lv1;
                        if level == 0 {
                            return 0;
                        }
                    }
                }
            }
        }
        level
    }

    /// `tryMapFluid`: true when mapping is finished (or capped) and sucking may begin.
    pub fn try_map_fluid(&mut self, world: &dyn GrateWorld, tries: usize) -> bool {
        if self.all.len() > MAP_LIMIT {
            return true;
        }
        for _ in 0..tries {
            let Some(nc) = self.work_poll() else {
                return true;
            };
            if world.fluid_id_at(self.fluid_id, nc.wc) == self.fluid_id {
                self.step_map(world, nc);
                if world.fluid_level(self.fluid_id, nc.wc) > 0 {
                    self.all.push(nc);
                }
            }
        }
        false
    }

    /// `trySuckFluid`: drains up to `level` from the highest mapped cells; returns what was taken.
    pub fn try_suck_fluid(&mut self, world: &mut dyn GrateWorld, level: i32) -> i32 {
        let mut tr = 0;
        while let Some(&nc) = self.all.peek() {
            if !self.is_connected(world, nc.wc) {
                self.restart = true;
                return tr;
            }
            if world.fluid_id_at(self.fluid_id, nc.wc) != self.fluid_id {
                self.all.pop();
                continue;
            }
            let lv1 = world.fluid_level(self.fluid_id, nc.wc);
            if lv1 == 0 {
                self.all.pop();
                continue;
            }
            if tr + lv1 <= level {
                tr += lv1;
                world.remove_block(nc.wc);
                self.all.pop();
                if tr == level {
                    return level;
                }
            }
            if world.set_fluid_level(self.fluid_id, nc.wc, level - tr) {
                return level;
            }
        }
        self.restart = true;
        tr
    }
}

/// Search state (`searchState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchState {
    Idle,
    Mapping,
    Pouring,
}

/// `TileGrate`.
#[derive(Debug, Clone)]
pub struct TileGrate {
    /// Facing (`Rotation`, 0..=5); the grate works on the side it faces.
    pub rotation: i32,
    pub buffer: FluidBuffer,
    pub pressure: i32,
    state: SearchState,
    search: Option<GratePathfinder>,
}

/// Persisted form (`writeToNBT`: `buf`, `pres`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrateTag {
    pub buf: FluidBufferTag,
    pub pres: i16,
}

impl TileGrate {
    pub fn new(rotation: i32) -> Self {
        Self {
            rotation,
            buffer: FluidBuffer::default(),
            pressure: 0,
            state: SearchState::Idle,
            search: None,
        }
    }

    /// `getPipeConnectableSides` / `getPipeFlangeSides`.
    pub fn pipe_sides(&self) -> u8 {
        1 << self.rotation
    }

    /// `getPipePressure`.
    pub fn pipe_pressure(&self) -> i32 {
        self.pressure
    }

    /// `restartPath`.
    pub fn restart_path(&mut self) {
        self.search = None;
        self.state = SearchState::Idle;
    }

    pub fn write_tag(&self) -> GrateTag {
        GrateTag {
            buf: self.buffer.write_tag(),
            pres: self.pressure as i16,
        }
    }

    pub fn read_tag(&mut self, t: &GrateTag) {
        self.buffer.read_tag(&t.buf);
        self.pressure = t.pres as i32;
    }

    /// The per-tick part of `g()`: takes the pressure of what is in front (`PipeLib.getPressure`
    /// of the neighbour, `None` if it is not a pipe/tank), and continues mapping.
    /// Moving liquid to that neighbour is [`TileGrate::flow_to`].
    pub fn tick(&mut self, front_pressure: Option<i32>, world: &dyn GrateWorld) {
        if let Some(pr) = front_pressure {
            self.pressure = pr - pr.signum();
        }
        if self.state == SearchState::Mapping {
            if let Some(s) = self.search.as_mut() {
                s.try_map_fluid(world, MAP_TRIES);
            }
        }
    }

    /// `PipeLib.movePipeLiquid` towards the pipe in front: this grate's buffer with the
    /// neighbour's pressure and buffer.
    pub fn flow_to(&mut self, other_pressure: i32, other: &mut FluidBuffer, world_tick: i64) {
        let p1 = self.pressure;
        if p1 < other_pressure {
            return;
        }
        let f1 = &mut self.buffer;
        let l1 = f1.get_level(world_tick) + f1.delta;
        if f1.fluid_type == 0 || l1 <= 0 {
            return;
        }
        let l2 = other.get_level(world_tick);
        if other.fluid_type != 0 && other.fluid_type != f1.fluid_type {
            return;
        }
        let qty = (if p1 > other_pressure { PRESSURE_FLOW } else { 0 })
            .max((l1 - l2) / 2)
            .min(other.max_level() - l2)
            .min(l1);
        if qty <= 0 {
            return;
        }
        let t = f1.fluid_type;
        f1.add_level(t, -qty);
        other.add_level(t, qty);
    }

    /// `onTileTick` for a grate at `pos` (every [`GRATE_TICK_DELAY`] ticks): start, advance or finish a search.
    pub fn on_tile_tick_at(&mut self, pos: WorldCoord, world: &mut dyn GrateWorld, world_tick: i64) {
        let sides: u8 = 63 ^ (1 << self.rotation);
        if self.pressure == 0 {
            self.restart_path();
        } else if self.pressure < -GRATE_THRESHOLD {
            if self.buffer.get_level(world_tick) >= self.buffer.max_level() {
                return;
            }
            if self.state == SearchState::Pouring {
                self.restart_path();
            }
            if self.state == SearchState::Idle {
                self.state = SearchState::Mapping;
                let mut s = GratePathfinder::new(false);
                if self.buffer.fluid_type == 0 {
                    if !s.start_suck(world, pos, sides) {
                        self.restart_path();
                        return;
                    }
                } else {
                    s.start(pos, self.buffer.fluid_type, sides);
                }
                self.search = Some(s);
            }
            if self.state == SearchState::Mapping {
                let Some(s) = self.search.as_mut() else { return };
                if !s.try_map_fluid(world, MAP_TRIES) {
                    return;
                }
                let fluid_id = s.fluid_id;
                let quanta = world.quanta(fluid_id);
                let lv = s.try_suck_fluid(world, quanta);
                if s.restart {
                    self.restart_path();
                }
                if lv == 0 {
                    return;
                }
                self.buffer.add_level(fluid_id, lv);
            }
        } else if self.pressure > GRATE_THRESHOLD {
            let fid = self.buffer.fluid_type;
            if fid == 0 {
                return;
            }
            let fq = world.quanta(fid);
            if fq == 0 || self.buffer.get_level(world_tick) < fq {
                return;
            }
            if self.state == SearchState::Mapping {
                self.restart_path();
            }
            if self.state == SearchState::Idle {
                self.state = SearchState::Pouring;
                let mut s = GratePathfinder::new(true);
                s.start(pos, fid, sides);
                self.search = Some(s);
            }
            if self.state == SearchState::Pouring {
                let Some(s) = self.search.as_mut() else { return };
                let fr = s.try_dump_fluid(world, fq, DUMP_TRIES);
                if s.restart {
                    self.restart_path();
                }
                if fr != fq {
                    self.buffer.add_level(fid, -fq);
                }
            }
        }
    }
}

const _: () = assert!(PIPE_MAX_LEVEL == 1000);

#[cfg(test)]
mod tests {
    use super::*;

    const WATER: i32 = 9;

    #[derive(Default)]
    struct W {
        liquid: HashMap<WorldCoord, i32>,
        solid: std::collections::HashSet<WorldCoord>,
    }
    impl GrateWorld for W {
        fn liquid_id(&self, p: WorldCoord) -> i32 {
            if self.liquid.contains_key(&p) { WATER } else { 0 }
        }
        fn fluid_id_at(&self, _f: i32, p: WorldCoord) -> i32 {
            self.liquid_id(p)
        }
        fn fluid_level(&self, _f: i32, p: WorldCoord) -> i32 {
            *self.liquid.get(&p).unwrap_or(&0)
        }
        fn set_fluid_level(&mut self, _f: i32, p: WorldCoord, l: i32) -> bool {
            if self.solid.contains(&p) {
                return false;
            }
            if l <= 0 { self.liquid.remove(&p); } else { self.liquid.insert(p, l); }
            true
        }
        fn quanta(&self, _f: i32) -> i32 { 1000 }
        fn is_air(&self, p: WorldCoord) -> bool {
            !self.liquid.contains_key(&p) && !self.solid.contains(&p)
        }
        fn remove_block(&mut self, p: WorldCoord) { self.liquid.remove(&p); }
    }

    fn pos() -> WorldCoord { WorldCoord::new(0, 10, 0) }

    #[test]
    fn suck_drains_source_in_front() {
        let mut w = W::default();
        // rotation 0 (down): works on every side except below... faces `rotation`, searches the rest
        let mut g = TileGrate::new(0);
        w.liquid.insert(pos().coord_step(1), 1000);
        g.pressure = -200;
        g.on_tile_tick_at(pos(), &mut w, 1);
        g.on_tile_tick_at(pos(), &mut w, 2);
        assert_eq!(g.buffer.fluid_type, WATER);
        assert_eq!(g.buffer.delta + g.buffer.level, 1000);
        assert!(w.liquid.is_empty());
    }

    #[test]
    fn pour_places_liquid_and_drains_buffer() {
        let mut w = W::default();
        let mut g = TileGrate::new(0);
        g.buffer.add_level(WATER, 1000);
        g.buffer.get_level(5);
        g.pressure = 200;
        g.on_tile_tick_at(pos(), &mut w, 6);
        assert_eq!(w.liquid.values().sum::<i32>(), 1000);
        assert_eq!(g.buffer.delta, -1000);
    }

    #[test]
    fn idle_pressure_restarts() {
        let mut w = W::default();
        let mut g = TileGrate::new(0);
        g.pressure = 0;
        g.on_tile_tick_at(pos(), &mut w, 1);
        assert!(g.search.is_none());
    }

    #[test]
    fn tag_roundtrip() {
        let mut g = TileGrate::new(2);
        g.pressure = -150;
        g.buffer.add_level(WATER, 5);
        let t = g.write_tag();
        let mut h = TileGrate::new(2);
        h.read_tag(&t);
        assert_eq!(h.pressure, -150);
        assert_eq!(h.buffer.delta, 5);
    }
}
