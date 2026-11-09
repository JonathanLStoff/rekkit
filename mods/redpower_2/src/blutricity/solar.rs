//! Solar Panel (`redpower_2:solar_panel`): port of `TileSolarPanel`.
//!
//! Features:
//! - 1/4-block flat panel (`[0, 0, 0, 16, 4, 16]`).
//! - Generates 2.0A daytime current when seeing sky, no storm, and V <= 100V.
//! - Conductor dynamics with `inv_cap = 4.0` ($C = 0.25\text{ F}$).
//! - Equalizes charge with adjacent Blutricity conductors.
//! - Chunk reload persistence for conductor charge and current state.

use crate::BluePowerConductor;
use crate::{BLUTRICITY_CONDUCTOR_RESISTANCE, BLUTRICITY_NOMINAL_VOLTAGE};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const SOLAR_BLOCK_ID: &str = "redpower_2:solar_panel";
/// Tile entity name in `GameRegistry`.
pub const SOLAR_TE_NAME: &str = "RPSolar";
/// Metadata index in `BlockMachinePanel`.
pub const SOLAR_METADATA: u8 = 0;
/// Nominal cutoff voltage in Volts (100V).
pub const SOLAR_NOMINAL_VOLTAGE: f64 = BLUTRICITY_NOMINAL_VOLTAGE;
/// Current generated per tick in Amperes under direct sunlight.
pub const SOLAR_CURRENT_GEN: f64 = 2.0;
/// Capacitance in Farads ($C = 0.25\text{ F}$, $1/C = 4.0$).
pub const SOLAR_CAPACITANCE: f64 = 0.25;
/// Bounding box of the 1/4 block panel (x1, y1, z1, x2, y2, z2).
pub const SOLAR_PART_BOUNDS: [f32; 6] = [0.0, 0.0, 0.0, 1.0, 0.25, 1.0];

/// Environmental factors read from world context per tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolarEnvironment {
    /// Sky light level at the block directly above (0..=15).
    pub sky_light: u8,
    /// Time of day in world ticks (0..24000).
    pub day_time: i64,
    /// Rain strength (0.0..=1.0).
    pub rain: f32,
    /// Thunder strength (0.0..=1.0).
    pub thunder: f32,
}

impl SolarEnvironment {
    /// Returns true if the block can see the open sky (sky light >= 15).
    pub fn sees_sky(&self) -> bool {
        self.sky_light >= 15
    }

    /// Returns true if it is daytime (ticks 0..12000 of the 24000-tick day cycle).
    pub fn is_day(&self) -> bool {
        self.day_time.rem_euclid(24000) < 12000
    }

    /// Returns true if there is rain or thunderstorm.
    pub fn is_storm(&self) -> bool {
        self.rain > 0.2 || self.thunder > 0.2
    }
}

/// State machine for `TileSolarPanel`.
#[derive(Debug, Clone, PartialEq)]
pub struct TileSolarPanel {
    /// RLC Blutricity conductor.
    pub cond: BluePowerConductor,
    /// Connection mask for adjacent conductors.
    pub con_mask: i32,
}

impl Default for TileSolarPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TileSolarPanel {
    /// Creates a new `TileSolarPanel` instance with $C = 0.25\text{ F}$.
    pub fn new() -> Self {
        Self {
            cond: BluePowerConductor::new(SOLAR_CAPACITANCE),
            con_mask: -1,
        }
    }

    /// Returns the current voltage across the panel capacitor.
    pub fn voltage(&self) -> f64 {
        self.cond.voltage
    }

    /// Advances one game tick given environmental readings.
    ///
    /// Generates 2.0A direct current when:
    /// - Voltage does not exceed nominal cutoff ($V \le 100\text{V}$)
    /// - The block can see the sky
    /// - It is daytime
    /// - There is no storm
    pub fn tick(&mut self, sees_sky: bool, is_day: bool, is_storm: bool) -> f64 {
        if sees_sky && is_day && !is_storm && self.cond.voltage <= SOLAR_NOMINAL_VOLTAGE {
            self.cond.add_charge(SOLAR_CURRENT_GEN);
            SOLAR_CURRENT_GEN
        } else {
            0.0
        }
    }

    /// Advances one game tick against a `SolarEnvironment` snapshot.
    pub fn tick_env(&mut self, env: &SolarEnvironment) -> f64 {
        self.tick(env.sees_sky(), env.is_day(), env.is_storm())
    }

    /// Equalizes charge and steps RLC iteration against an adjacent conductor.
    pub fn conduct(&mut self, other: &mut BluePowerConductor) -> f64 {
        self.cond.step_iterate(other)
    }

    /// Deserializes state from NBT string map for chunk reload.
    pub fn from_nbt(nbt: &HashMap<String, String>) -> Self {
        let mut panel = Self::new();
        let get = |k: &str| nbt.get(k).and_then(|v| v.parse::<f64>().ok());
        panel.cond.charge = get("charge").unwrap_or(0.0).max(0.0);
        panel.cond.current = get("current").unwrap_or(0.0);
        panel.cond.resistance = get("resistance").unwrap_or(BLUTRICITY_CONDUCTOR_RESISTANCE);
        panel.cond.capacitance = get("capacitance").unwrap_or(SOLAR_CAPACITANCE).max(0.01);
        panel.cond.voltage = panel.cond.charge / panel.cond.capacitance;
        if let Some(mask) = nbt.get("con_mask").and_then(|v| v.parse::<i32>().ok()) {
            panel.con_mask = mask;
        }
        panel
    }

    /// Serializes state into NBT string map for chunk reload persistence.
    pub fn to_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("charge".into(), self.cond.charge.to_string());
        nbt.insert("current".into(), self.cond.current.to_string());
        nbt.insert("resistance".into(), self.cond.resistance.to_string());
        nbt.insert("capacitance".into(), self.cond.capacitance.to_string());
        nbt.insert("con_mask".into(), self.con_mask.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_panel_daylight_generation() {
        let mut solar = TileSolarPanel::new();
        assert_eq!(solar.voltage(), 0.0);

        // Daytime, clear sky, sees sky -> should generate 2.0A
        let generated = solar.tick(true, true, false);
        assert_eq!(generated, 2.0);
        // Voltage = Charge / Capacitance = 2.0 / 0.25 = 8.0V
        assert!((solar.voltage() - 8.0).abs() < 1e-6);

        // Night time -> 0 generation
        let gen_night = solar.tick(true, false, false);
        assert_eq!(gen_night, 0.0);
        assert!((solar.voltage() - 8.0).abs() < 1e-6);

        // Obstructed sky -> 0 generation
        let gen_obstructed = solar.tick(false, true, false);
        assert_eq!(gen_obstructed, 0.0);

        // Storm -> 0 generation
        let gen_storm = solar.tick(true, true, true);
        assert_eq!(gen_storm, 0.0);
    }

    #[test]
    fn test_solar_panel_voltage_cutoff() {
        let mut solar = TileSolarPanel::new();
        solar.cond.voltage = 100.1;
        solar.cond.charge = 100.1 * 0.25;

        // V > 100.0V -> should not generate even in ideal conditions
        let generated = solar.tick(true, true, false);
        assert_eq!(generated, 0.0);

        // V <= 100.0V -> should generate
        solar.cond.voltage = 100.0;
        solar.cond.charge = 100.0 * 0.25;
        let generated = solar.tick(true, true, false);
        assert_eq!(generated, 2.0);
    }

    #[test]
    fn test_solar_environment_checks() {
        let env_day = SolarEnvironment {
            sky_light: 15,
            day_time: 6000,
            rain: 0.0,
            thunder: 0.0,
        };
        assert!(env_day.sees_sky());
        assert!(env_day.is_day());
        assert!(!env_day.is_storm());

        let mut solar = TileSolarPanel::new();
        let gen = solar.tick_env(&env_day);
        assert_eq!(gen, 2.0);

        let env_night = SolarEnvironment {
            sky_light: 15,
            day_time: 18000,
            rain: 0.0,
            thunder: 0.0,
        };
        assert!(!env_night.is_day());
        assert_eq!(solar.tick_env(&env_night), 0.0);
    }

    #[test]
    fn test_solar_panel_conduction() {
        let mut solar = TileSolarPanel::new();
        solar.cond.add_charge(10.0); // V = 10 / 0.25 = 40V

        let mut wire = BluePowerConductor::new(0.25);
        assert_eq!(wire.voltage, 0.0);

        let flow = solar.conduct(&mut wire);
        assert!(flow > 0.0);
        assert!(wire.voltage > 0.0);
        assert!(solar.voltage() < 40.0);
    }

    #[test]
    fn test_solar_panel_chunk_reload_persistence() {
        let mut solar = TileSolarPanel::new();
        solar.tick(true, true, false);
        solar.tick(true, true, false);
        solar.con_mask = 42;

        let mut nbt = HashMap::new();
        solar.to_nbt(&mut nbt);

        let restored = TileSolarPanel::from_nbt(&nbt);
        assert_eq!(restored.con_mask, 42);
        assert!((restored.voltage() - solar.voltage()).abs() < 1e-6);
        assert!((restored.cond.charge - solar.cond.charge).abs() < 1e-6);
        assert_eq!(restored.cond.resistance, solar.cond.resistance);
    }
}
