//! Wind Turbine (`redpower_2:wind_turbine`): port of `TileWindTurbine` and `ContainerWindTurbine`.
//!
//! Features:
//! - 1-slot GUI layout at `[80, 35]` holding the turbine rotor (`ItemWindmill` / `wood_sail`).
//! - Raytraces clear air space in front of the rotor across 289 samples for up to 5780 efficiency.
//! - Rotor durability degradation: 1 point of durability lost per 6600 ticks of operation.
//! - Broken rotor converts into broken item / wooden sail (`redpower_2:wood_sail`).
//! - Mechanical redstone brake: halts rotation and generation when powered by redstone.
//! - Automatic overvoltage brake: engages at $V > 110.0\text{V}$, releases at $V < 100.0\text{V}$.
//! - Power application into Blutricity grid up to $130.0\text{V}$ safe limit.
//! - Chunk reload persistence for rotor slot, power state, efficiency, and brake engagement.

use crate::BluePowerConductor;
use crate::{
    ItemStack, BLUTRICITY_CONDUCTOR_RESISTANCE, BLUTRICITY_NOMINAL_VOLTAGE,
    BLUTRICITY_WIND_MAX_VOLTAGE,
};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const TURBINE_BLOCK_ID: &str = "redpower_2:wind_turbine";
/// Tile entity name in `GameRegistry`.
pub const TURBINE_TE_NAME: &str = "RPWindTurbine";
/// Wooden sail item ID.
pub const WOOD_SAIL_ID: &str = "redpower_2:wood_sail";
/// Wooden windmill item ID.
pub const WINDMILL_WOOD_ID: &str = "redpower_2:windmill_wood";
/// Metadata index in `BlockMachine`.
pub const TURBINE_METADATA: u8 = 9;
/// Overvoltage brake threshold in Volts (110V).
pub const TURBINE_BRAKE_VOLTAGE: f64 = 110.0;
/// Brake release voltage in Volts (100V).
pub const TURBINE_BRAKE_RELEASE_VOLTAGE: f64 = BLUTRICITY_NOMINAL_VOLTAGE;
/// Maximum operational voltage in Volts (130V).
pub const TURBINE_MAX_SAFE_VOLTAGE: f64 = BLUTRICITY_WIND_MAX_VOLTAGE;
/// Default capacitance in Farads ($C = 4.0\text{ F}$, $1/C = 0.25$).
pub const TURBINE_CAPACITANCE: f64 = 4.0;
/// Ticks of continuous operation per point of rotor durability damage (6600 ticks).
pub const ROTOR_DEGRADE_INTERVAL: u32 = 6600;
/// Default maximum durability for a wooden windmill rotor.
pub const ROTOR_DEFAULT_MAX_DURABILITY: u16 = 1500;
/// Total raytrace sample buffer size (17x17 = 289 samples).
pub const RAYTRACE_SAMPLES: usize = 289;
/// Maximum efficiency rating with full 20-block clearance (289 * 20 = 5780).
pub const MAX_EFFICIENCY: i32 = 5780;

/// Windmill rotor orientation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindmillType {
    /// Vertical axis turbine (type 1).
    Vertical = 1,
    /// Horizontal axis turbine (type 2).
    Horizontal = 2,
}

/// State machine for `TileWindTurbine`.
#[derive(Debug, Clone, PartialEq)]
pub struct TileWindTurbine {
    /// RLC Blutricity conductor.
    pub cond: BluePowerConductor,
    /// Mounted rotor item stack in the 1-slot inventory.
    pub rotor: Option<ItemStack>,
    /// Whether blades are actively deployed.
    pub has_blades: bool,
    /// Whether mechanical brakes are actively engaged.
    pub has_brakes: bool,
    /// External redstone brake signal applied.
    pub redstone_brake: bool,
    /// Windmill rotor type (1 = vertical, 2 = horizontal).
    pub windmill_type: u8,
    /// Countdown ticks until next rotor durability loss.
    pub prop_ticks: u32,
    /// Current electrical output power in Watts.
    pub power: i32,
    /// Mechanical rotational speed.
    pub speed: i32,
    /// Rotational phase for visual rendering.
    pub phase: i32,
    /// Obstruction clearance efficiency rating (0..=5780).
    pub efficiency: i32,
    /// Raytrace sample history ring buffer.
    pub ray_trace: [u8; RAYTRACE_SAMPLES],
    /// Ring buffer pointer for next raytrace sample.
    pub tracer: usize,
    /// Conductor connection mask.
    pub con_mask: i32,
}

impl Default for TileWindTurbine {
    fn default() -> Self {
        Self::new()
    }
}

impl TileWindTurbine {
    /// Creates a new `TileWindTurbine` instance.
    pub fn new() -> Self {
        Self {
            cond: BluePowerConductor::new(TURBINE_CAPACITANCE),
            rotor: None,
            has_blades: false,
            has_brakes: false,
            redstone_brake: false,
            windmill_type: WindmillType::Horizontal as u8,
            prop_ticks: ROTOR_DEGRADE_INTERVAL,
            power: 0,
            speed: 0,
            phase: 0,
            efficiency: MAX_EFFICIENCY,
            ray_trace: [20; RAYTRACE_SAMPLES],
            tracer: 0,
            con_mask: -1,
        }
    }

    /// Returns the current terminal voltage.
    pub fn voltage(&self) -> f64 {
        self.cond.voltage
    }

    /// Mounts and deploys a turbine rotor into slot 0.
    pub fn deploy_rotor(&mut self, item: ItemStack) {
        self.rotor = Some(item);
        self.has_blades = true;
        self.prop_ticks = ROTOR_DEGRADE_INTERVAL;
    }

    /// Tears down and dismounts active blades.
    pub fn teardown_blades(&mut self) {
        self.has_blades = false;
        self.power = 0;
        self.speed = 0;
    }

    /// Sets the external redstone brake signal.
    pub fn set_redstone_brake(&mut self, brake: bool) {
        self.redstone_brake = brake;
        self.update_brakes();
    }

    /// Evaluates brake engagement based on redstone signal and voltage thresholds.
    pub fn update_brakes(&mut self) {
        if self.redstone_brake || (!self.has_brakes && self.cond.voltage > TURBINE_BRAKE_VOLTAGE) {
            self.has_brakes = true;
        } else if self.has_brakes
            && self.cond.voltage < TURBINE_BRAKE_RELEASE_VOLTAGE
            && !self.redstone_brake
        {
            self.has_brakes = false;
        }
    }

    /// Records an air clearance raytrace sample (0..=20 blocks unobstructed).
    pub fn record_raytrace(&mut self, clear_blocks: u8) {
        let sample = clear_blocks.min(20);
        let old_sample = self.ray_trace[self.tracer];
        self.efficiency =
            (self.efficiency - (old_sample as i32) + (sample as i32)).clamp(0, MAX_EFFICIENCY);
        self.ray_trace[self.tracer] = sample;
        self.tracer = (self.tracer + 1) % RAYTRACE_SAMPLES;
    }

    /// Advances one game tick given the current wind speed.
    ///
    /// - Checks redstone and overvoltage brakes.
    /// - If brakes are engaged, rotation and power generation are halted.
    /// - Degrades rotor durability by 1 every 6600 ticks.
    /// - Generates power: $P / 5\text{ W}$ applied as current $I = (P / 5) / \max(V, 1.0)$ up to 130V.
    pub fn tick(&mut self, wind_speed: i32) -> f64 {
        self.update_brakes();

        if !self.has_blades || self.rotor.is_none() {
            self.power = 0;
            self.speed = 0;
            return 0.0;
        }

        // Calculate power from wind speed and efficiency
        match self.windmill_type {
            1 => {
                // Vertical axis
                self.power = (2 * wind_speed * self.efficiency) / 2240;
                self.speed = (self.power * self.power) / 20000;
            }
            _ => {
                // Horizontal axis
                self.power = (wind_speed * self.efficiency) / 5780;
                self.speed = (self.power * self.power) / 5000;
            }
        }

        // Durability degradation
        if self.prop_ticks == 0 {
            if let Some(rotor) = self.rotor.as_mut() {
                rotor.damage += 1;
                let max_durability = ROTOR_DEFAULT_MAX_DURABILITY;
                if rotor.damage > max_durability {
                    // Rotor breaks -> replaced by wooden sail
                    self.rotor = Some(ItemStack::new(WOOD_SAIL_ID, 1));
                    self.teardown_blades();
                    return 0.0;
                }
            }
            self.prop_ticks = ROTOR_DEGRADE_INTERVAL;
        } else {
            self.prop_ticks -= 1;
        }

        // Brakes halt generation
        if self.has_brakes {
            return 0.0;
        }

        // Safe voltage ceiling
        if self.cond.voltage > TURBINE_MAX_SAFE_VOLTAGE {
            return 0.0;
        }

        // Apply power (power / 5 W)
        let power_w = (self.power as f64) / 5.0;
        if power_w > 0.0 {
            let current = power_w / self.cond.voltage.max(1.0);
            self.cond.add_charge(current);
            power_w
        } else {
            0.0
        }
    }

    /// Equalizes charge and steps RLC iteration against an adjacent conductor.
    pub fn conduct(&mut self, other: &mut BluePowerConductor) -> f64 {
        self.cond.step_iterate(other)
    }

    /// Deserializes state from NBT string map for chunk reload.
    pub fn from_nbt(nbt: &HashMap<String, String>) -> Self {
        let mut turbine = Self::new();
        let get_f = |k: &str| nbt.get(k).and_then(|v| v.parse::<f64>().ok());
        let get_i = |k: &str| nbt.get(k).and_then(|v| v.parse::<i32>().ok());
        let get_u = |k: &str| nbt.get(k).and_then(|v| v.parse::<u32>().ok());

        turbine.cond.charge = get_f("charge").unwrap_or(0.0).max(0.0);
        turbine.cond.current = get_f("current").unwrap_or(0.0);
        turbine.cond.resistance = get_f("resistance").unwrap_or(BLUTRICITY_CONDUCTOR_RESISTANCE);
        turbine.cond.capacitance = get_f("capacitance")
            .unwrap_or(TURBINE_CAPACITANCE)
            .max(0.01);
        turbine.cond.voltage = turbine.cond.charge / turbine.cond.capacitance;

        turbine.has_blades = nbt
            .get("has_blades")
            .is_some_and(|v| v == "1" || v == "true");
        turbine.has_brakes = nbt
            .get("has_brakes")
            .is_some_and(|v| v == "1" || v == "true");
        turbine.redstone_brake = nbt
            .get("redstone_brake")
            .is_some_and(|v| v == "1" || v == "true");

        if let Some(wt) = nbt.get("windmill_type").and_then(|v| v.parse::<u8>().ok()) {
            turbine.windmill_type = wt;
        }
        if let Some(pt) = get_u("prop_ticks") {
            turbine.prop_ticks = pt;
        }
        if let Some(p) = get_i("power") {
            turbine.power = p;
        }
        if let Some(s) = get_i("speed") {
            turbine.speed = s;
        }
        if let Some(eff) = get_i("efficiency") {
            turbine.efficiency = eff;
        }
        if let Some(mask) = get_i("con_mask") {
            turbine.con_mask = mask;
        }

        if let Some(rotor_id) = nbt.get("rotor_id") {
            let count = nbt
                .get("rotor_count")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(1);
            let damage = nbt
                .get("rotor_damage")
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(0);
            let mut stack = ItemStack::new(rotor_id, count);
            stack.damage = damage;
            turbine.rotor = Some(stack);
        }

        turbine
    }

    /// Serializes state into NBT string map for chunk reload persistence.
    pub fn to_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("charge".into(), self.cond.charge.to_string());
        nbt.insert("current".into(), self.cond.current.to_string());
        nbt.insert("resistance".into(), self.cond.resistance.to_string());
        nbt.insert("capacitance".into(), self.cond.capacitance.to_string());

        nbt.insert(
            "has_blades".into(),
            if self.has_blades { "1" } else { "0" }.into(),
        );
        nbt.insert(
            "has_brakes".into(),
            if self.has_brakes { "1" } else { "0" }.into(),
        );
        nbt.insert(
            "redstone_brake".into(),
            if self.redstone_brake { "1" } else { "0" }.into(),
        );

        nbt.insert("windmill_type".into(), self.windmill_type.to_string());
        nbt.insert("prop_ticks".into(), self.prop_ticks.to_string());
        nbt.insert("power".into(), self.power.to_string());
        nbt.insert("speed".into(), self.speed.to_string());
        nbt.insert("efficiency".into(), self.efficiency.to_string());
        nbt.insert("con_mask".into(), self.con_mask.to_string());

        if let Some(rotor) = &self.rotor {
            nbt.insert("rotor_id".into(), rotor.item_id.clone());
            nbt.insert("rotor_count".into(), rotor.count.to_string());
            nbt.insert("rotor_damage".into(), rotor.damage.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_turbine_unobstructed_generation() {
        let mut turbine = TileWindTurbine::new();
        assert_eq!(turbine.voltage(), 0.0);

        // Without blades, no power generated
        let gen0 = turbine.tick(10000);
        assert_eq!(gen0, 0.0);

        // Mount rotor
        turbine.deploy_rotor(ItemStack::new(WINDMILL_WOOD_ID, 1));
        assert!(turbine.has_blades);

        // Tick with wind speed 10,000 and full efficiency (5780)
        let power_w = turbine.tick(10000);
        assert!(power_w > 0.0);
        assert!(turbine.voltage() > 0.0);
        assert!(turbine.power > 0);
        assert!(turbine.speed > 0);
    }

    #[test]
    fn test_wind_turbine_redstone_brake() {
        let mut turbine = TileWindTurbine::new();
        turbine.deploy_rotor(ItemStack::new(WINDMILL_WOOD_ID, 1));
        turbine.cond.voltage = 50.0;
        turbine.cond.charge = 200.0;

        // Apply redstone brake
        turbine.set_redstone_brake(true);
        assert!(turbine.has_brakes);

        // Generation should immediately halt
        let gen_braked = turbine.tick(1000);
        assert_eq!(gen_braked, 0.0);

        // Release redstone brake
        turbine.set_redstone_brake(false);
        assert!(!turbine.has_brakes);
        let gen_released = turbine.tick(1000);
        assert!(gen_released > 0.0);
    }

    #[test]
    fn test_wind_turbine_overvoltage_brake() {
        let mut turbine = TileWindTurbine::new();
        turbine.deploy_rotor(ItemStack::new(WINDMILL_WOOD_ID, 1));

        // Set voltage above 110V
        turbine.cond.voltage = 111.0;
        turbine.cond.charge = 111.0 * 4.0;

        let gen = turbine.tick(10000);
        assert_eq!(gen, 0.0);
        assert!(turbine.has_brakes);

        // Discharge below 100V
        turbine.cond.voltage = 99.0;
        turbine.cond.charge = 99.0 * 4.0;
        let gen2 = turbine.tick(10000);
        assert!(gen2 > 0.0);
        assert!(!turbine.has_brakes);
    }

    #[test]
    fn test_wind_turbine_rotor_degradation_and_breakage() {
        let mut turbine = TileWindTurbine::new();
        let mut rotor = ItemStack::new(WINDMILL_WOOD_ID, 1);
        rotor.damage = 1500;
        turbine.deploy_rotor(rotor);

        turbine.prop_ticks = 0;
        turbine.tick(10000);

        // Rotor should have broken into wood_sail
        assert!(!turbine.has_blades);
        assert!(turbine.rotor.is_some());
        assert_eq!(turbine.rotor.as_ref().unwrap().item_id, WOOD_SAIL_ID);
    }

    #[test]
    fn test_wind_turbine_raytrace_clearance() {
        let mut turbine = TileWindTurbine::new();
        assert_eq!(turbine.efficiency, MAX_EFFICIENCY);

        // Sample with obstruction (0 clear blocks)
        for _ in 0..RAYTRACE_SAMPLES {
            turbine.record_raytrace(0);
        }
        assert_eq!(turbine.efficiency, 0);

        // Half clearance (10 blocks)
        for _ in 0..RAYTRACE_SAMPLES {
            turbine.record_raytrace(10);
        }
        assert_eq!(turbine.efficiency, 289 * 10);
    }

    #[test]
    fn test_wind_turbine_chunk_reload_persistence() {
        let mut turbine = TileWindTurbine::new();
        let mut rotor = ItemStack::new(WINDMILL_WOOD_ID, 1);
        rotor.damage = 42;
        turbine.deploy_rotor(rotor);
        turbine.power = 1500;
        turbine.speed = 450;
        turbine.efficiency = 5000;
        turbine.set_redstone_brake(true);
        turbine.con_mask = 255;
        turbine.cond.add_charge(80.0);

        let mut nbt = HashMap::new();
        turbine.to_nbt(&mut nbt);

        let restored = TileWindTurbine::from_nbt(&nbt);
        assert!(restored.has_blades);
        assert!(restored.has_brakes);
        assert!(restored.redstone_brake);
        assert_eq!(restored.power, 1500);
        assert_eq!(restored.speed, 450);
        assert_eq!(restored.efficiency, 5000);
        assert_eq!(restored.con_mask, 255);
        assert!((restored.voltage() - turbine.voltage()).abs() < 1e-6);

        let restored_rotor = restored.rotor.unwrap();
        assert_eq!(restored_rotor.item_id, WINDMILL_WOOD_ID);
        assert_eq!(restored_rotor.damage, 42);
    }
}
