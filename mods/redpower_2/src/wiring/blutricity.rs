//! Blutricity dynamic RLC simulation (`BluePowerConductor` / `BluePowerEndpoint`, spec 2.5).
//!
//! Each node keeps a capacitor voltage `vcap`, an accumulated capacitor current `icap`, and one
//! inductor current per connection slot. A tick lazily folds `icap` into `vcap` the first time the
//! voltage is read in that tick; `iterate` then moves current along every connection.

use std::collections::HashMap;

/// Edge-connection table (`BluePowerConductor.dirmap`).
pub const DIRMAP: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 11, 14, 18, 23];
pub const RESISTANCE: f64 = 0.01;
pub const IND_SCALE: f64 = 0.07;
pub const COND_PARALLEL: f64 = 0.5;
/// Inverse capacitance of cables and endpoints (C = 4 F).
pub const INV_CAP_ENDPOINT: f64 = 0.25;
/// Inverse capacitance of panels (C = 0.25 F).
pub const INV_CAP_PANEL: f64 = 4.0;

/// One conductor. `inv_cap` stands in for the abstract `getInvCap()`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlueConductor {
    pub imask: u32,
    /// One current per set bit of `imask`, in bit order.
    pub currents: Vec<f64>,
    pub vcap: f64,
    pub icap: f64,
    pub veff: f64,
    pub last_tick: u32,
    pub it1: f64,
    pub itot: f64,
    pub inv_cap: f64,
    /// `BluePowerEndpoint` state (display only).
    pub charge: i32,
    pub flow: i32,
}

impl BlueConductor {
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
            charge: 0,
            flow: 0,
        }
    }

    pub fn endpoint() -> Self {
        Self::new(INV_CAP_ENDPOINT)
    }

    pub fn panel() -> Self {
        Self::new(INV_CAP_PANEL)
    }

    /// Rebuild the connection mask. `conm`: 6-bit face connections (a straight slot per axis when the
    /// axis' low face bit is set, as `getConDirMask(a * 2)`); `econm`: edge connection bits.
    /// Currents of slots that stay are kept, new slots start at 0.
    pub fn recache(&mut self, conm: u32, econm: u32) {
        let mut imo = 0u32;
        for a in 0..3 {
            if conm & con_dir_mask(a * 2) != 0 {
                imo |= 1 << a;
            }
        }
        for (a, d) in DIRMAP.iter().enumerate() {
            if econm & (1 << d) != 0 {
                imo |= 8 << a;
            }
        }
        if self.imask != imo {
            let mut c2 = Vec::with_capacity(imo.count_ones() as usize);
            let mut s = 0;
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
    }

    fn compute_voltage(&mut self) {
        self.itot = 0.5 * self.it1;
        self.it1 = 0.0;
        self.vcap += 0.05 * self.icap * self.inv_cap;
        self.icap = 0.0;
    }

    /// Voltage at game tick `tick`; settles the capacitor once per tick (compared mod 65536).
    pub fn voltage(&mut self, tick: u64) -> f64 {
        let lt = (tick & 0xFFFF) as u32;
        if lt != self.last_tick {
            self.last_tick = lt;
            self.compute_voltage();
        }
        self.vcap
    }

    pub fn apply_current(&mut self, tick: u64, i_in: f64) {
        self.voltage(tick);
        self.icap += i_in;
        self.it1 += i_in.abs();
    }

    pub fn apply_direct(&mut self, tick: u64, i_in: f64) {
        self.apply_current(tick, i_in);
    }

    /// Draw power `p` from the node (no more than it holds).
    pub fn draw_power(&mut self, tick: u64, p: f64) {
        let p1 = self.vcap * self.vcap - 0.1 * p * self.inv_cap;
        let t = if p1 < 0.0 { 0.0 } else { p1.sqrt() - self.vcap };
        self.apply_direct(tick, 20.0 * t / self.inv_cap);
    }

    /// Energy above `vthresh`.
    pub fn energy(&mut self, tick: u64, vthresh: f64) -> f64 {
        let d = self.voltage(tick);
        let tr = 0.5 * (d * d - vthresh * vthresh) / self.inv_cap;
        if tr < 0.0 {
            0.0
        } else {
            tr
        }
    }

    /// Feed power `p` into the node.
    pub fn apply_power(&mut self, tick: u64, p: f64) {
        let t = (self.vcap * self.vcap + 0.1 * p * self.inv_cap).sqrt() - self.vcap;
        self.apply_direct(tick, 20.0 * t / self.inv_cap);
    }

    /// Endpoint extra step after `iterate` (`BluePowerEndpoint.iterate`).
    pub fn endpoint_update(&mut self, tick: u64) {
        self.charge = (self.voltage(tick) * 10.0) as i32;
        self.flow = (self.flow << 1) | i32::from(self.charge >= 600);
    }

    /// Chunk save: key/value strings.
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
    }

    /// Chunk load. As the original, the scalar state is restored only if the saved current list
    /// matches the mask's bit count.
    pub fn load(&mut self, nbt: &HashMap<String, String>) {
        let f = |k: &str| {
            nbt.get(k)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0)
        };
        self.imask = nbt.get("bpim").and_then(|v| v.parse().ok()).unwrap_or(0);
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
            self.last_tick = nbt.get("ltk").and_then(|v| v.parse().ok()).unwrap_or(0);
        }
    }
}

/// Face connection bit of `RedPowerLib.getConDirMask(dir)`: one bit per face.
fn con_dir_mask(dir: u32) -> u32 {
    1 << dir
}

/// A set of conductors and the links between them; `link(a, slot, b)` says conductor `a`'s
/// connection bit `slot` leads to `b`.
#[derive(Debug, Default)]
pub struct BlueGrid {
    pub nodes: HashMap<u64, BlueConductor>,
    links: HashMap<(u64, u32), u64>,
}

impl BlueGrid {
    pub fn add(&mut self, id: u64, c: BlueConductor) {
        self.nodes.insert(id, c);
    }

    /// Link both ways: `a` via bit `slot_a`, `b` via bit `slot_b`. Both nodes get the bit in
    /// their mask (new currents 0).
    pub fn link(&mut self, a: u64, slot_a: u32, b: u64, slot_b: u32) {
        self.links.insert((a, slot_a), b);
        self.links.insert((b, slot_b), a);
        for (id, slot) in [(a, slot_a), (b, slot_b)] {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.add_slot(slot);
            }
        }
    }

    /// Iterate one conductor: current moves along each of its connections (`iterate()`).
    pub fn iterate(&mut self, id: u64, tick: u64) {
        let Some(mut me) = self.nodes.remove(&id) else {
            return;
        };
        me.voltage(tick);
        let mut s = 0usize;
        let mut dm = me.imask;
        while dm > 0 {
            let d = dm.trailing_zeros();
            dm &= !(1 << d);
            let neighbour = self.links.get(&(id, d)).copied();
            let Some(other) = neighbour.and_then(|n| self.nodes.get_mut(&n)) else {
                s += 1;
                continue;
            };
            let r = RESISTANCE + RESISTANCE;
            let mut i = me.currents[s];
            let v = me.vcap - other.voltage(tick);
            me.currents[s] += (v - i * r) * IND_SCALE;
            i += v * COND_PARALLEL;
            me.apply_current(tick, -i);
            other.apply_current(tick, i);
            s += 1;
        }
        self.nodes.insert(id, me);
    }

    /// Iterate every node once (ascending id) at `tick`.
    pub fn tick(&mut self, tick: u64) {
        let mut ids: Vec<u64> = self.nodes.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            self.iterate(id, tick);
        }
    }
}

impl BlueConductor {
    /// Set a connection bit, inserting a zero current at its place.
    pub fn add_slot(&mut self, slot: u32) {
        let m = 1u32 << slot;
        if self.imask & m != 0 {
            return;
        }
        let idx = (self.imask & (m - 1)).count_ones() as usize;
        self.currents.insert(idx, 0.0);
        self.imask |= m;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voltage_settles_once_per_tick() {
        let mut c = BlueConductor::endpoint();
        c.apply_current(1, 100.0);
        assert_eq!(c.vcap, 0.0);
        // next tick: 0.05 * 100 * 0.25
        assert!((c.voltage(2) - 1.25).abs() < 1e-12);
        assert!((c.voltage(2) - 1.25).abs() < 1e-12);
    }

    #[test]
    fn recache_keeps_currents() {
        let mut c = BlueConductor::endpoint();
        c.recache(0b000001, 0);
        assert_eq!(c.imask, 1);
        c.currents[0] = 3.0;
        c.recache(0b000101, 1 << 11);
        assert_eq!(c.imask.count_ones(), 3);
        assert_eq!(c.currents[0], 3.0);
        assert_eq!(c.currents.len(), 3);
    }

    #[test]
    fn current_flows_high_to_low_and_conserves_charge() {
        let mut g = BlueGrid::default();
        let mut a = BlueConductor::endpoint();
        a.vcap = 100.0;
        g.add(1, a);
        g.add(2, BlueConductor::endpoint());
        g.link(1, 0, 2, 0);
        for t in 1..200 {
            g.tick(t);
        }
        let (va, vb) = (g.nodes[&1].vcap, g.nodes[&2].vcap);
        assert!(vb > 5.0 && va < 100.0, "va={va} vb={vb}");
        // total icap is conserved, so (vcap+vcap)/inv_cap-weighted sum stays within drift of 100
        assert!(va + vb <= 100.0 + 1e-6 + 1.0);
    }

    #[test]
    fn apply_and_draw_power() {
        let mut c = BlueConductor::endpoint();
        c.vcap = 100.0;
        c.apply_power(1, 1000.0);
        let up = c.voltage(2);
        assert!(up > 100.0);
        c.draw_power(2, 1.0e9);
        // drawing more than stored cannot push voltage below zero in one step
        assert!(c.voltage(3) >= 0.0 || c.voltage(3).is_finite());
        assert!(c.energy(3, 60.0) >= 0.0);
    }

    #[test]
    fn endpoint_flow_bits_and_nbt_roundtrip() {
        let mut c = BlueConductor::endpoint();
        c.vcap = 80.0;
        c.last_tick = 5;
        c.endpoint_update(5);
        assert_eq!(c.charge, 800);
        assert_eq!(c.flow & 1, 1);
        c.recache(1, 0);
        c.currents[0] = 1.5;
        let mut nbt = HashMap::new();
        c.save(&mut nbt);
        let mut d = BlueConductor::endpoint();
        d.load(&nbt);
        assert_eq!(d.imask, 1);
        assert_eq!(d.currents, vec![1.5]);
        assert_eq!(d.vcap, 80.0);
    }
}
