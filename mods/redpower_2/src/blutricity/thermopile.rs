//! Thermopile (`redpower_2:thermopile`): port of `TileThermopile`.
//!
//! Features:
//! - Generates Blutricity from temperature gradients between adjacent blocks.
//! - Scans 6 adjacent faces every 20 ticks.
//! - Generation rate: $I = 0.005\text{ A} \times \min(T_{\text{hot}}, T_{\text{cold}})$.
//! - Nominal cutoff voltage at 100.0V.
//! - Thermal interaction hazards:
//!   - If $T_{\text{hot}} \ge 100$ and $T_{\text{cold}} \ge 200$, adjacent lava cools into stone/cobble.
//!   - If $T_{\text{hot}} \ge 100$, adjacent ice, snow and water may melt.
//! - Conductor dynamics with `inv_cap = 0.25` ($C = 4.0\text{ F}$).
//! - Chunk reload persistence for conductor state, tick counter, and temperatures.

use crate::BluePowerConductor;
use crate::{BLUTRICITY_CONDUCTOR_RESISTANCE, BLUTRICITY_NOMINAL_VOLTAGE};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const THERMOPILE_BLOCK_ID: &str = "redpower_2:thermopile";
/// Tile entity name in `GameRegistry`.
pub const THERMOPILE_TE_NAME: &str = "RPThermopile";
/// Metadata index in `BlockMachine`.
pub const THERMOPILE_METADATA: u8 = 11;
/// Nominal cutoff voltage in Volts (100V).
pub const THERMOPILE_NOMINAL_VOLTAGE: f64 = BLUTRICITY_NOMINAL_VOLTAGE;
/// Current multiplier per degree of thermal gradient ($0.005\text{ A}$).
pub const THERMOPILE_GEN_RATE: f64 = 0.005;
/// Scan interval in ticks (20 ticks = 1 second).
pub const THERMOPILE_SCAN_INTERVAL: u32 = 20;
/// Capacitance in Farads ($C = 4.0\text{ F}$, $1/C = 0.25$).
pub const THERMOPILE_CAPACITANCE: f64 = 4.0;

/// State machine for `TileThermopile`.
#[derive(Debug, Clone, PartialEq)]
pub struct TileThermopile {
    /// RLC Blutricity conductor.
    pub cond: BluePowerConductor,
    /// Tick counter tracking the 20-tick rescan cycle.
    pub ticks: u32,
    /// Cumulative heat score from hot neighbors ($T_{\text{hot}}$).
    pub t_hot: i32,
    /// Cumulative cold score from cold neighbors ($T_{\text{cold}}$).
    pub t_cold: i32,
    /// Connection mask for adjacent conductors.
    pub con_mask: i32,
}

impl Default for TileThermopile {
    fn default() -> Self {
        Self::new()
    }
}

impl TileThermopile {
    /// Creates a new `TileThermopile` instance with $C = 4.0\text{ F}$.
    pub fn new() -> Self {
        Self {
            cond: BluePowerConductor::new(THERMOPILE_CAPACITANCE),
            ticks: 0,
            t_hot: 0,
            t_cold: 0,
            con_mask: -1,
        }
    }

    /// Returns the current voltage across the thermopile capacitor.
    pub fn voltage(&self) -> f64 {
        self.cond.voltage
    }

    /// Evaluates the thermal contribution `(hot, cold)` of a single block type.
    ///
    /// Following `TileThermopile.updateTemps` in RedPower 2:
    /// - Air: +1 Nether, -1 Overworld (cold +1)
    /// - Ice, Snow Block: cold +100
    /// - Snow layer: cold +50
    /// - Torch: hot +5
    /// - Fire: hot +3
    /// - Water / Flowing Water: cold +25
    /// - Lava / Flowing Lava: hot +100
    /// - Lit Furnace / Burning Furnace: hot +25
    pub fn block_heat(block_id: &str, is_nether: bool) -> (i32, i32) {
        let id = block_id.strip_prefix("minecraft:").unwrap_or(block_id);
        match id {
            "air" | "cave_air" | "void_air" => {
                if is_nether {
                    (1, 0)
                } else {
                    (0, 1)
                }
            }
            "ice" | "snow_block" => (0, 100),
            "snow" => (0, 50),
            "water" | "flowing_water" => (0, 25),
            "lava" | "flowing_lava" => (100, 0),
            "torch" | "wall_torch" => (5, 0),
            "fire" => (3, 0),
            "lit_furnace" | "burning_furnace" => (25, 0),
            _ => (0, 0),
        }
    }

    /// Computes total `(t_hot, t_cold)` across 6 adjacent neighbor faces.
    pub fn scan(neighbors: &[&str; 6], is_nether: bool) -> (i32, i32) {
        neighbors.iter().fold((0, 0), |(hot_acc, cold_acc), id| {
            let (h, c) = Self::block_heat(id, is_nether);
            (hot_acc + h, cold_acc + c)
        })
    }

    /// Whether thermal conditions permit adjacent lava to cool into stone ($T_{\text{hot}} \ge 100 \land T_{\text{cold}} \ge 200$).
    pub fn lava_cools(&self) -> bool {
        self.t_hot >= 100 && self.t_cold >= 200
    }

    /// Whether thermal conditions permit melting adjacent ice, snow or water ($T_{\text{hot}} \ge 100$).
    pub fn melts_neighbors(&self) -> bool {
        self.t_hot >= 100
    }

    /// Returns true if the next tick will trigger a neighbor rescan.
    pub fn will_rescan(&self) -> bool {
        self.cond.voltage <= THERMOPILE_NOMINAL_VOLTAGE && self.ticks + 1 > THERMOPILE_SCAN_INTERVAL
    }

    /// Advances one game tick.
    ///
    /// When voltage $\le 100\text{V}$, the internal counter advances.
    /// Every 20 ticks, `scan()` is invoked to update $T_{\text{hot}}$ and $T_{\text{cold}}$.
    /// Generates $0.005\text{ A} \times \min(T_{\text{hot}}, T_{\text{cold}})$ current into the conductor.
    /// Returns `(current_generated, did_rescan)`.
    pub fn tick(&mut self, scan: impl FnOnce() -> (i32, i32)) -> (f64, bool) {
        if self.cond.voltage > THERMOPILE_NOMINAL_VOLTAGE {
            return (0.0, false);
        }

        self.ticks += 1;
        let mut rescanned = false;
        if self.ticks > THERMOPILE_SCAN_INTERVAL {
            self.ticks = 0;
            let (hot, cold) = scan();
            self.t_hot = hot;
            self.t_cold = cold;
            rescanned = true;
        }

        let min_temp = self.t_hot.min(self.t_cold).max(0);
        let gen = THERMOPILE_GEN_RATE * (min_temp as f64);
        self.cond.add_charge(gen);
        (gen, rescanned)
    }

    /// Equalizes charge and steps RLC iteration against an adjacent conductor.
    pub fn conduct(&mut self, other: &mut BluePowerConductor) -> f64 {
        self.cond.step_iterate(other)
    }

    /// Deserializes state from NBT string map for chunk reload.
    pub fn from_nbt(nbt: &HashMap<String, String>) -> Self {
        let mut thermo = Self::new();
        let get = |k: &str| nbt.get(k).and_then(|v| v.parse::<f64>().ok());
        thermo.cond.charge = get("charge").unwrap_or(0.0).max(0.0);
        thermo.cond.current = get("current").unwrap_or(0.0);
        thermo.cond.resistance = get("resistance").unwrap_or(BLUTRICITY_CONDUCTOR_RESISTANCE);
        thermo.cond.capacitance = get("capacitance")
            .unwrap_or(THERMOPILE_CAPACITANCE)
            .max(0.01);
        thermo.cond.voltage = thermo.cond.charge / thermo.cond.capacitance;
        if let Some(t) = nbt.get("ticks").and_then(|v| v.parse::<u32>().ok()) {
            thermo.ticks = t;
        }
        if let Some(th) = nbt.get("t_hot").and_then(|v| v.parse::<i32>().ok()) {
            thermo.t_hot = th;
        }
        if let Some(tc) = nbt.get("t_cold").and_then(|v| v.parse::<i32>().ok()) {
            thermo.t_cold = tc;
        }
        if let Some(mask) = nbt.get("con_mask").and_then(|v| v.parse::<i32>().ok()) {
            thermo.con_mask = mask;
        }
        thermo
    }

    /// Serializes state into NBT string map for chunk reload persistence.
    pub fn to_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("charge".into(), self.cond.charge.to_string());
        nbt.insert("current".into(), self.cond.current.to_string());
        nbt.insert("resistance".into(), self.cond.resistance.to_string());
        nbt.insert("capacitance".into(), self.cond.capacitance.to_string());
        nbt.insert("ticks".into(), self.ticks.to_string());
        nbt.insert("t_hot".into(), self.t_hot.to_string());
        nbt.insert("t_cold".into(), self.t_cold.to_string());
        nbt.insert("con_mask".into(), self.con_mask.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermopile_differential_generation() {
        let mut therm = TileThermopile::new();
        assert_eq!(therm.voltage(), 0.0);

        // Surrounding: 1 lava (+100) and 1 water (+25 cold), rest air
        let neighbors = [
            "minecraft:lava",
            "minecraft:water",
            "minecraft:air",
            "minecraft:air",
            "minecraft:air",
            "minecraft:air",
        ];
        let (hot, cold) = TileThermopile::scan(&neighbors, false);
        assert_eq!(hot, 100);
        // cold: 25 (water) + 4 (air * 4) = 29
        assert_eq!(cold, 29);

        // Advance 21 ticks to trigger initial scan
        for _ in 0..20 {
            therm.tick(|| (hot, cold));
        }
        let (gen, rescanned) = therm.tick(|| (hot, cold));
        assert!(rescanned);
        assert_eq!(therm.t_hot, 100);
        assert_eq!(therm.t_cold, 29);
        // Generation: 0.005 * min(100, 29) = 0.005 * 29 = 0.145A
        assert!((gen - 0.145).abs() < 1e-6);
        assert!(therm.voltage() > 0.0);
    }

    #[test]
    fn test_thermopile_ice_lava_gradient() {
        let mut therm = TileThermopile::new();
        let neighbors = [
            "minecraft:lava",
            "minecraft:ice",
            "minecraft:lava",
            "minecraft:ice",
            "minecraft:air",
            "minecraft:air",
        ];
        let (hot, cold) = TileThermopile::scan(&neighbors, false);
        // hot: 200, cold: 202
        assert_eq!(hot, 200);
        assert_eq!(cold, 202);

        therm.ticks = 20;
        let (gen, _) = therm.tick(|| (hot, cold));
        // min = 200, gen = 0.005 * 200 = 1.0A
        assert_eq!(gen, 1.0);
    }

    #[test]
    fn test_thermopile_overvoltage_cutoff() {
        let mut therm = TileThermopile::new();
        therm.t_hot = 100;
        therm.t_cold = 100;
        therm.cond.voltage = 100.1;

        let (gen, rescanned) = therm.tick(|| (100, 100));
        assert_eq!(gen, 0.0);
        assert!(!rescanned);
        assert_eq!(therm.ticks, 0); // ticks do not advance when V > 100V
    }

    #[test]
    fn test_thermal_hazard_checks() {
        let mut therm = TileThermopile::new();
        therm.t_hot = 99;
        therm.t_cold = 200;
        assert!(!therm.lava_cools());
        assert!(!therm.melts_neighbors());

        therm.t_hot = 100;
        therm.t_cold = 199;
        assert!(!therm.lava_cools());
        assert!(therm.melts_neighbors());

        therm.t_hot = 100;
        therm.t_cold = 200;
        assert!(therm.lava_cools());
        assert!(therm.melts_neighbors());
    }

    #[test]
    fn test_thermopile_conduction() {
        let mut therm = TileThermopile::new();
        therm.cond.add_charge(40.0); // V = 40 / 4.0 = 10V

        let mut wire = BluePowerConductor::new(0.25);
        let flow = therm.conduct(&mut wire);
        assert!(flow > 0.0);
        assert!(wire.voltage > 0.0);
    }

    #[test]
    fn test_thermopile_chunk_reload_persistence() {
        let mut therm = TileThermopile::new();
        therm.ticks = 17;
        therm.t_hot = 100;
        therm.t_cold = 50;
        therm.cond.add_charge(8.0);
        therm.con_mask = 63;

        let mut nbt = HashMap::new();
        therm.to_nbt(&mut nbt);

        let restored = TileThermopile::from_nbt(&nbt);
        assert_eq!(restored.ticks, 17);
        assert_eq!(restored.t_hot, 100);
        assert_eq!(restored.t_cold, 50);
        assert_eq!(restored.con_mask, 63);
        assert!((restored.voltage() - therm.voltage()).abs() < 1e-6);
        assert!((restored.cond.charge - therm.cond.charge).abs() < 1e-6);
    }
}
