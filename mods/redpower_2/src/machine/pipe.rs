//! Fluid pipe (`redpower_2:pipe`): port of `TilePipe`, `FluidBuffer` and `PipeLib.movePipeLiquid`.
//!
//! Pure logic over a map of pipe tiles; the engine layer feeds neighbours and the world tick.

use crate::WorldCoord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Packet sub-id for the buffer-only update (`TilePipe.sendItemUpdate`).
pub const SUBID_PIPE_BUFFER: u8 = 11;
/// Per-step transfer floor when pressures differ.
pub const PRESSURE_FLOW: i32 = 25;
/// Maximum level of a pipe (and pump) buffer: `FluidBuffer.getMaxLevel`.
pub const PIPE_MAX_LEVEL: i32 = 1000;
/// Pressure a tank container reports, as `PipeLib` normalises it.
pub const TANK_PRESSURE: i32 = 100;

/// `FluidBuffer`: a typed level with changes deferred to the next tick (`Delta`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FluidBuffer {
    pub fluid_type: i32,
    pub level: i32,
    pub delta: i32,
    last_tick: i32,
}

impl FluidBuffer {
    pub fn set_type(&mut self, t: i32) {
        self.fluid_type = t;
    }

    pub fn max_level(&self) -> i32 {
        PIPE_MAX_LEVEL
    }

    /// `getLevel`: folds `delta` into `level` once per world tick (low 16 bits).
    pub fn get_level(&mut self, world_tick: i64) -> i32 {
        let lt = (world_tick & 0xFFFF) as i32;
        if lt == self.last_tick {
            return self.level;
        }
        self.last_tick = lt;
        self.level += self.delta;
        self.delta = 0;
        if self.level == 0 {
            self.fluid_type = 0;
        }
        self.level
    }

    pub fn add_level(&mut self, fluid_type: i32, lvl: i32) {
        self.fluid_type = fluid_type;
        self.delta += lvl;
    }
}

/// `TilePipe` state. `cover_sides`/`covers` mirror `TileCovered` (cover class is `covers[i] >> 8`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pipe {
    pub pressure: i32,
    pub buffer: FluidBuffer,
    pub cover_sides: u8,
    pub covers: [u16; 6],
    /// Cached connections (`ConCache`), `None` when stale.
    pub con_cache: Option<u8>,
}

impl Pipe {
    /// `getPipeConnectableSides`: all six, minus sides with a cover of class < 3.
    pub fn connectable_sides(&self) -> u8 {
        let mut tr = 63u8;
        for i in 0..6 {
            if self.cover_sides & (1 << i) != 0 && (self.covers[i] >> 8) < 3 {
                tr &= !(1 << i);
            }
        }
        tr
    }

    /// `getPipeFlangeSides`: straight runs and dead ends have no flanges.
    pub fn flange_sides(&self) -> u8 {
        let c = self.con_cache.unwrap_or(0);
        if c == 3 || c == 12 || c == 48 || c.count_ones() == 1 {
            0
        } else {
            c
        }
    }

    pub fn uncache(&mut self) {
        self.con_cache = None;
    }
}

/// Pure pressure rule of `TilePipe.g()` from the neighbours' pressures (None = not a pipe/tank).
pub fn average_pressure(neighbours: &[Option<i32>]) -> i32 {
    let (mut pr, mut d, mut min, mut max) = (0i32, 0, 0i32, 0i32);
    for p in neighbours.iter().flatten() {
        min = min.min(*p);
        max = max.max(*p);
        pr += p;
        d += 1;
    }
    if d == 0 {
        return 0;
    }
    if min < 0 {
        min += 1;
    }
    if max > 0 {
        max -= 1;
    }
    min.max(max.min(pr / d + pr.signum()))
}

/// Persisted form of a buffer (`FluidBuffer.writeToNBT`). The original writes the
/// tick under `lck` but reads `ltk`, so it never survives a reload; we keep that.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FluidBufferTag {
    #[serde(rename = "type")]
    pub fluid_type: i16,
    pub lvl: i16,
    pub del: i16,
    pub lck: i16,
}

impl FluidBuffer {
    pub fn write_tag(&self) -> FluidBufferTag {
        FluidBufferTag {
            fluid_type: self.fluid_type as i16,
            lvl: self.level as i16,
            del: self.delta as i16,
            lck: self.last_tick as i16,
        }
    }

    pub fn read_tag(&mut self, t: &FluidBufferTag) {
        self.fluid_type = t.fluid_type as i32;
        self.level = t.lvl as i32;
        self.delta = t.del as i32;
        self.last_tick = 0;
    }

    /// `writeToPacket`: (type, level) as two UVLC values.
    pub fn write_packet(&self) -> (u32, u32) {
        (self.fluid_type as u32, self.level as u32)
    }

    pub fn read_packet(&mut self, fluid_type: u32, level: u32) {
        self.fluid_type = fluid_type as i32;
        self.level = level as i32;
    }
}

/// `ITankContainer` side as `PipeLib` sees it (a BuildCraft-style tank).
pub trait PipeTank {
    /// `getTankPressure` (sign only matters).
    fn tank_pressure(&self) -> i32;
    /// Contents as (fluid id, amount); fluid id 0 = a liquid with no registered class.
    fn liquid(&self) -> Option<(i32, i32)>;
    fn capacity(&self) -> i32;
    /// Removes up to `qty`, returns what was drained.
    fn drain(&mut self, qty: i32) -> i32;
    /// Adds `qty` of `fluid_id`, returns what was accepted.
    fn fill(&mut self, fluid_id: i32, qty: i32) -> i32;
}

/// `PipeLib.getPressure` for tanks: the sign scaled to +-100.
pub fn tank_pressure_norm(p: i32) -> i32 {
    p.signum() * TANK_PRESSURE
}

/// The tank half of `PipeLib.movePipeLiquid`: moves between a pipe-like buffer and a tank.
pub fn move_liquid_tank(
    src_pressure: i32,
    f1: &mut FluidBuffer,
    tank: &mut dyn PipeTank,
    world_tick: i64,
) {
    let p2 = tank_pressure_norm(tank.tank_pressure());
    let p1 = src_pressure;
    let l1 = f1.get_level(world_tick) + f1.delta;
    let mut l2 = 0;
    let mut fid = 0;
    if let Some((id, amt)) = tank.liquid() {
        if amt > 0 {
            if id == 0 {
                return;
            }
            l2 = amt;
            fid = id;
            if f1.fluid_type != 0 && f1.fluid_type != id {
                return;
            }
        }
    }
    if p1 < p2 && l2 > 0 {
        let qty = PRESSURE_FLOW
            .max((l2 - l1) / 2)
            .min(f1.max_level() - l1)
            .min(l2);
        if qty > 0 {
            let got = tank.drain(qty);
            f1.add_level(fid, got);
        }
    } else if p1 > p2 && f1.fluid_type != 0 && l1 > 0 {
        let qty = PRESSURE_FLOW
            .max((l1 - l2) / 2)
            .min(tank.capacity() - l2)
            .min(l1);
        if qty > 0 {
            let t = f1.fluid_type;
            let put = tank.fill(t, qty);
            f1.add_level(t, -put);
        }
    }
}

/// A set of pipes by position.
#[derive(Debug, Default, Clone)]
pub struct PipeWorld {
    pub pipes: HashMap<WorldCoord, Pipe>,
}

impl PipeWorld {
    /// `PipeLib.getConnections` for the pipe at `pos`: connectable sides facing a connectable neighbour.
    pub fn connections(&self, pos: WorldCoord) -> u8 {
        let Some(p) = self.pipes.get(&pos) else {
            return 0;
        };
        let sides = p.connectable_sides();
        let mut trs = 0u8;
        for i in 0..6 {
            if sides & (1 << i) != 0 {
                if let Some(n) = self.pipes.get(&pos.coord_step(i)) {
                    if n.connectable_sides() & (1 << (i ^ 1)) != 0 {
                        trs |= 1 << i;
                    }
                }
            }
        }
        trs
    }

    /// One `TilePipe.g()` server tick: pressure update, then liquid movement.
    pub fn tick_pipe(&mut self, pos: WorldCoord, world_tick: i64) {
        if !self.pipes.contains_key(&pos) {
            return;
        }
        let con = self.connections(pos);
        let mut ps = [None; 6];
        for (i, slot) in ps.iter_mut().enumerate() {
            if con & (1 << i) != 0 {
                *slot = self
                    .pipes
                    .get(&pos.coord_step(i as i32))
                    .map(|n| n.pressure);
            }
        }
        let np = average_pressure(&ps);
        if let Some(p) = self.pipes.get_mut(&pos) {
            p.pressure = np;
            p.con_cache = Some(con);
        }
        self.move_liquid(pos, con, world_tick);
    }

    /// `PipeLib.movePipeLiquid` between pipes.
    pub fn move_liquid(&mut self, pos: WorldCoord, sides: u8, world_tick: i64) {
        for i in 0..6i32 {
            if sides & (1 << i) == 0 {
                continue;
            }
            let npos = pos.coord_step(i);
            let (Some(src), Some(dst)) = (self.pipes.get(&pos), self.pipes.get(&npos)) else {
                continue;
            };
            let (p1, p2) = (src.pressure, dst.pressure);
            if p1 < p2 {
                continue;
            }
            let mut f1 = src.buffer;
            let mut f2 = dst.buffer;
            let mut l1 = f1.get_level(world_tick);
            l1 += f1.delta;
            if f1.fluid_type == 0 || l1 <= 0 {
                continue;
            }
            let l2 = f2.get_level(world_tick);
            if f2.fluid_type != 0 && f2.fluid_type != f1.fluid_type {
                continue;
            }
            let mut qty = (if p1 > p2 { PRESSURE_FLOW } else { 0 }).max((l1 - l2) / 2);
            qty = qty.min(f2.max_level() - l2).min(l1);
            if qty <= 0 {
                continue;
            }
            let t = f1.fluid_type;
            f1.add_level(t, -qty);
            f2.add_level(t, qty);
            self.pipes.get_mut(&pos).unwrap().buffer = f1;
            self.pipes.get_mut(&npos).unwrap().buffer = f2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wc(x: i32) -> WorldCoord {
        WorldCoord::new(x, 0, 0)
    }

    #[test]
    fn buffer_defers_delta_to_next_tick() {
        let mut b = FluidBuffer::default();
        b.add_level(1, 10);
        assert_eq!(b.get_level(0), 0); // same tick as last_tick=0: unchanged
        assert_eq!(b.get_level(1), 10);
        b.add_level(1, -10);
        assert_eq!(b.get_level(2), 0);
        assert_eq!(b.fluid_type, 0);
    }

    #[test]
    fn covers_block_connections() {
        let p = Pipe {
            cover_sides: 1,
            covers: [2 << 8, 0, 0, 0, 0, 0],
            ..Default::default()
        };
        assert_eq!(p.connectable_sides(), 62);
        let p = Pipe {
            covers: [3 << 8, 0, 0, 0, 0, 0],
            ..Default::default()
        };
        assert_eq!(p.connectable_sides(), 63);
    }

    #[test]
    fn flanges_only_at_junctions() {
        let mut p = Pipe::default();
        for (c, f) in [(3, 0), (12, 0), (48, 0), (1, 0), (5, 5), (7, 7)] {
            p.con_cache = Some(c);
            assert_eq!(p.flange_sides(), f);
        }
    }

    #[test]
    fn pressure_rules() {
        assert_eq!(average_pressure(&[None, None]), 0);
        assert_eq!(average_pressure(&[Some(1000)]), 999);
        assert_eq!(average_pressure(&[Some(-1000)]), -999);
        assert_eq!(average_pressure(&[Some(0), Some(0)]), 0);
    }

    #[test]
    fn liquid_flows_downhill_and_levels() {
        let mut w = PipeWorld::default();
        // sides 4/5 are z in the original's numbering; x axis uses 16/32 handled by coord_step
        let (a, b) = (wc(0), wc(1));
        let mut pa = Pipe::default();
        pa.buffer.fluid_type = 1;
        pa.buffer.level = 16;
        w.pipes.insert(a, pa);
        w.pipes.insert(b, Pipe::default());
        for t in 1..20 {
            w.tick_pipe(a, t);
            w.tick_pipe(b, t);
        }
        let la = w.pipes[&a].buffer.level + w.pipes[&a].buffer.delta;
        let lb = w.pipes[&b].buffer.level + w.pipes[&b].buffer.delta;
        assert_eq!(la + lb, 16);
        assert!((la - lb).abs() <= 1);
    }

    #[test]
    fn mismatched_fluids_do_not_mix() {
        let mut w = PipeWorld::default();
        let mut pa = Pipe::default();
        pa.buffer.fluid_type = 1;
        pa.buffer.level = 10;
        let mut pb = Pipe::default();
        pb.buffer.fluid_type = 2;
        pb.buffer.level = 2;
        w.pipes.insert(wc(0), pa);
        w.pipes.insert(wc(1), pb);
        w.tick_pipe(wc(0), 1);
        assert_eq!(w.pipes[&wc(1)].buffer.level, 2);
    }

    #[test]
    fn pressure_pushes_fluid_up() {
        let mut w = PipeWorld::default();
        let pa = Pipe {
            pressure: 1000,
            buffer: FluidBuffer {
                fluid_type: 1,
                level: 4,
                ..Default::default()
            },
            ..Default::default()
        };
        w.pipes.insert(wc(0), pa);
        w.pipes.insert(wc(1), Pipe::default());
        w.move_liquid(wc(0), w.connections(wc(0)), 1);
        assert_eq!(w.pipes[&wc(1)].buffer.delta, 4);
    }

    struct T {
        p: i32,
        fluid: i32,
        amt: i32,
        cap: i32,
    }
    impl PipeTank for T {
        fn tank_pressure(&self) -> i32 {
            self.p
        }
        fn liquid(&self) -> Option<(i32, i32)> {
            Some((self.fluid, self.amt))
        }
        fn capacity(&self) -> i32 {
            self.cap
        }
        fn drain(&mut self, q: i32) -> i32 {
            let d = q.min(self.amt);
            self.amt -= d;
            d
        }
        fn fill(&mut self, f: i32, q: i32) -> i32 {
            let a = q.min(self.cap - self.amt);
            self.amt += a;
            self.fluid = f;
            a
        }
    }

    #[test]
    fn tank_fills_and_drains_pipe() {
        let mut t = T {
            p: 0,
            fluid: 1,
            amt: 100,
            cap: 1000,
        };
        let mut b = FluidBuffer::default();
        move_liquid_tank(-1, &mut b, &mut t, 1);
        assert_eq!((b.delta, t.amt), (50, 50));
        let mut b = FluidBuffer {
            fluid_type: 1,
            level: 40,
            ..Default::default()
        };
        let mut t = T {
            p: 0,
            fluid: 1,
            amt: 0,
            cap: 1000,
        };
        move_liquid_tank(5, &mut b, &mut t, 1);
        assert_eq!((b.delta, t.amt), (-25, 25));
        assert_eq!(tank_pressure_norm(-7), -100);
    }

    #[test]
    fn buffer_tag_roundtrip() {
        let b = FluidBuffer {
            fluid_type: 3,
            level: 9,
            delta: -2,
            ..Default::default()
        };
        let mut c = FluidBuffer::default();
        c.read_tag(&b.write_tag());
        assert_eq!((c.fluid_type, c.level, c.delta), (3, 9, -2));
        assert_eq!(c.max_level(), 1000);
    }
}
