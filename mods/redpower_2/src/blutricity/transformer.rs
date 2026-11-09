//! Blutricity Transformer (`redpower_2:transformer`): port of `TileTransformer`.
//!
//! Features:
//! - Steps voltage between 100V nominal low voltage and 10,000V high voltage (100x ratio).
//! - Conserves power across boundaries: $V_{\text{high}} = 100 \times V_{\text{low}}$, $I_{\text{high}} = I_{\text{low}} / 100$.
//! - Two independent conductor terminals:
//!   - Primary (`cond`): Low voltage ($C = 4.0\text{ F}$, $R = 0.01\ \Omega$).
//!   - Secondary (`cond2`): High voltage ($C = 0.04\text{ F}$, $R = 1.0\ \Omega$).
//! - Sided connection masks based on 6-axis block rotation.
//! - Chunk reload persistence for both primary and secondary conductor states.

use crate::BluePowerConductor;
use crate::{BLUTRICITY_CONDUCTOR_RESISTANCE, BLUTRICITY_NOMINAL_VOLTAGE};
use std::collections::HashMap;

/// Block ID in `redpower_2`.
pub const TRANSFORMER_BLOCK_ID: &str = "redpower_2:transformer";
/// Tile entity name in `GameRegistry`.
pub const TRANSFORMER_TE_NAME: &str = "RPTransformer";
/// Metadata index in `BlockMachinePanel`.
pub const TRANSFORMER_METADATA: u8 = 4;
/// Voltage step ratio ($100\times$).
pub const VOLTAGE_RATIO: f64 = 100.0;
/// Nominal low voltage (100V).
pub const LOW_NOMINAL_VOLTAGE: f64 = BLUTRICITY_NOMINAL_VOLTAGE;
/// Nominal high voltage (10,000V).
pub const HIGH_NOMINAL_VOLTAGE: f64 = LOW_NOMINAL_VOLTAGE * VOLTAGE_RATIO;
/// Primary capacitance ($C = 4.0\text{ F}$).
pub const PRIMARY_CAPACITANCE: f64 = 4.0;
/// Secondary capacitance ($C = 0.04\text{ F}$, $1/C = 25.0$).
pub const SECONDARY_CAPACITANCE: f64 = 0.04;
/// Secondary internal resistance ($1.0\ \Omega$).
pub const SECONDARY_RESISTANCE: f64 = 1.0;

/// State machine for `TileTransformer`.
#[derive(Debug, Clone, PartialEq)]
pub struct TileTransformer {
    /// Low voltage primary conductor (connectClass 64).
    pub cond: BluePowerConductor,
    /// High voltage secondary conductor (connectClass 68).
    pub cond2: BluePowerConductor,
    /// Orientation and rotation byte.
    pub rotation: u8,
    /// Primary connection mask.
    pub con_mask1: i32,
    /// Secondary connection mask.
    pub con_mask2: i32,
}

impl Default for TileTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl TileTransformer {
    /// Creates a new `TileTransformer` with primary and secondary conductors.
    pub fn new() -> Self {
        let mut cond2 = BluePowerConductor::new(SECONDARY_CAPACITANCE);
        cond2.resistance = SECONDARY_RESISTANCE;

        Self {
            cond: BluePowerConductor::new(PRIMARY_CAPACITANCE),
            cond2,
            rotation: 0,
            con_mask1: -1,
            con_mask2: -1,
        }
    }

    /// Returns the primary low-side voltage.
    pub fn low_voltage(&self) -> f64 {
        self.cond.voltage
    }

    /// Returns the secondary high-side voltage.
    pub fn high_voltage(&self) -> f64 {
        self.cond2.voltage
    }

    /// Transfers charge between primary and secondary terminals across the 100x transformer coupling.
    ///
    /// Following `computeVoltage` and `applyCurrent` in `TileTransformer.java`:
    /// `Vcap2 = cond.getVoltage() * 100.0`.
    /// High voltage terminal directly couples with primary low voltage scaled by 100x.
    pub fn step_transform(&mut self) {
        self.cond2.voltage = self.cond.voltage * VOLTAGE_RATIO;
        self.cond2.charge = self.cond2.voltage * self.cond2.capacitance;
    }

    /// Advances one tick of transformer operation.
    pub fn tick(&mut self) {
        self.step_transform();
    }

    /// Steps iteration between low-voltage primary conductor and a connected node.
    pub fn conduct_low(&mut self, other: &mut BluePowerConductor) -> f64 {
        let flow = self.cond.step_iterate(other);
        self.step_transform();
        flow
    }

    /// Steps iteration between high-voltage secondary conductor and a connected node.
    ///
    /// Reflected current transfers back to primary side: $I_1 = I_2 \times 100.0$.
    pub fn conduct_high(&mut self, other: &mut BluePowerConductor) -> f64 {
        let flow = self.cond2.step_iterate(other);
        self.cond.add_charge(-flow * VOLTAGE_RATIO);
        self.step_transform();
        flow
    }

    /// Restores transformer electrical state from NBT string map.
    pub fn from_nbt(nbt: &HashMap<String, String>) -> Self {
        let mut tf = Self::new();
        let get = |k: &str| nbt.get(k).and_then(|v| v.parse::<f64>().ok());

        // Primary conductor
        tf.cond.charge = get("c1_charge").unwrap_or(0.0).max(0.0);
        tf.cond.current = get("c1_current").unwrap_or(0.0);
        tf.cond.resistance = get("c1_resistance").unwrap_or(BLUTRICITY_CONDUCTOR_RESISTANCE);
        tf.cond.capacitance = get("c1_capacitance")
            .unwrap_or(PRIMARY_CAPACITANCE)
            .max(0.01);
        tf.cond.voltage = tf.cond.charge / tf.cond.capacitance;

        // Secondary conductor
        tf.cond2.charge = get("c2_charge").unwrap_or(0.0).max(0.0);
        tf.cond2.current = get("c2_current").unwrap_or(0.0);
        tf.cond2.resistance = get("c2_resistance").unwrap_or(SECONDARY_RESISTANCE);
        tf.cond2.capacitance = get("c2_capacitance")
            .unwrap_or(SECONDARY_CAPACITANCE)
            .max(0.001);
        tf.cond2.voltage = tf.cond2.charge / tf.cond2.capacitance;

        if let Some(rot) = nbt.get("rotation").and_then(|v| v.parse::<u8>().ok()) {
            tf.rotation = rot;
        }
        if let Some(m1) = nbt.get("con_mask1").and_then(|v| v.parse::<i32>().ok()) {
            tf.con_mask1 = m1;
        }
        if let Some(m2) = nbt.get("con_mask2").and_then(|v| v.parse::<i32>().ok()) {
            tf.con_mask2 = m2;
        }
        tf
    }

    /// Serializes transformer electrical state into NBT string map.
    pub fn to_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("c1_charge".into(), self.cond.charge.to_string());
        nbt.insert("c1_current".into(), self.cond.current.to_string());
        nbt.insert("c1_resistance".into(), self.cond.resistance.to_string());
        nbt.insert("c1_capacitance".into(), self.cond.capacitance.to_string());

        nbt.insert("c2_charge".into(), self.cond2.charge.to_string());
        nbt.insert("c2_current".into(), self.cond2.current.to_string());
        nbt.insert("c2_resistance".into(), self.cond2.resistance.to_string());
        nbt.insert("c2_capacitance".into(), self.cond2.capacitance.to_string());

        nbt.insert("rotation".into(), self.rotation.to_string());
        nbt.insert("con_mask1".into(), self.con_mask1.to_string());
        nbt.insert("con_mask2".into(), self.con_mask2.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_voltage_step_up() {
        let mut tf = TileTransformer::new();
        // Charge low side to 100V: Q = 100 * 4.0 = 400C
        tf.cond.add_charge(400.0);
        assert_eq!(tf.low_voltage(), 100.0);

        // Step transform
        tf.tick();

        // Secondary should step up to 100x ratio (10,000V)
        assert_eq!(tf.high_voltage(), 10000.0);
        let ratio = tf.high_voltage() / tf.low_voltage();
        assert!((ratio - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_transformer_voltage_step_down() {
        let mut tf = TileTransformer::new();
        // High voltage side receives current from high-voltage wire:
        let mut high_wire = BluePowerConductor::new(0.04);
        high_wire.add_charge(400.0); // 10,000V on external high wire

        tf.conduct_high(&mut high_wire);

        // Primary low voltage side should step down accordingly (100V)
        assert!(tf.low_voltage() > 0.0);
        let ratio = tf.high_voltage() / tf.low_voltage();
        assert!((ratio - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_transformer_conduction() {
        let mut tf = TileTransformer::new();
        tf.cond.add_charge(100.0); // 25V

        let mut low_wire = BluePowerConductor::new(0.25);
        let flow1 = tf.conduct_low(&mut low_wire);
        assert!(flow1 > 0.0);
        assert!(low_wire.voltage > 0.0);

        tf.cond2.add_charge(40.0); // 1000V
        let mut high_wire = BluePowerConductor::new(0.04);
        let flow2 = tf.conduct_high(&mut high_wire);
        assert!(flow2 > 0.0);
        assert!(high_wire.voltage > 0.0);
    }

    #[test]
    fn test_transformer_nbt_persistence() {
        let mut tf = TileTransformer::new();
        tf.cond.add_charge(200.0);
        tf.cond2.add_charge(50.0);
        tf.rotation = 5;
        tf.con_mask1 = 15;
        tf.con_mask2 = 240;

        let mut nbt = HashMap::new();
        tf.to_nbt(&mut nbt);

        let restored = TileTransformer::from_nbt(&nbt);
        assert_eq!(restored.rotation, 5);
        assert_eq!(restored.con_mask1, 15);
        assert_eq!(restored.con_mask2, 240);
        assert!((restored.low_voltage() - tf.low_voltage()).abs() < 1e-6);
        assert!((restored.high_voltage() - tf.high_voltage()).abs() < 1e-6);
    }
}
