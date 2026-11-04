//! Blutricity dynamic RLC circuit simulation: `BluePowerConductor`, `BluePowerEndpoint` and the
//! grid that steps them (`spec 2.5 Blutricity Power Grid`).
//!
//! The model is the original's, field for field:
//!
//! - A node stores its charge as a capacitor voltage `Vcap` plus an accumulated current `Icap`.
//!   The two are folded together once per game tick, lazily, the first time the voltage is read in
//!   that tick (`computeVoltage`): `Itot = 0.5 * It1`, `Vcap += 0.05 * Icap * invCap`.
//! - Every connection slot keeps its own inductor current: three straight slots (one per axis) and
//!   twelve cube-edge slots. A slot is a bit of [`BlueConductor::imask`], its current an entry of
//!   [`BlueConductor::currents`] in bit order.
//! - `iterate` walks every set slot and moves current to the neighbour:
//!   `R = R1 + R2`, `V = Vcap - Vn`, `I_next = I + (V - I * R) * 0.07`,
//!   `I_flow = I + V * 0.5` — the parallel term uses the *pre-update* current, exactly as the
//!   Java does — then `-I_flow` is applied here and `+I_flow` to the neighbour.
//! - `BluePowerEndpoint` is a conductor that also keeps a display charge (`Charge = Vcap * 10`)
//!   and a 32-tick history of "charged" bits (`Flow`), updated after its own iteration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Edge-slot index of each of the twelve cube edges (`BluePowerConductor.dirmap`). The value
/// packs `(direction << 2) | step`, i.e. which neighbour and which edge of it.
pub const DIRMAP: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 11, 14, 18, 23];

/// `getConDirMask(dir)`: the 30-bit connectable-mask bits belonging to one face. Bits 0..23 are
/// the cube's edges (four per face), bits 24..29 the six faces themselves.
pub const CON_DIR_MASK: [u32; 6] = [
    17_895_680,  // dir 0 (-Y, down)
    35_791_360,  // dir 1 (+Y, up)
    71_565_329,  // dir 2 (-Z, north)
    143_130_658, // dir 3 (+Z, south)
    268_452_932, // dir 4 (-X, west)
    536_905_864, // dir 5 (+X, east)
];

/// Conductor internal resistance (`getResistance`), ohms.
pub const RESISTANCE: f64 = 0.01;
/// Inductive scale of the current update (`getIndScale`).
pub const IND_SCALE: f64 = 0.07;
/// Parallel conductance term (`getCondParallel`).
pub const COND_PARALLEL: f64 = 0.5;
/// Inverse capacitance of cables and endpoints: `C = 4 F`.
pub const INV_CAP_ENDPOINT: f64 = 0.25;
/// Inverse capacitance of panels: `C = 0.25 F`.
pub const INV_CAP_PANEL: f64 = 4.0;

/// One Blutricity conductor (`BluePowerConductor`), optionally an endpoint (`BluePowerEndpoint`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlueConductor {
    /// Connection bits: bit 0..2 straight slots per axis, bits 3..14 the twelve cube edges.
    pub imask: u32,
    /// One inductor current per set bit of `imask`, in bit order.
    pub currents: Vec<f64>,
    /// Capacitor voltage, volts.
    pub vcap: f64,
    /// Current accumulated for the next capacitor update.
    pub icap: f64,
    /// Vestigial effective voltage kept for NBT parity (`Veff`).
    pub veff: f64,
    /// Game tick (mod 65536) of the last capacitor update.
    pub last_tick: u32,
    /// Sum of `|I|` applied this tick, doubled into `itot`.
    pub it1: f64,
    /// Total current of the previous tick (`Itot`).
    pub itot: f64,
    /// `1 / C`, standing in for the abstract `getInvCap()`.
    pub inv_cap: f64,
    /// This node is a `BluePowerEndpoint` (keeps `charge`/`flow`).
    pub endpoint: bool,
    /// Endpoint display charge (`Charge`, volts x 10).
    pub charge: i32,
    /// Endpoint 32-tick "at least 60 V" history (`Flow`).
    pub flow: i32,
}

pub type BluePowerConductor = BlueConductor;

impl BlueConductor {
    /// A plain conductor with the given inverse capacitance.
    pub fn new(inv_cap: f64) -> Self {
        Self {
            imask: 0,
            currents: Vec::new(),
            vcap: 0.0,
            icap: 0.0,
            veff: 0.0,
            last_tick: 0,
            it1: 0.0,
            itot: 0.0,
            inv_cap,
            endpoint: false,
            charge: 0,
            flow: 0,
        }
    }

    /// A `BluePowerEndpoint`: a cable endpoint, `C = 4 F`.
    pub fn endpoint() -> Self {
        Self {
            endpoint: true,
            ..Self::new(INV_CAP_ENDPOINT)
        }
    }

    /// A panel's conductor, `C = 0.25 F`.
    pub fn panel() -> Self {
        Self::new(INV_CAP_PANEL)
    }

    /// Rebuild the connection mask from a 30-bit connectable mask (`conm`) and a 30-bit edge mask
    /// (`econm`, as a wind turbine's sails give). Currents of slots that survive are kept in place,
    /// new slots start at zero; the mask is left untouched when nothing changed.
    pub fn recache(&mut self, conm: u32, econm: u32) {
        let mut imo = 0u32;
        for a in 0..3 {
            if conm & CON_DIR_MASK[a * 2] != 0 {
                imo |= 1 << a;
            }
        }
        for (a, d) in DIRMAP.iter().enumerate() {
            if econm & (1 << d) != 0 {
                imo |= 8 << a;
            }
        }
        if self.imask == imo {
            return;
        }
        let mut c2 = Vec::with_capacity(imo.count_ones() as usize);
        let mut s = 0usize;
        for a in 0..15 {
            let m = 1u32 << a;
            let mut v = 0.0;
            if self.imask & m != 0 {
                v = self.currents[s];
                s += 1;
            }
            if imo & m != 0 {
                c2.push(v);
            }
        }
        self.currents = c2;
        self.imask = imo;
    }

    /// Set one connection bit, inserting a zero current at its place.
    pub fn add_slot(&mut self, slot: u32) {
        let m = 1u32 << slot;
        if self.imask & m != 0 {
            return;
        }
        let idx = (self.imask & (m - 1)).count_ones() as usize;
        self.currents.insert(idx, 0.0);
        self.imask |= m;
    }

    /// Clear one connection bit, dropping its current.
    pub fn remove_slot(&mut self, slot: u32) {
        let m = 1u32 << slot;
        if self.imask & m == 0 {
            return;
        }
        let idx = (self.imask & (m - 1)).count_ones() as usize;
        self.currents.remove(idx);
        self.imask &= !m;
    }

    /// Fold this tick's accumulated current into the capacitor (`computeVoltage`).
    fn compute_voltage(&mut self) {
        self.itot = 0.5 * self.it1;
        self.it1 = 0.0;
        self.vcap += 0.05 * self.icap * self.inv_cap;
        self.icap = 0.0;
    }

    /// Voltage at game tick `tick` (`getVoltage`). The capacitor is settled once per tick; the
    /// tick is compared modulo 65536 as the original does with world time.
    pub fn voltage(&mut self, tick: u64) -> f64 {
        let lt = (tick & 0xFFFF) as u32;
        if lt != self.last_tick {
            self.last_tick = lt;
            self.compute_voltage();
        }
        self.vcap
    }

    /// Add current `i_in` for this tick (`applyCurrent`): settles the capacitor first, then adds to
    /// `Icap` and to the absolute-current sum.
    pub fn apply_current(&mut self, tick: u64, i_in: f64) {
        self.voltage(tick);
        self.icap += i_in;
        self.it1 += i_in.abs();
    }

    /// Alias of [`BlueConductor::apply_current`] (`applyDirect`).
    pub fn apply_direct(&mut self, tick: u64, i_in: f64) {
        self.apply_current(tick, i_in);
    }

    /// Draw power `P` (W) from the node, never pushing the voltage below zero (`drawPower`).
    pub fn draw_power(&mut self, tick: u64, p: f64) {
        let p1 = self.vcap * self.vcap - 0.1 * p * self.inv_cap;
        let t = if p1 < 0.0 { 0.0 } else { p1.sqrt() - self.vcap };
        self.apply_direct(tick, 20.0 * t / self.inv_cap);
    }

    /// Energy (W-t) held above `vthresh` (`getEnergy`).
    pub fn energy(&mut self, tick: u64, vthresh: f64) -> f64 {
        let d = self.voltage(tick);
        let tr = 0.5 * (d * d - vthresh * vthresh) / self.inv_cap;
        if tr < 0.0 {
            0.0
        } else {
            tr
        }
    }

    /// Feed power `P` (W) into the node (`applyPower`).
    pub fn apply_power(&mut self, tick: u64, p: f64) {
        let t = (self.vcap * self.vcap + 0.1 * p * self.inv_cap).sqrt() - self.vcap;
        self.apply_direct(tick, 20.0 * t / self.inv_cap);
    }

    /// Endpoint post-iteration step (`BluePowerEndpoint.iterate`): latch the display charge and
    /// shift the "charged" bit into the flow history.
    pub fn endpoint_update(&mut self, tick: u64) {
        self.charge = (self.voltage(tick) * 10.0) as i32;
        self.flow = (self.flow << 1) | i32::from(self.charge >= 600);
    }

    /// Charge gauge fill out of `i` (`getChargeScaled`): endpoints only, `min(i, i * Charge / 1000)`.
    pub fn charge_scaled(&self, i: i32) -> i32 {
        if self.endpoint {
            i.min(i * self.charge / 1000)
        } else {
            0
        }
    }

    /// Flow gauge fill out of `i` (`getFlowScaled`): endpoints only, `bitCount(Flow) * i / 32`.
    pub fn flow_scaled(&self, i: i32) -> i32 {
        if self.endpoint {
            self.flow.count_ones() as i32 * i / 32
        } else {
            0
        }
    }

    /// Chunk save (`writeToNBT`), as key/value strings.
    pub fn save(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("bpim".into(), self.imask.to_string());
        let list: Vec<String> = self.currents.iter().map(|c| c.to_string()).collect();
        nbt.insert("bpil".into(), list.join(","));
        nbt.insert("vcap".into(), self.vcap.to_string());
        nbt.insert("icap".into(), self.icap.to_string());
        nbt.insert("veff".into(), self.veff.to_string());
        nbt.insert("it1".into(), self.it1.to_string());
        nbt.insert("itot".into(), self.itot.to_string());
        nbt.insert("ltk".into(), self.last_tick.to_string());
        if self.endpoint {
            nbt.insert("chg".into(), self.charge.to_string());
            nbt.insert("flw".into(), self.flow.to_string());
        }
    }

    /// Chunk load (`readFromNBT`). As in the original, the currents and the scalar state are only
    /// restored when the saved current list length matches the saved mask's bit count.
    pub fn load(&mut self, nbt: &HashMap<String, String>) {
        let f = |k: &str| {
            nbt.get(k)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0)
        };
        self.imask = nbt
            .get("bpim")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let l = self.imask.count_ones() as usize;
        self.currents = vec![0.0; l];
        let list: Vec<f64> = nbt
            .get("bpil")
            .map(|s| {
                s.split(',')
                    .filter(|p| !p.is_empty())
                    .filter_map(|p| p.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        if list.len() == l {
            self.currents = list;
            self.vcap = f("vcap");
            self.icap = f("icap");
            self.veff = f("veff");
            self.it1 = f("it1");
            self.itot = f("itot");
            self.last_tick = nbt
                .get("ltk")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);
        }
        if let Some(c) = nbt.get("chg").and_then(|v| v.parse::<i32>().ok()) {
            self.charge = c;
        }
        if let Some(fl) = nbt.get("flw").and_then(|v| v.parse::<i32>().ok()) {
            self.flow = fl;
        }
    }
}

/// A set of conductors and the links between them: `link(a, slot_a, b, slot_b)` says conductor
/// `a`'s connection bit `slot_a` leads to `b` and back.
#[derive(Debug, Default)]
pub struct BlueGrid {
    /// Conductors by tile id.
    pub nodes: HashMap<u64, BlueConductor>,
    /// `(node, connection bit)` to neighbour.
    links: HashMap<(u64, u32), u64>,
}

impl BlueGrid {
    /// Add or replace a conductor.
    pub fn add(&mut self, id: u64, c: BlueConductor) {
        self.nodes.insert(id, c);
    }

    /// Link both ways: `a` through bit `slot_a`, `b` through bit `slot_b`. Both nodes gain the
    /// bit (a new slot starts at zero current).
    pub fn link(&mut self, a: u64, slot_a: u32, b: u64, slot_b: u32) {
        self.links.insert((a, slot_a), b);
        self.links.insert((b, slot_b), a);
        for (id, slot) in [(a, slot_a), (b, slot_b)] {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.add_slot(slot);
            }
        }
    }

    /// Drop every link touching `id` and remove the node.
    pub fn remove(&mut self, id: u64) {
        self.links.retain(|(n, _), other| *n != id && *other != id);
        self.nodes.remove(&id);
    }

    /// Iterate one conductor at `tick` (`BluePowerConductor.iterate`, plus the endpoint step).
    /// Current moves along every set connection bit, in bit order; a slot whose neighbour is gone
    /// keeps its current and is skipped.
    pub fn iterate(&mut self, id: u64, tick: u64) {
        let Some(mut me) = self.nodes.remove(&id) else {
            return;
        };
        me.voltage(tick);
        let mut s = 0usize;
        let mut dm = me.imask;
        while dm > 0 {
            let d = dm.trailing_zeros();
            dm &= !(1u32 << d);
            let Some(other_id) = self.links.get(&(id, d)).copied() else {
                s += 1;
                continue;
            };
            let Some(other) = self.nodes.get_mut(&other_id) else {
                s += 1;
                continue;
            };
            let i = me.currents.get(s).copied().unwrap_or(0.0);
            let v = me.vcap - other.voltage(tick);
            let r = RESISTANCE + RESISTANCE;
            let i_next = i + (v - i * r) * IND_SCALE;
            if let Some(cur) = me.currents.get_mut(s) {
                *cur = i_next;
            }
            // The parallel term carries the pre-update current (`I += V * condParallel`).
            let i_flow = i + v * COND_PARALLEL;
            me.apply_current(tick, -i_flow);
            other.apply_current(tick, i_flow);
            s += 1;
        }
        if me.endpoint {
            me.endpoint_update(tick);
        }
        self.nodes.insert(id, me);
    }

    /// Iterate every node once at `tick`, in ascending id order.
    pub fn tick(&mut self, tick: u64) {
        let mut ids: Vec<u64> = self.nodes.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            self.iterate(id, tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    /// `computeVoltage`: `Vcap += 0.05 * Icap * invCap`, once per tick, lazily on first read.
    #[test]
    fn voltage_calculation_is_exact_and_settles_once_per_tick() {
        let mut c = BlueConductor::endpoint();
        c.apply_current(1, 100.0);
        // Nothing is folded in until the next tick.
        assert_eq!(c.vcap, 0.0);
        // 0.05 * 100 * 0.25 = 1.25 V
        assert!(approx(c.voltage(2), 1.25), "v={}", c.voltage(2));
        // Reading again in the same tick must not fold again.
        assert!(approx(c.voltage(2), 1.25));
        assert_eq!(c.icap, 0.0);

        // A panel conductor has C = 0.25 F, invCap = 4: 0.05 * 10 * 4 = 2.0 V per amp-tick.
        let mut p = BlueConductor::panel();
        p.apply_current(1, 10.0);
        assert!(approx(p.voltage(2), 2.0));

        // it1 doubles into itot and resets.
        let mut d = BlueConductor::endpoint();
        d.apply_current(1, 3.0);
        d.apply_current(1, -5.0);
        assert!(approx(d.it1, 8.0));
        d.voltage(2);
        assert!(approx(d.itot, 4.0));
        assert_eq!(d.it1, 0.0);
    }

    /// The tick counter is compared modulo 65536, as world time is in the original.
    #[test]
    fn voltage_wraps_at_65536_ticks() {
        let mut c = BlueConductor::endpoint();
        c.last_tick = 5;
        c.vcap = 7.0;
        // Same low 16 bits: no recompute.
        assert_eq!(c.voltage(65_541), 7.0);
        // Different low 16 bits: recompute (no accumulated current, so the voltage is unchanged).
        assert_eq!(c.voltage(65_542), 7.0);
        assert_eq!(c.last_tick, 6);
    }

    /// One exact dynamic iteration step: I_next = I + (V - I*R)*0.07, R = 0.02 ohm.
    #[test]
    fn exact_current_iteration_step() {
        let mut g = BlueGrid::default();
        let mut a = BlueConductor::endpoint();
        a.vcap = 100.0;
        a.last_tick = 5;
        g.add(1, a);
        let mut b = BlueConductor::endpoint();
        b.last_tick = 5;
        g.add(2, b);
        g.link(1, 0, 2, 0);

        // V = 100 - 0 = 100, I = 0: I_next = 0 + (100 - 0) * 0.07 = 7.0
        g.iterate(1, 5);
        assert!(
            approx(g.nodes[&1].currents[0], 7.0),
            "{:?}",
            g.nodes[&1].currents
        );
        // I_flow = I + V*0.5 = 0 + 50 = 50 A: -50 into a, +50 into b.
        assert!(approx(g.nodes[&1].icap, -50.0));
        assert!(approx(g.nodes[&2].icap, 50.0));
        assert!(approx(g.nodes[&1].it1, 50.0));

        // Neighbour iterates the other way in the same tick: I_next = 0 + (0 - 100)*0.07 = -7.
        g.iterate(2, 5);
        assert!(approx(g.nodes[&2].currents[0], -7.0));
        // b sends 50 A back down the link as well: +50 received, +50 sent.
        assert!(approx(g.nodes[&2].icap, 100.0));
        assert!(approx(g.nodes[&1].icap, -100.0));

        // Next tick the capacitors settle: 0.05 * (-100) * 0.25 = -1.25 V on a, +1.25 V on b.
        let va = g.nodes.get_mut(&1).unwrap().voltage(6);
        let vb = g.nodes.get_mut(&2).unwrap().voltage(6);
        assert!(approx(va, 98.75), "va={va}");
        assert!(approx(vb, 1.25), "vb={vb}");

        // Second step with a live inductor current and a smaller potential difference:
        // V = 98.75 - 1.25 = 97.5, I = 7.0
        // I_next = 7.0 + (97.5 - 7.0 * 0.02) * 0.07 = 7.0 + 97.36 * 0.07 = 13.8152
        g.iterate(1, 6);
        assert!(approx(g.nodes[&1].currents[0], 13.8152));
        // I_flow = 7.0 + 97.5 * 0.5 = 55.75
        assert!(approx(g.nodes[&1].icap, -55.75));
        assert!(approx(g.nodes[&2].icap, 55.75));
    }

    /// The parallel term uses the current from *before* the update (`I += V * 0.5`), not `I_next`.
    #[test]
    fn parallel_flow_uses_pre_update_current() {
        let mut g = BlueGrid::default();
        let mut a = BlueConductor::endpoint();
        a.vcap = 100.0;
        a.last_tick = 5;
        a.add_slot(0);
        a.currents[0] = 10.0;
        g.add(1, a);
        let mut b = BlueConductor::endpoint();
        b.last_tick = 5;
        g.add(2, b);
        g.link(1, 0, 2, 0);
        g.nodes.get_mut(&1).unwrap().currents[0] = 10.0;

        g.iterate(1, 5);
        // I_next = 10 + (100 - 10*0.02)*0.07 = 16.986
        assert!(approx(g.nodes[&1].currents[0], 16.986));
        // I_flow = 10 + 50 = 60 (a spec reading of I_next + 50 would give 66.986).
        assert!(approx(g.nodes[&1].icap, -60.0));
        assert!(approx(g.nodes[&2].icap, 60.0));
    }

    /// A parallel pair of endpoints shares the flow: each link carries its own inductor current.
    #[test]
    fn parallel_links_carry_separate_currents() {
        let mut g = BlueGrid::default();
        let mut src = BlueConductor::endpoint();
        src.vcap = 100.0;
        src.last_tick = 1;
        g.add(1, src);
        for id in 2..=3 {
            let mut e = BlueConductor::endpoint();
            e.last_tick = 1;
            g.add(id, e);
        }
        g.link(1, 0, 2, 0);
        g.link(1, 1, 3, 0);

        g.iterate(1, 1);
        assert_eq!(g.nodes[&1].imask, 0b11);
        assert_eq!(g.nodes[&1].currents.len(), 2);
        // Both links see the same 100 V drop, so both inductor currents are 7 A.
        assert!(approx(g.nodes[&1].currents[0], 7.0));
        assert!(approx(g.nodes[&1].currents[1], 7.0));
        // Source loses 50 A per link, each sink gains 50 A.
        assert!(approx(g.nodes[&1].icap, -100.0));
        assert!(approx(g.nodes[&2].icap, 50.0));
        assert!(approx(g.nodes[&3].icap, 50.0));
    }

    /// Endpoint charge storage: `Charge = Vcap * 10`, `Flow` shifts in `Charge >= 600`.
    #[test]
    fn endpoint_charge_storage_and_flow_history() {
        let mut e = BlueConductor::endpoint();
        e.last_tick = 5;

        e.vcap = 80.0;
        e.endpoint_update(5);
        assert_eq!(e.charge, 800);
        assert_eq!(e.flow & 1, 1);

        e.vcap = 40.0;
        e.endpoint_update(6);
        assert_eq!(e.charge, 400);
        assert_eq!(e.flow & 1, 0);
        assert_eq!(e.flow, 0b10);

        e.vcap = 60.0;
        e.endpoint_update(7);
        assert_eq!(e.charge, 600);
        assert_eq!(e.flow, 0b101);

        // Gauges: charge out of 1000, flow out of 32 bits.
        assert_eq!(e.charge_scaled(10), 6);
        assert_eq!(e.flow_scaled(32), 2);
        // A plain conductor has neither.
        assert_eq!(BlueConductor::panel().charge_scaled(10), 0);
        assert_eq!(BlueConductor::panel().flow_scaled(32), 0);
    }

    /// Charge and flow survive a chunk reload along with the conductor state.
    #[test]
    fn endpoint_charge_persists() {
        let mut e = BlueConductor::endpoint();
        e.vcap = 80.0;
        e.last_tick = 5;
        e.endpoint_update(5);
        e.recache(CON_DIR_MASK[0], 0);
        e.currents[0] = 1.5;
        let mut nbt = HashMap::new();
        e.save(&mut nbt);
        assert_eq!(nbt["chg"], "800");
        assert_eq!(nbt["flw"], "1");

        let mut r = BlueConductor::endpoint();
        r.load(&nbt);
        assert_eq!(r.imask, 1);
        assert_eq!(r.currents, vec![1.5]);
        assert_eq!(r.vcap, 80.0);
        assert_eq!(r.charge, 800);
        assert_eq!(r.flow, 1);
    }

    /// A mismatched saved current list leaves the scalar state alone (`readFromNBT`).
    #[test]
    fn load_ignores_scalars_when_current_list_is_short() {
        let mut nbt = HashMap::new();
        nbt.insert("bpim".to_string(), "3".to_string());
        nbt.insert("bpil".to_string(), "1.0".to_string());
        nbt.insert("vcap".to_string(), "50.0".to_string());
        let mut c = BlueConductor::endpoint();
        c.load(&nbt);
        assert_eq!(c.imask, 3);
        assert_eq!(c.currents, vec![0.0, 0.0]);
        assert_eq!(c.vcap, 0.0);
    }

    /// `recache` maps a 30-bit connectable mask onto the axis slots and the edge mask onto the
    /// twelve edge slots, keeping the currents of surviving slots.
    #[test]
    fn recache_maps_axis_and_edge_bits() {
        // Face bit 24 (dir 0, -Y) and face bit 26 (dir 2, -Z): the Y and Z axis slots.
        let mut c = BlueConductor::endpoint();
        c.recache((1 << 24) | (1 << 26), 0);
        assert_eq!(c.imask, 0b011);
        assert_eq!(c.currents, vec![0.0, 0.0]);

        // Edge 11 is DIRMAP[8], so it sets bit 8 + 3 = 11 of the mask.
        c.recache(0, 1 << 11);
        assert_eq!(c.imask, 0b1000_0000_0000, "imask={:#b}", c.imask);
        assert_eq!(c.currents.len(), 1);

        // Keep a live current when the slot survives, zero the new ones.
        let mut d = BlueConductor::endpoint();
        d.recache(1 << 24, 0);
        d.currents[0] = 4.0;
        d.recache((1 << 24) | (1 << 28), 0);
        assert_eq!(d.imask, 0b101);
        assert_eq!(d.currents, vec![4.0, 0.0]);

        // No change: the mask and its currents are untouched.
        d.recache((1 << 24) | (1 << 28), 0);
        assert_eq!(d.currents, vec![4.0, 0.0]);
    }

    /// `drawPower` / `applyPower` are the original's energy <-> current conversions.
    #[test]
    fn draw_and_apply_power_match_the_formulas() {
        // Charging: 1,000 W into a 4 F endpoint at 100 V.
        let mut c = BlueConductor::endpoint();
        c.vcap = 100.0;
        c.last_tick = 1;
        c.apply_power(1, 1_000.0);
        let t = (100.0f64 * 100.0 + 0.1 * 1_000.0 * 0.25).sqrt() - 100.0;
        assert!(approx(c.icap, 20.0 * t / 0.25));
        let charged = c.voltage(2);

        // Discharging: the same power back out cannot push the voltage below zero.
        let mut d = BlueConductor::endpoint();
        d.vcap = 100.0;
        d.last_tick = 1;
        d.draw_power(1, 1_000.0);
        let t = (100.0f64 * 100.0 - 0.1 * 1_000.0 * 0.25).sqrt() - 100.0;
        assert!(approx(d.icap, 20.0 * t / 0.25));
        assert!(d.voltage(2) < 100.0);
        assert!(charged > 100.0);

        // Asking for more energy than the node holds draws nothing: the voltage stays put rather
        // than going negative (`t` is clamped to 0).
        let mut e = BlueConductor::endpoint();
        e.vcap = 10.0;
        e.last_tick = 1;
        e.draw_power(1, 1.0e9);
        assert!(approx(e.voltage(2), 10.0));
        assert!(approx(e.energy(2, 0.0), 200.0));
    }

    /// `getEnergy` is the energy held above a voltage threshold.
    #[test]
    fn energy_above_threshold() {
        let mut c = BlueConductor::endpoint();
        c.vcap = 100.0;
        c.last_tick = 3;
        // 0.5 * (100^2 - 60^2) / 0.25 = 0.5 * 6400 / 0.25 = 12,800
        assert!(approx(c.energy(3, 60.0), 12_800.0));
        assert!(approx(c.energy(3, 120.0), 0.0));
    }

    /// A slot whose neighbour is gone keeps its current and is skipped.
    #[test]
    fn missing_neighbour_skips_its_slot() {
        let mut g = BlueGrid::default();
        let mut a = BlueConductor::endpoint();
        a.vcap = 100.0;
        a.last_tick = 5;
        a.add_slot(0);
        a.add_slot(1);
        a.currents[0] = 3.0;
        g.add(1, a);
        g.link(1, 0, 2, 0); // node 2 never added
        g.iterate(1, 5);
        assert_eq!(g.nodes[&1].currents[0], 3.0);
        assert_eq!(g.nodes[&1].icap, 0.0);
    }

    /// `BlueGrid::tick` iterates every node once and runs the endpoint step for endpoints.
    #[test]
    fn grid_tick_runs_every_node_and_endpoint_step() {
        let mut g = BlueGrid::default();
        let mut a = BlueConductor::endpoint();
        a.vcap = 100.0;
        a.last_tick = 1;
        g.add(1, a);
        let mut b = BlueConductor::endpoint();
        b.last_tick = 1;
        g.add(2, b);
        g.link(1, 0, 2, 0);

        g.tick(1);
        // a iterates first (-50 to b), then b sends the mirror current back (+50): each side
        // counts the link once per iteration, as the original does.
        assert!(approx(g.nodes[&1].icap, -100.0));
        assert!(approx(g.nodes[&2].icap, 100.0));
        // Endpoint step ran at tick 1 with the un-settled voltage.
        assert_eq!(g.nodes[&1].charge, 1000);
        assert_eq!(g.nodes[&2].charge, 0);
        // Ticks 2..6 settle the capacitors; each endpoint keeps its history.
        for t in 2..7 {
            g.tick(t);
        }
        assert!(g.nodes[&1].vcap > 0.0 && g.nodes[&2].vcap > 0.0);
        assert_ne!(g.nodes[&1].flow, 0);
    }

    /// Removing a node drops the links that pointed at it.
    #[test]
    fn remove_drops_links() {
        let mut g = BlueGrid::default();
        g.add(1, BlueConductor::endpoint());
        g.add(2, BlueConductor::endpoint());
        g.link(1, 0, 2, 0);
        g.remove(2);
        g.iterate(1, 1);
        assert_eq!(g.nodes[&1].currents[0], 0.0);
    }
}
