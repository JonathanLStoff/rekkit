//! Shared foundation library (`rekkit_core`) for Rekkit mods targeting the Horizon engine.
//!
//! Implements the cross-mod building blocks specified in `planning/mods/rekkit_core.md`:
//! - [`energy`]: IC2 EU (voltage tiers, over-voltage explosion checks, cable loss), BuildCraft/TE MJ
//!   (thermal stages, 5% conduit loss), RedPower 2 Blutricity ($V, I, Q$), AE1 energy, and Power
//!   Converters cross-mod conversion ratios.
//! - [`inventory`]: [`ItemStack`], fixed-size [`Inventory`], sided slot insertion/extraction, and
//!   redstone comparator fill signal calculation (`0..=15`).
//! - [`fluid`]: [`FluidStack`] and millibucket [`FluidTank`] with NBT persistence helpers.
//! - [`side`]: 6-face [`Direction`], 6-bit [`ConnectionMask`] (`API 32`), and Thermal Expansion
//!   6-color [`SideMode`] / [`SideConfig`].
//! - [`grid`]: [`BlockPos`], [`UnionFind`], and [`GridGraph`] (connected components + RedPower 2
//!   Dijkstra pneumatic tube routing with color tags and restrictor weights).
//! - [`frequency`]: 16-color [`DyeColor`], 16-channel [`BundledSignal`], 4,096-channel
//!   [`FrequencyKey`], and [`ChannelOwner`].
//! - [`multiblock`]: [`MultiblockShape`] validation for solid and hollow 3D structures.
//! - [`hooks`]: Horizon `NATIVE_HOOKS_API = 40` mirror structs and [`HorizonNativeMod`] trait.
//! - [`util`]: Deterministic [`XorShift64`] RNG and [`TickTimer`].

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Current Horizon `NATIVE_HOOKS_API` version targeted by Rekkit mods.
pub const NATIVE_HOOKS_API: u32 = 40;

/// Pinned Horizon engine revision for native mixin compatibility.
pub const HORIZON_REVISION: &str = "ed102e0592eded5585c610c205488f771f52ac40";

/// Pinned `horizon-mixins` tool revision (`native_mods::MIXINS_REV`).
pub const MIXINS_REVISION: &str = "3f99a5c60dfa311cc365524c44ae2fd799bfa1de";

// ============================================================================
// 1. Energy Systems & Cross-Mod Conversions
// ============================================================================

pub mod energy {
    use super::*;

    /// Standard NBT keys stored on `ModBlockEntity.nbt` for cross-mod energy and fluid exchange.
    pub mod nbt_keys {
        pub const EU_STORED: &str = "eu_stored";
        pub const EU_CAPACITY: &str = "eu_capacity";
        pub const EU_MAX_TIER: &str = "eu_max_tier";
        pub const MJ_STORED: &str = "mj_stored";
        pub const MJ_CAPACITY: &str = "mj_capacity";
        pub const BT_VOLTAGE: &str = "bt_voltage";
        pub const BT_CHARGE: &str = "bt_charge";
        pub const AE_STORED: &str = "ae_stored";
        pub const AE_CAPACITY: &str = "ae_capacity";
        pub const EMC_STORED: &str = "emc_stored";
        pub const EMC_CAPACITY: &str = "emc_capacity";
        pub const FLUID_ID: &str = "fluid_id";
        pub const FLUID_MB: &str = "fluid_mb";
        pub const FLUID_CAPACITY_MB: &str = "fluid_capacity_mb";
    }

    /// IndustrialCraft 2 voltage tiers (`LV`, `MV`, `HV`, `EV`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub enum EuTier {
        /// Low Voltage: up to 32 EU/packet.
        LV = 1,
        /// Medium Voltage: up to 128 EU/packet.
        MV = 2,
        /// High Voltage: up to 512 EU/packet.
        HV = 3,
        /// Extreme Voltage: up to 2048 EU/packet.
        EV = 4,
    }

    impl EuTier {
        /// Maximum safe packet voltage (EU/packet) for this tier.
        pub const fn max_voltage(self) -> u32 {
            match self {
                Self::LV => 32,
                Self::MV => 128,
                Self::HV => 512,
                Self::EV => 2048,
            }
        }

        /// Classify a packet voltage into the lowest tier capable of handling it safely.
        pub const fn from_voltage(voltage: u32) -> Self {
            if voltage <= 32 {
                Self::LV
            } else if voltage <= 128 {
                Self::MV
            } else if voltage <= 512 {
                Self::HV
            } else {
                Self::EV
            }
        }
    }

    /// Error returned when a machine receives an EU packet exceeding its rated [`EuTier`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OverVoltageError {
        pub packet_voltage: u32,
        pub max_safe_voltage: u32,
    }

    /// A discrete IndustrialCraft 2 energy packet.
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct EuPacket {
        pub amount: f64,
        pub voltage: u32,
    }

    impl EuPacket {
        pub fn new(amount: f64, voltage: u32) -> Self {
            Self { amount, voltage }
        }

        /// Compute the remaining packet energy after traversing `distance_blocks` of `cable`.
        /// Returns `Err(OverVoltageError)` if the packet's voltage melts the cable.
        pub fn after_cable_loss(
            self,
            cable: EuCableKind,
            distance_blocks: u32,
        ) -> Result<f64, OverVoltageError> {
            let max_v = cable.max_voltage();
            if self.voltage > max_v {
                return Err(OverVoltageError {
                    packet_voltage: self.voltage,
                    max_safe_voltage: max_v,
                });
            }
            let loss = (distance_blocks as f64) * cable.loss_per_block();
            Ok((self.amount - loss).max(0.0))
        }
    }

    /// Canonical IC2 cable types with exact voltage ratings and distance loss rates.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum EuCableKind {
        /// Ultra-low-voltage Tin Cable: 32 EU/p, 0.025 EU/block (1 EU per 40 blocks).
        Tin,
        /// Insulated Copper Cable: 32 EU/p, 0.20 EU/block (1 EU per 5 blocks).
        Copper,
        /// 2x Insulated Gold Cable: 128 EU/p, 0.40 EU/block (1 EU per 2.5 blocks).
        Gold,
        /// 3x Insulated HV Iron Cable: 512 EU/p, 0.80 EU/block (1 EU per 1.25 blocks).
        HvIron,
        /// Glass Fibre Cable: 512 EU/p, 0.025 EU/block (1 EU per 40 blocks).
        GlassFibre,
        /// EV-rated HV Cable: 2048 EU/p, 0.80 EU/block.
        EvIron,
    }

    impl EuCableKind {
        pub const fn max_voltage(self) -> u32 {
            match self {
                Self::Tin | Self::Copper => 32,
                Self::Gold => 128,
                Self::HvIron | Self::GlassFibre => 512,
                Self::EvIron => 2048,
            }
        }

        pub const fn loss_per_block(self) -> f64 {
            match self {
                Self::Tin | Self::GlassFibre => 0.025,
                Self::Copper => 0.20,
                Self::Gold => 0.40,
                Self::HvIron | Self::EvIron => 0.80,
            }
        }
    }

    /// EU energy buffer for IC2 machines, generators, and storage blocks (BatBox, MFE, MFSU).
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct EuStorage {
        pub stored: f64,
        pub capacity: f64,
        pub max_tier: EuTier,
    }

    impl EuStorage {
        pub fn new(capacity: f64, max_tier: EuTier) -> Self {
            Self {
                stored: 0.0,
                capacity,
                max_tier,
            }
        }

        /// Inject an [`EuPacket`] into this buffer.
        /// Fails with [`OverVoltageError`] if `packet.voltage > self.max_tier.max_voltage()`.
        /// Returns the amount of EU accepted.
        pub fn receive_packet(&mut self, packet: EuPacket) -> Result<f64, OverVoltageError> {
            let max_v = self.max_tier.max_voltage();
            if packet.voltage > max_v {
                return Err(OverVoltageError {
                    packet_voltage: packet.voltage,
                    max_safe_voltage: max_v,
                });
            }
            let space = (self.capacity - self.stored).max(0.0);
            let accepted = packet.amount.clamp(0.0, space);
            self.stored += accepted;
            Ok(accepted)
        }

        /// Consume `amount` EU if available.
        pub fn consume(&mut self, amount: f64) -> bool {
            if self.stored >= amount {
                self.stored -= amount;
                true
            } else {
                false
            }
        }

        pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
            nbt.insert(nbt_keys::EU_STORED.into(), format!("{:.2}", self.stored));
            nbt.insert(
                nbt_keys::EU_CAPACITY.into(),
                format!("{:.2}", self.capacity),
            );
            nbt.insert(
                nbt_keys::EU_MAX_TIER.into(),
                (self.max_tier as u8).to_string(),
            );
        }
    }

    /// BuildCraft engine thermal stages (`Blue` -> `Green` -> `Yellow` -> `Red` -> `Overheated`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum EngineHeatStage {
        Blue,
        Green,
        Yellow,
        Red,
        Overheated,
    }

    impl EngineHeatStage {
        pub fn from_ratio(ratio: f64) -> Self {
            if ratio < 0.25 {
                Self::Blue
            } else if ratio < 0.50 {
                Self::Green
            } else if ratio < 0.75 {
                Self::Yellow
            } else if ratio < 1.0 {
                Self::Red
            } else {
                Self::Overheated
            }
        }
    }

    /// Minecraft Joules (`MJ`) energy buffer for BuildCraft, Thermal Expansion, Forestry, and Railcraft.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct MjBuffer {
        pub stored_mj: f64,
        pub capacity_mj: f64,
        pub max_in_mj: f64,
        pub max_out_mj: f64,
    }

    impl MjBuffer {
        /// Thermal Expansion Redstone Energy Conduit fixed 5% transmission loss factor.
        pub const CONDUIT_EFFICIENCY: f64 = 0.95;

        pub fn new(capacity_mj: f64, max_in_mj: f64, max_out_mj: f64) -> Self {
            Self {
                stored_mj: 0.0,
                capacity_mj,
                max_in_mj,
                max_out_mj,
            }
        }

        pub fn receive(&mut self, amount_mj: f64) -> f64 {
            let space = (self.capacity_mj - self.stored_mj).max(0.0);
            let accepted = amount_mj.min(self.max_in_mj).clamp(0.0, space);
            self.stored_mj += accepted;
            accepted
        }

        pub fn extract(&mut self, max_extract_mj: f64) -> f64 {
            let extracted = max_extract_mj
                .min(self.max_out_mj)
                .clamp(0.0, self.stored_mj);
            self.stored_mj -= extracted;
            extracted
        }

        /// Transmit MJ through a Thermal Expansion Redstone Energy Conduit (fixed 5% loss).
        pub fn transmit_via_conduit(&mut self, dest: &mut MjBuffer, max_mj: f64) -> f64 {
            let raw = self.extract(max_mj);
            let delivered = raw * Self::CONDUIT_EFFICIENCY;
            let accepted = dest.receive(delivered);
            // Refund any unaccepted portion back to source (before loss)
            if accepted < delivered && delivered > 0.0 {
                let unused_raw = (delivered - accepted) / Self::CONDUIT_EFFICIENCY;
                self.stored_mj = (self.stored_mj + unused_raw).min(self.capacity_mj);
            }
            accepted
        }
    }

    /// RedPower 2 Blutricity electrical node ($V$ in Volts, $Q$ in Coulombs/Charge).
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BlutricityNode {
        pub voltage: f64,
        pub charge: f64,
        pub capacitance: f64,
    }

    impl BlutricityNode {
        pub const NOMINAL_VOLTAGE: f64 = 100.0;
        pub const MIN_OPERATING_VOLTAGE: f64 = 60.0;
        pub const MAX_SAFE_VOLTAGE: f64 = 120.0;

        pub fn new(capacitance: f64) -> Self {
            Self {
                voltage: 0.0,
                charge: 0.0,
                capacitance: capacitance.max(1.0),
            }
        }

        pub fn is_operational(&self) -> bool {
            self.voltage >= Self::MIN_OPERATING_VOLTAGE
        }

        pub fn add_charge(&mut self, delta_q: f64) {
            self.charge = (self.charge + delta_q).max(0.0);
            self.voltage = self.charge / self.capacitance;
        }

        /// Equalize charge between two connected Blutricity nodes across resistance `r_ohms` over `dt`.
        /// Uses Ohm's Law: $I = (V_a - V_b) / R$.
        pub fn step_ohm_transfer(&mut self, other: &mut Self, r_ohms: f64, dt: f64) -> f64 {
            let r = r_ohms.max(0.001);
            let current = (self.voltage - other.voltage) / r;
            let delta_q = current * dt;
            self.add_charge(-delta_q);
            other.add_charge(delta_q);
            current
        }
    }

    /// Applied Energistics (`AE1`) network energy buffer.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AeEnergyBuffer {
        pub stored_ae: f64,
        pub capacity_ae: f64,
        pub idle_drain_per_tick: f64,
        pub powered: bool,
    }

    impl AeEnergyBuffer {
        pub fn new(capacity_ae: f64, idle_drain_per_tick: f64) -> Self {
            Self {
                stored_ae: 0.0,
                capacity_ae,
                idle_drain_per_tick,
                powered: false,
            }
        }

        pub fn tick_idle(&mut self) -> bool {
            if self.stored_ae >= self.idle_drain_per_tick {
                self.stored_ae -= self.idle_drain_per_tick;
                self.powered = true;
            } else {
                self.stored_ae = 0.0;
                self.powered = false;
            }
            self.powered
        }
    }

    /// Energy units supported by Rekkit's cross-mod `Power Converters` / `AE1` bridge.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum EnergyUnit {
        /// IndustrialCraft 2 Energy Unit (`1 MJ = 2.5 EU`).
        EU,
        /// BuildCraft / Thermal Expansion / Forestry / Railcraft Minecraft Joule (`1 MJ = 2.5 EU`).
        MJ,
        /// Applied Energistics Unit (`1 EU = 2 AE`, `1 MJ = 5 AE`).
        AE,
        /// Redstone Flux (`1 MJ = 10 RF`).
        RF,
        /// RedPower 2 Blutricity Watt-tick (`1 EU = 2.5 W`).
        BlutricityWatt,
    }

    /// Convert an energy value between any two [`EnergyUnit`] standards using canonical ratios:
    /// `1 MJ = 2.5 EU = 5.0 AE = 10.0 RF = 6.25 BlutricityWatt`.
    pub fn convert_energy(amount: f64, from: EnergyUnit, to: EnergyUnit) -> f64 {
        if from == to {
            return amount;
        }
        // Normalize to MJ first
        let mj = match from {
            EnergyUnit::MJ => amount,
            EnergyUnit::EU => amount / 2.5,
            EnergyUnit::AE => amount / 5.0,
            EnergyUnit::RF => amount / 10.0,
            EnergyUnit::BlutricityWatt => amount / 6.25,
        };
        match to {
            EnergyUnit::MJ => mj,
            EnergyUnit::EU => mj * 2.5,
            EnergyUnit::AE => mj * 5.0,
            EnergyUnit::RF => mj * 10.0,
            EnergyUnit::BlutricityWatt => mj * 6.25,
        }
    }
}

// ============================================================================
// 2. Item Stacks, Inventories & Comparator Math
// ============================================================================

pub mod inventory {
    use super::*;

    /// Inventory item stack (mirrors `horizon_client::items::ItemStack`).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ItemStack {
        pub item_id: String,
        pub count: u32,
        #[serde(default)]
        pub damage: u16,
        #[serde(default = "ItemStack::default_max_stack")]
        pub max_stack_size: u32,
    }

    impl ItemStack {
        pub const fn default_max_stack() -> u32 {
            64
        }

        pub fn new(item_id: impl Into<String>, count: u32) -> Self {
            Self {
                item_id: item_id.into(),
                count,
                damage: 0,
                max_stack_size: 64,
            }
        }

        pub fn with_max_stack(mut self, max_stack_size: u32) -> Self {
            self.max_stack_size = max_stack_size.clamp(1, 99);
            self
        }

        pub fn can_merge(&self, other: &Self) -> bool {
            self.item_id == other.item_id
                && self.damage == other.damage
                && self.max_stack_size == other.max_stack_size
        }
    }

    /// Fixed-size slot inventory with sided slot insertion/extraction and comparator signal output.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Inventory {
        pub slots: Vec<Option<ItemStack>>,
    }

    impl Inventory {
        pub fn new(slot_count: usize) -> Self {
            Self {
                slots: vec![None; slot_count],
            }
        }

        pub fn len(&self) -> usize {
            self.slots.len()
        }

        pub fn is_empty(&self) -> bool {
            self.slots.iter().all(Option::is_none)
        }

        /// Insert `stack` across any slot in the inventory, returning any leftover stack.
        pub fn insert_item(&mut self, stack: ItemStack) -> Option<ItemStack> {
            let all: Vec<usize> = (0..self.slots.len()).collect();
            self.insert_into_slots(stack, &all)
        }

        /// Insert `stack` only into the indices listed in `allowed_slots`.
        pub fn insert_into_slots(
            &mut self,
            mut stack: ItemStack,
            allowed_slots: &[usize],
        ) -> Option<ItemStack> {
            if stack.count == 0 {
                return None;
            }
            // 1. Merge into existing matching stacks
            for &idx in allowed_slots {
                if let Some(Some(existing)) = self.slots.get_mut(idx) {
                    if existing.can_merge(&stack) && existing.count < existing.max_stack_size {
                        let space = existing.max_stack_size - existing.count;
                        let transfer = space.min(stack.count);
                        existing.count += transfer;
                        stack.count -= transfer;
                        if stack.count == 0 {
                            return None;
                        }
                    }
                }
            }
            // 2. Place into empty allowed slots
            for &idx in allowed_slots {
                if let Some(slot @ None) = self.slots.get_mut(idx) {
                    let take = stack.count.min(stack.max_stack_size);
                    let mut placed = stack.clone();
                    placed.count = take;
                    *slot = Some(placed);
                    stack.count -= take;
                    if stack.count == 0 {
                        return None;
                    }
                }
            }
            Some(stack)
        }

        /// Extract up to `max_count` items from the first non-empty slot in `allowed_slots`.
        pub fn extract_from_slots(
            &mut self,
            max_count: u32,
            allowed_slots: &[usize],
        ) -> Option<ItemStack> {
            if max_count == 0 {
                return None;
            }
            for &idx in allowed_slots {
                if let Some(slot_opt) = self.slots.get_mut(idx) {
                    if let Some(existing) = slot_opt {
                        let take = max_count.min(existing.count);
                        let mut out = existing.clone();
                        out.count = take;
                        existing.count -= take;
                        if existing.count == 0 {
                            *slot_opt = None;
                        }
                        return Some(out);
                    }
                }
            }
            None
        }

        /// Compute the redstone comparator signal (`0..=15`) for `native_block_signal` (`API 38`).
        pub fn redstone_signal(&self) -> i32 {
            if self.slots.is_empty() {
                return 0;
            }
            let mut sum_ratios = 0.0f64;
            let mut has_any = false;
            for stack in self.slots.iter().flatten() {
                if stack.count > 0 {
                    has_any = true;
                    let max_s = stack.max_stack_size.max(1) as f64;
                    sum_ratios += (stack.count as f64 / max_s).min(1.0);
                }
            }
            if !has_any {
                0
            } else {
                let avg = sum_ratios / (self.slots.len() as f64);
                1 + (avg * 14.0).floor() as i32
            }
        }
    }
}

// ============================================================================
// 3. Fluid Tanks (Millibuckets `mB`)
// ============================================================================

pub mod fluid {
    use super::*;
    use crate::energy::nbt_keys;

    /// A measured volume of fluid in millibuckets (`1,000 mB = 1 Bucket`).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FluidStack {
        pub fluid_id: String,
        pub amount_mb: u32,
    }

    impl FluidStack {
        pub fn new(fluid_id: impl Into<String>, amount_mb: u32) -> Self {
            Self {
                fluid_id: fluid_id.into(),
                amount_mb,
            }
        }
    }

    /// Single-fluid tank measured in millibuckets (`mB`).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FluidTank {
        pub capacity_mb: u32,
        pub contents: Option<FluidStack>,
    }

    impl FluidTank {
        pub fn new(capacity_mb: u32) -> Self {
            Self {
                capacity_mb,
                contents: None,
            }
        }

        pub fn amount_mb(&self) -> u32 {
            self.contents.as_ref().map_or(0, |f| f.amount_mb)
        }

        pub fn fluid_id(&self) -> Option<&str> {
            self.contents.as_ref().map(|f| f.fluid_id.as_str())
        }

        pub fn can_accept(&self, fluid_id: &str) -> bool {
            match &self.contents {
                None => true,
                Some(existing) => {
                    existing.fluid_id == fluid_id && existing.amount_mb < self.capacity_mb
                }
            }
        }

        /// Fill the tank with `incoming`, returning the number of millibuckets accepted.
        /// If `do_fill` is `false`, simulates the fill without mutating state.
        pub fn fill(&mut self, incoming: FluidStack, do_fill: bool) -> u32 {
            if incoming.amount_mb == 0 || !self.can_accept(&incoming.fluid_id) {
                return 0;
            }
            let current = self.amount_mb();
            let space = self.capacity_mb.saturating_sub(current);
            let accepted = incoming.amount_mb.min(space);
            if do_fill && accepted > 0 {
                match &mut self.contents {
                    Some(existing) => existing.amount_mb += accepted,
                    None => {
                        self.contents = Some(FluidStack {
                            fluid_id: incoming.fluid_id,
                            amount_mb: accepted,
                        });
                    }
                }
            }
            accepted
        }

        /// Drain up to `max_drain_mb` millibuckets from the tank.
        /// If `do_drain` is `false`, simulates the drain without mutating state.
        pub fn drain(&mut self, max_drain_mb: u32, do_drain: bool) -> Option<FluidStack> {
            let existing = self.contents.as_mut()?;
            if max_drain_mb == 0 || existing.amount_mb == 0 {
                return None;
            }
            let drained = max_drain_mb.min(existing.amount_mb);
            let out = FluidStack {
                fluid_id: existing.fluid_id.clone(),
                amount_mb: drained,
            };
            if do_drain {
                existing.amount_mb -= drained;
                if existing.amount_mb == 0 {
                    self.contents = None;
                }
            }
            Some(out)
        }

        pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
            nbt.insert(
                nbt_keys::FLUID_CAPACITY_MB.into(),
                self.capacity_mb.to_string(),
            );
            if let Some(f) = &self.contents {
                nbt.insert(nbt_keys::FLUID_ID.into(), f.fluid_id.clone());
                nbt.insert(nbt_keys::FLUID_MB.into(), f.amount_mb.to_string());
            } else {
                nbt.remove(nbt_keys::FLUID_ID);
                nbt.insert(nbt_keys::FLUID_MB.into(), "0".into());
            }
        }

        pub fn read_nbt(default_capacity_mb: u32, nbt: &HashMap<String, String>) -> Self {
            let capacity_mb = nbt
                .get(nbt_keys::FLUID_CAPACITY_MB)
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(default_capacity_mb);
            let amount_mb = nbt
                .get(nbt_keys::FLUID_MB)
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);
            let fluid_id = nbt.get(nbt_keys::FLUID_ID).cloned().unwrap_or_default();
            let contents = if amount_mb > 0 && !fluid_id.is_empty() {
                Some(FluidStack {
                    fluid_id,
                    amount_mb: amount_mb.min(capacity_mb),
                })
            } else {
                None
            };
            Self {
                capacity_mb,
                contents,
            }
        }
    }
}

// ============================================================================
// 4. 6-Face Directions, Connection Bitmasks & CoFH Side Configuration
// ============================================================================

pub mod side {
    use super::*;

    /// Standard 6 block faces ordered `0..6` to match Horizon's `ModContainer.faces: [Vec<usize>; 6]`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[repr(u8)]
    pub enum Direction {
        Down = 0,
        Up = 1,
        North = 2,
        South = 3,
        West = 4,
        East = 5,
    }

    impl Direction {
        pub const ALL: [Self; 6] = [
            Self::Down,
            Self::Up,
            Self::North,
            Self::South,
            Self::West,
            Self::East,
        ];

        pub const fn opposite(self) -> Self {
            match self {
                Self::Down => Self::Up,
                Self::Up => Self::Down,
                Self::North => Self::South,
                Self::South => Self::North,
                Self::West => Self::East,
                Self::East => Self::West,
            }
        }

        pub const fn offset(self) -> [i32; 3] {
            match self {
                Self::Down => [0, -1, 0],
                Self::Up => [0, 1, 0],
                Self::North => [0, 0, -1],
                Self::South => [0, 0, 1],
                Self::West => [-1, 0, 0],
                Self::East => [1, 0, 0],
            }
        }

        pub const fn bit(self) -> u8 {
            1 << (self as u8)
        }
    }

    /// 6-bit neighbor connectivity mask for cables, pipes, tubes, and wires (`native_update_shape` API 32).
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ConnectionMask(pub u8);

    impl ConnectionMask {
        pub const NONE: Self = Self(0);
        pub const ALL: Self = Self(0b0011_1111);

        pub const fn is_connected(self, dir: Direction) -> bool {
            (self.0 & dir.bit()) != 0
        }

        pub fn set(&mut self, dir: Direction, connected: bool) {
            if connected {
                self.0 |= dir.bit();
            } else {
                self.0 &= !dir.bit();
            }
        }

        pub const fn count(self) -> u32 {
            (self.0 & 0b0011_1111).count_ones()
        }
    }

    /// Thermal Expansion / CoFH 6-color side IO role.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum SideMode {
        /// Grey / Open: no connection.
        #[default]
        Disabled,
        /// Blue: Primary input.
        InputPrimary,
        /// Red: Primary output.
        OutputPrimary,
        /// Green: Secondary input.
        InputSecondary,
        /// Yellow: Secondary / byproduct output.
        OutputSecondary,
        /// Orange: Omni-directional (all inputs and outputs).
        Omni,
    }

    impl SideMode {
        /// Cycle to the next side mode when a player clicks a face in the configuration tab.
        pub const fn next(self) -> Self {
            match self {
                Self::Disabled => Self::InputPrimary,
                Self::InputPrimary => Self::OutputPrimary,
                Self::OutputPrimary => Self::InputSecondary,
                Self::InputSecondary => Self::OutputSecondary,
                Self::OutputSecondary => Self::Omni,
                Self::Omni => Self::Disabled,
            }
        }
    }

    /// Per-face IO configuration matrix for configurable machines.
    #[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SideConfig {
        pub sides: [SideMode; 6],
        pub auto_output: bool,
    }

    impl SideConfig {
        pub fn new(auto_output: bool) -> Self {
            Self {
                sides: [SideMode::Omni; 6],
                auto_output,
            }
        }

        pub fn mode(&self, dir: Direction) -> SideMode {
            self.sides[dir as usize]
        }

        pub fn set_mode(&mut self, dir: Direction, mode: SideMode) {
            self.sides[dir as usize] = mode;
        }

        pub fn cycle_mode(&mut self, dir: Direction) -> SideMode {
            let next = self.sides[dir as usize].next();
            self.sides[dir as usize] = next;
            next
        }

        /// Convert this side configuration into Horizon's `ModContainer.faces: [Vec<usize>; 6]`.
        pub fn to_container_faces(
            &self,
            in_primary: &[usize],
            out_primary: &[usize],
            in_secondary: &[usize],
            out_secondary: &[usize],
        ) -> [Vec<usize>; 6] {
            std::array::from_fn(|face_idx| match self.sides[face_idx] {
                SideMode::Disabled => Vec::new(),
                SideMode::InputPrimary => in_primary.to_vec(),
                SideMode::OutputPrimary => out_primary.to_vec(),
                SideMode::InputSecondary => in_secondary.to_vec(),
                SideMode::OutputSecondary => out_secondary.to_vec(),
                SideMode::Omni => {
                    let mut all = Vec::new();
                    for &s in in_primary
                        .iter()
                        .chain(in_secondary)
                        .chain(out_primary)
                        .chain(out_secondary)
                    {
                        if !all.contains(&s) {
                            all.push(s);
                        }
                    }
                    all
                }
            })
        }
    }
}

// ============================================================================
// 5. Connected Grids: Union-Find & Weighted Tube/Pipe Routing
// ============================================================================

pub mod grid {
    use super::*;
    use crate::frequency::DyeColor;
    use crate::side::Direction;

    /// 3D integer block coordinate in world space.
    #[derive(
        Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
    )]
    pub struct BlockPos(pub i32, pub i32, pub i32);

    impl BlockPos {
        pub const fn new(x: i32, y: i32, z: i32) -> Self {
            Self(x, y, z)
        }

        pub const fn to_array(self) -> [i32; 3] {
            [self.0, self.1, self.2]
        }

        pub const fn from_array(arr: [i32; 3]) -> Self {
            Self(arr[0], arr[1], arr[2])
        }

        pub const fn offset(self, dir: Direction) -> Self {
            let d = dir.offset();
            Self(self.0 + d[0], self.1 + d[1], self.2 + d[2])
        }

        pub fn neighbors(self) -> [Self; 6] {
            Direction::ALL.map(|d| self.offset(d))
        }

        pub const fn manhattan_distance(self, other: Self) -> u32 {
            self.0.abs_diff(other.0) + self.1.abs_diff(other.1) + self.2.abs_diff(other.2)
        }
    }

    /// Disjoint-set (`Union-Find`) structure with path compression and union by rank.
    #[derive(Debug, Clone)]
    pub struct UnionFind {
        parent: Vec<usize>,
        rank: Vec<u8>,
    }

    impl UnionFind {
        pub fn new(size: usize) -> Self {
            Self {
                parent: (0..size).collect(),
                rank: vec![0; size],
            }
        }

        pub fn find(&mut self, i: usize) -> usize {
            if self.parent[i] != i {
                let root = self.find(self.parent[i]);
                self.parent[i] = root;
            }
            self.parent[i]
        }

        pub fn union(&mut self, a: usize, b: usize) -> bool {
            let root_a = self.find(a);
            let root_b = self.find(b);
            if root_a == root_b {
                return false;
            }
            match self.rank[root_a].cmp(&self.rank[root_b]) {
                Ordering::Less => self.parent[root_a] = root_b,
                Ordering::Greater => self.parent[root_b] = root_a,
                Ordering::Equal => {
                    self.parent[root_b] = root_a;
                    self.rank[root_a] += 1;
                }
            }
            true
        }
    }

    /// A node in a pipe, tube, or cable network.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GridNode {
        /// Traversal weight (`1` for standard tube/pipe, `500` for RedPower Restrictor Tube).
        pub weight: u32,
        /// Optional paint color (for RedPower Pneumatic Tubes / Insulated Wires).
        pub color: Option<DyeColor>,
    }

    impl GridNode {
        pub const STANDARD: Self = Self {
            weight: 1,
            color: None,
        };
        /// RedPower 2 Restrictor Tube virtual distance weight (`500` blocks).
        pub const RESTRICTOR_WEIGHT: u32 = 500;

        pub const fn restrictor(color: Option<DyeColor>) -> Self {
            Self {
                weight: Self::RESTRICTOR_WEIGHT,
                color,
            }
        }

        pub fn allows_color(self, item_color: Option<DyeColor>) -> bool {
            match (self.color, item_color) {
                (None, _) | (_, None) => true,
                (Some(node_c), Some(item_c)) => node_c == item_c,
            }
        }
    }

    #[derive(Clone, Eq, PartialEq)]
    struct DijkstraState {
        cost: u32,
        pos: BlockPos,
    }

    impl Ord for DijkstraState {
        fn cmp(&self, other: &Self) -> Ordering {
            other
                .cost
                .cmp(&self.cost)
                .then_with(|| other.pos.cmp(&self.pos))
        }
    }

    impl PartialOrd for DijkstraState {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    /// Spatial grid graph for cables, pipes, and pneumatic tubes.
    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct GridGraph {
        pub nodes: HashMap<BlockPos, GridNode>,
    }

    impl GridGraph {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn insert_node(&mut self, pos: BlockPos, node: GridNode) {
            self.nodes.insert(pos, node);
        }

        pub fn remove_node(&mut self, pos: BlockPos) -> Option<GridNode> {
            self.nodes.remove(&pos)
        }

        /// Partition all nodes in the graph into connected components using [`UnionFind`].
        /// Two adjacent nodes connect if their colors are compatible.
        pub fn connected_components(&self) -> Vec<Vec<BlockPos>> {
            let mut positions: Vec<BlockPos> = self.nodes.keys().copied().collect();
            positions.sort();
            let index_of: HashMap<BlockPos, usize> =
                positions.iter().enumerate().map(|(i, &p)| (p, i)).collect();

            let mut uf = UnionFind::new(positions.len());
            for (i, &pos) in positions.iter().enumerate() {
                let node_a = self.nodes[&pos];
                for neighbor in pos.neighbors() {
                    if let Some(&j) = index_of.get(&neighbor) {
                        let node_b = self.nodes[&neighbor];
                        if node_a.allows_color(node_b.color) {
                            uf.union(i, j);
                        }
                    }
                }
            }

            let mut groups: HashMap<usize, Vec<BlockPos>> = HashMap::new();
            for (i, &pos) in positions.iter().enumerate() {
                let root = uf.find(i);
                groups.entry(root).or_default().push(pos);
            }
            let mut result: Vec<Vec<BlockPos>> = groups.into_values().collect();
            result.sort_by(|a, b| a[0].cmp(&b[0]));
            result
        }

        /// Find the lowest-cost route from `start` to the nearest reachable position in `goals`
        /// using Dijkstra's algorithm, respecting RedPower 2 tube color tags and Restrictor weights.
        /// Returns `Some((chosen_goal, full_path_from_start_to_goal, total_cost))`.
        pub fn shortest_path(
            &self,
            start: BlockPos,
            goals: &[BlockPos],
            item_color: Option<DyeColor>,
        ) -> Option<(BlockPos, Vec<BlockPos>, u32)> {
            if goals.is_empty() {
                return None;
            }
            let start_node = self.nodes.get(&start)?;
            if !start_node.allows_color(item_color) {
                return None;
            }

            let goal_set: std::collections::HashSet<BlockPos> = goals.iter().copied().collect();
            let mut dist: HashMap<BlockPos, u32> = HashMap::new();
            let mut prev: HashMap<BlockPos, BlockPos> = HashMap::new();
            let mut heap = BinaryHeap::new();

            dist.insert(start, 0);
            heap.push(DijkstraState {
                cost: 0,
                pos: start,
            });

            while let Some(DijkstraState { cost, pos }) = heap.pop() {
                if goal_set.contains(&pos) {
                    let mut path = vec![pos];
                    let mut curr = pos;
                    while let Some(&p) = prev.get(&curr) {
                        path.push(p);
                        curr = p;
                    }
                    path.reverse();
                    return Some((pos, path, cost));
                }

                if cost > *dist.get(&pos).unwrap_or(&u32::MAX) {
                    continue;
                }

                for next_pos in pos.neighbors() {
                    let Some(next_node) = self.nodes.get(&next_pos) else {
                        continue;
                    };
                    if !next_node.allows_color(item_color) {
                        continue;
                    }
                    let next_cost = cost.saturating_add(next_node.weight.max(1));
                    if next_cost < *dist.get(&next_pos).unwrap_or(&u32::MAX) {
                        dist.insert(next_pos, next_cost);
                        prev.insert(next_pos, pos);
                        heap.push(DijkstraState {
                            cost: next_cost,
                            pos: next_pos,
                        });
                    }
                }
            }
            None
        }
    }
}

// ============================================================================
// 6. 16-Color Dyes, Bundled Signals & Frequency Triads
// ============================================================================

pub mod frequency {
    use super::*;

    /// Standard 16 Minecraft dye colors (`0..16`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[repr(u8)]
    pub enum DyeColor {
        White = 0,
        Orange = 1,
        Magenta = 2,
        LightBlue = 3,
        Yellow = 4,
        Lime = 5,
        Pink = 6,
        Gray = 7,
        LightGray = 8,
        Cyan = 9,
        Purple = 10,
        Blue = 11,
        Brown = 12,
        Green = 13,
        Red = 14,
        Black = 15,
    }

    impl DyeColor {
        pub const fn from_u8(val: u8) -> Self {
            match val & 0x0F {
                0 => Self::White,
                1 => Self::Orange,
                2 => Self::Magenta,
                3 => Self::LightBlue,
                4 => Self::Yellow,
                5 => Self::Lime,
                6 => Self::Pink,
                7 => Self::Gray,
                8 => Self::LightGray,
                9 => Self::Cyan,
                10 => Self::Purple,
                11 => Self::Blue,
                12 => Self::Brown,
                13 => Self::Green,
                14 => Self::Red,
                _ => Self::Black,
            }
        }

        pub const fn next(self) -> Self {
            Self::from_u8((self as u8 + 1) & 0x0F)
        }

        pub fn from_dye_item(item_id: &str) -> Option<Self> {
            match item_id {
                "minecraft:white_dye" => Some(Self::White),
                "minecraft:orange_dye" => Some(Self::Orange),
                "minecraft:magenta_dye" => Some(Self::Magenta),
                "minecraft:light_blue_dye" => Some(Self::LightBlue),
                "minecraft:yellow_dye" => Some(Self::Yellow),
                "minecraft:lime_dye" => Some(Self::Lime),
                "minecraft:pink_dye" => Some(Self::Pink),
                "minecraft:gray_dye" => Some(Self::Gray),
                "minecraft:light_gray_dye" => Some(Self::LightGray),
                "minecraft:cyan_dye" => Some(Self::Cyan),
                "minecraft:purple_dye" => Some(Self::Purple),
                "minecraft:blue_dye" => Some(Self::Blue),
                "minecraft:brown_dye" => Some(Self::Brown),
                "minecraft:green_dye" => Some(Self::Green),
                "minecraft:red_dye" => Some(Self::Red),
                "minecraft:black_dye" => Some(Self::Black),
                _ => None,
            }
        }

        pub const fn name(self) -> &'static str {
            match self {
                Self::White => "White",
                Self::Orange => "Orange",
                Self::Magenta => "Magenta",
                Self::LightBlue => "Light Blue",
                Self::Yellow => "Yellow",
                Self::Lime => "Lime",
                Self::Pink => "Pink",
                Self::Gray => "Gray",
                Self::LightGray => "Light Gray",
                Self::Cyan => "Cyan",
                Self::Purple => "Purple",
                Self::Blue => "Blue",
                Self::Brown => "Brown",
                Self::Green => "Green",
                Self::Red => "Red",
                Self::Black => "Black",
            }
        }

        pub const fn bit(self) -> u16 {
            1u16 << (self as u8)
        }
    }

    /// 16-channel Bundled Cable state for RedPower 2 and ComputerCraft (`redstone.getBundledInput`).
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct BundledSignal {
        pub levels: [u8; 16],
    }

    impl BundledSignal {
        pub const fn new() -> Self {
            Self { levels: [0; 16] }
        }

        pub const fn level(&self, color: DyeColor) -> u8 {
            self.levels[color as usize]
        }

        pub fn set_level(&mut self, color: DyeColor, strength: u8) {
            self.levels[color as usize] = strength;
        }

        /// Convert to ComputerCraft's 16-bit digital bundled bitmask (`1 << color` for non-zero channels).
        pub fn to_digital_mask(&self) -> u16 {
            let mut mask = 0u16;
            for i in 0..16 {
                if self.levels[i] > 0 {
                    mask |= 1u16 << i;
                }
            }
            mask
        }

        /// Construct from ComputerCraft's 16-bit digital bundled bitmask (`255` for active bits).
        pub fn from_digital_mask(mask: u16) -> Self {
            let mut levels = [0u8; 16];
            for (i, slot) in levels.iter_mut().enumerate() {
                if (mask & (1u16 << i)) != 0 {
                    *slot = 255;
                }
            }
            Self { levels }
        }
    }

    /// A 3-color frequency triad ($16^3 = 4,096$ channels) used by EnderStorage and WR-CBE.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct FrequencyKey(pub DyeColor, pub DyeColor, pub DyeColor);

    impl FrequencyKey {
        pub const DEFAULT: Self = Self(DyeColor::White, DyeColor::White, DyeColor::White);

        pub const fn to_index(self) -> u16 {
            ((self.0 as u16) << 8) | ((self.1 as u16) << 4) | (self.2 as u16)
        }

        pub const fn from_index(index: u16) -> Self {
            let c0 = DyeColor::from_u8(((index >> 8) & 0x0F) as u8);
            let c1 = DyeColor::from_u8(((index >> 4) & 0x0F) as u8);
            let c2 = DyeColor::from_u8((index & 0x0F) as u8);
            Self(c0, c1, c2)
        }

        pub fn cycle_pad(&mut self, pad: usize) {
            match pad {
                0 => self.0 = self.0.next(),
                1 => self.1 = self.1.next(),
                2 => self.2 = self.2.next(),
                _ => {}
            }
        }
    }

    /// Channel ownership: public pool or private player UUID namespace (Diamond Latch / Tesseract).
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum ChannelOwner {
        Public,
        Private(String),
    }
}

// ============================================================================
// 7. Multiblock Pattern Validation
// ============================================================================

pub mod multiblock {
    use super::*;
    use crate::grid::BlockPos;

    /// Structural template for 3D multiblocks (Coke Oven, Blast Furnace, Alveary, Tanks, MAC).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum MultiblockShape {
        /// Every block in `[0..width, 0..height, 0..depth]` must equal `block_id`
        /// (e.g. Railcraft 3x3x3 Coke Oven, Forestry 3x3x3 Alveary).
        SolidCuboid {
            width: i32,
            height: i32,
            depth: i32,
            block_id: String,
        },
        /// Outer shell blocks must be in `shell_ids`, while interior `[1..width-1, 1..height-1, 1..depth-1]`
        /// must be empty (`None` or `"minecraft:air"`) or in `interior_ids`
        /// (e.g. Railcraft 3x3x4 Blast Furnace, Iron/Steel Tanks, AE1 Molecular Assembler Chamber).
        HollowCuboid {
            width: i32,
            height: i32,
            depth: i32,
            shell_ids: Vec<String>,
            interior_ids: Vec<String>,
        },
    }

    impl MultiblockShape {
        /// Validate whether the multiblock is properly formed with minimum corner at `origin`.
        pub fn validate_at<F>(&self, origin: BlockPos, get_block: F) -> bool
        where
            F: Fn(BlockPos) -> Option<String>,
        {
            match self {
                Self::SolidCuboid {
                    width,
                    height,
                    depth,
                    block_id,
                } => {
                    if *width <= 0 || *height <= 0 || *depth <= 0 {
                        return false;
                    }
                    for dx in 0..*width {
                        for dy in 0..*height {
                            for dz in 0..*depth {
                                let pos = BlockPos(origin.0 + dx, origin.1 + dy, origin.2 + dz);
                                if get_block(pos).as_deref() != Some(block_id.as_str()) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                }
                Self::HollowCuboid {
                    width,
                    height,
                    depth,
                    shell_ids,
                    interior_ids,
                } => {
                    if *width < 3 || *height < 3 || *depth < 3 {
                        return false;
                    }
                    for dx in 0..*width {
                        for dy in 0..*height {
                            for dz in 0..*depth {
                                let pos = BlockPos(origin.0 + dx, origin.1 + dy, origin.2 + dz);
                                let is_boundary = dx == 0
                                    || dx == *width - 1
                                    || dy == 0
                                    || dy == *height - 1
                                    || dz == 0
                                    || dz == *depth - 1;
                                let actual = get_block(pos);
                                if is_boundary {
                                    let Some(b) = actual else {
                                        return false;
                                    };
                                    if !shell_ids.iter().any(|s| s == &b) {
                                        return false;
                                    }
                                } else {
                                    match actual.as_deref() {
                                        None | Some("minecraft:air") => {
                                            if !interior_ids.is_empty()
                                                && !interior_ids
                                                    .iter()
                                                    .any(|s| s == "minecraft:air")
                                            {
                                                return false;
                                            }
                                        }
                                        Some(b) => {
                                            if !interior_ids.iter().any(|s| s == b) {
                                                return false;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                }
            }
        }
    }
}

// ============================================================================
// 8. Horizon Native Hooks API (API 8–40) Mirror Types & Trait
// ============================================================================

pub mod hooks {
    use super::*;
    use crate::inventory::ItemStack;

    /// Mirrors `NativeMod` in `native_hooks.rs` (`Options > Mods` registration).
    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct NativeModInfo {
        pub id: String,
        pub title: String,
        pub description: String,
        pub keys: Vec<[String; 3]>,
    }

    /// Mirrors `ModContainer` in `horizon_client::containers` (API 16 & 17).
    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ModContainer {
        pub slots: usize,
        pub menu: Option<String>,
        pub inputs: Vec<usize>,
        pub outputs: Vec<usize>,
        pub faces: [Vec<usize>; 6],
    }

    /// Mirrors `ModBlockEntity` in `horizon_client::containers` (API 15 & 17).
    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ModBlockEntity {
        pub id: String,
        pub nbt: HashMap<String, String>,
        pub items: Vec<Option<ItemStack>>,
        pub data: Vec<i32>,
    }

    /// Mirrors `NativeScreen` in `native_hooks.rs` (API 19).
    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    pub struct NativeScreen {
        pub width: f32,
        pub height: f32,
        pub slots: Vec<Option<[f32; 2]>>,
        pub inventory_y: f32,
        pub inventory_x: f32,
        pub own_labels: bool,
    }

    /// Trait modeling Horizon's `NATIVE_HOOKS_API = 40` entrypoints in
    /// `crates/horizon-client/src/game/native_hooks.rs`.
    /// Default implementations are no-ops so each mod only overrides the hooks it uses.
    pub trait HorizonNativeMod {
        fn native_mods(&self, mods: &mut Vec<NativeModInfo>) {
            let _ = mods;
        }

        fn native_tooltip(&self, stack: &ItemStack, lines: &mut Vec<String>) {
            let _ = (stack, lines);
        }

        fn native_use_item_on(
            &mut self,
            stack: &ItemStack,
            block_at_pos: &str,
            entity: Option<&mut ModBlockEntity>,
            acting_player_uuid: Option<&str>,
            claimed: &mut bool,
        ) {
            let _ = (stack, block_at_pos, entity, acting_player_uuid, claimed);
        }

        fn native_block_entity(
            &mut self,
            block: &str,
            entity: &mut Option<(String, HashMap<String, String>)>,
        ) {
            let _ = (block, entity);
        }

        fn native_block_entity_ticks(&self, block: &str, ticks: &mut bool) {
            let _ = (block, ticks);
        }

        fn native_block_entity_container(&self, block: &str, container: &mut Option<ModContainer>) {
            let _ = (block, container);
        }

        fn native_block_entity_tick(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity) {
            let _ = (pos, entity);
        }

        fn native_screen(&self, block: &str, screen: &mut Option<NativeScreen>) {
            let _ = (block, screen);
        }

        fn native_screen_click(
            &self,
            block: &str,
            data: &[i32],
            at: [f32; 2],
            right: bool,
            button: &mut Option<i32>,
        ) {
            let _ = (block, data, at, right, button);
        }

        fn native_screen_tooltip(
            &self,
            block: &str,
            data: &[i32],
            at: [f32; 2],
            lines: &mut Vec<String>,
        ) {
            let _ = (block, data, at, lines);
        }

        fn native_menu_button(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity, id: i32) {
            let _ = (pos, entity, id);
        }

        fn native_is_signal_source(&self, block: &str, source: &mut bool) {
            let _ = (block, source);
        }

        fn native_block_signal(
            &self,
            block: &str,
            entity: Option<&ModBlockEntity>,
            direct: bool,
            power: &mut i32,
        ) {
            let _ = (block, entity, direct, power);
        }

        fn native_net_from_client(&mut self, player_uuid: &str, channel: &str, data: &[u8]) {
            let _ = (player_uuid, channel, data);
        }

        fn native_net_from_server(&mut self, channel: &str, data: &[u8]) {
            let _ = (channel, data);
        }
    }
}

// ============================================================================
// 9. Deterministic Tick & RNG Utilities
// ============================================================================

pub mod util {
    use super::*;

    /// Repeating tick timer for machine cycles, RedPower Timers, and quarry steps.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TickTimer {
        pub elapsed: u32,
        pub period: u32,
    }

    impl TickTimer {
        pub fn new(period: u32) -> Self {
            Self {
                elapsed: 0,
                period: period.max(1),
            }
        }

        /// Advance by 1 tick; returns `true` and resets to `0` each time `period` is reached.
        pub fn tick(&mut self) -> bool {
            self.elapsed += 1;
            if self.elapsed >= self.period {
                self.elapsed = 0;
                true
            } else {
                false
            }
        }

        /// Progress fraction in `0.0..=1.0` for GUI progress bars.
        pub fn progress(&self) -> f32 {
            (self.elapsed as f32 / self.period.max(1) as f32).clamp(0.0, 1.0)
        }
    }

    /// Fast, deterministic, seedable 64-bit PRNG for Recycler scrap rolls, Pulverizer byproducts,
    /// and Forestry bee Mendelian allele inheritance.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct XorShift64 {
        state: u64,
    }

    impl XorShift64 {
        pub const fn new(seed: u64) -> Self {
            Self {
                state: if seed == 0 {
                    0x9E37_79B9_7F4A_7C15
                } else {
                    seed
                },
            }
        }

        pub fn next_u64(&mut self) -> u64 {
            let mut x = self.state;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.state = x;
            x
        }

        /// Return `true` with probability `1 / one_in_n` (e.g. `one_in(8)` for IC2 Recycler 12.5% scrap).
        pub fn one_in(&mut self, one_in_n: u64) -> bool {
            if one_in_n <= 1 {
                return true;
            }
            self.next_u64().is_multiple_of(one_in_n)
        }

        /// Return `true` with percentage probability `percent` (`0..=100`).
        pub fn chance_percent(&mut self, percent: u32) -> bool {
            if percent >= 100 {
                return true;
            }
            if percent == 0 {
                return false;
            }
            ((self.next_u64() % 100) as u32) < percent
        }
    }
}
