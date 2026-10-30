//! Reference Rust mod implementation for Rekkit targeting the Horizon engine.
//!
//! This module demonstrates the architectural patterns for porting classic Java
//! mods (e.g. EnderStorage, RedPower, IC2) into standalone Rust crates ready
//! for Horizon's plugin and component lifecycle.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard 16 Minecraft dye color codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// A 3-color frequency triad (4,096 possible channels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrequencyKey(pub DyeColor, pub DyeColor, pub DyeColor);

impl FrequencyKey {
    pub const DEFAULT: Self = Self(DyeColor::White, DyeColor::White, DyeColor::White);

    /// Compute a unique integer index in range 0..4096.
    pub fn to_index(&self) -> u16 {
        ((self.0 as u16) << 8) | ((self.1 as u16) << 4) | (self.2 as u16)
    }
}

/// Channel ownership: public or player-locked via diamond latch.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelOwner {
    Public,
    Private(String), // Player UUID string
}

/// Simulated inventory item stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: String,
    pub count: u32,
}

/// Core domain state for a frequency-linked container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyStorage {
    pub frequency: FrequencyKey,
    pub owner: ChannelOwner,
    pub items: Vec<Option<ItemStack>>,
    pub is_open: bool,
}

impl FrequencyStorage {
    pub const SLOT_COUNT: usize = 27;

    pub fn new(frequency: FrequencyKey, owner: ChannelOwner) -> Self {
        Self {
            frequency,
            owner,
            items: vec![None; Self::SLOT_COUNT],
            is_open: false,
        }
    }

    /// Insert an item into the first available matching or empty slot.
    pub fn insert_item(&mut self, mut stack: ItemStack) -> Option<ItemStack> {
        // First try to merge with existing stacks
        for slot in self.items.iter_mut().flatten() {
            if slot.item_id == stack.item_id {
                let space = 64 - slot.count;
                let transfer = space.min(stack.count);
                slot.count += transfer;
                stack.count -= transfer;
                if stack.count == 0 {
                    return None;
                }
            }
        }

        // Then place into first empty slot
        for slot in self.items.iter_mut() {
            if slot.is_none() {
                *slot = Some(stack);
                return None;
            }
        }

        Some(stack) // Inventory full, return leftovers
    }
}

/// Global registry managing cross-dimensional storage channels.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StorageNetworkManager {
    channels: HashMap<(ChannelOwner, FrequencyKey), FrequencyStorage>,
}

impl StorageNetworkManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create(&mut self, owner: ChannelOwner, freq: FrequencyKey) -> &mut FrequencyStorage {
        let key = (owner.clone(), freq);
        self.channels.entry(key).or_insert_with(|| FrequencyStorage::new(freq, owner))
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}

/// Mod lifecycle hook trait expected by the Horizon plugin interface.
pub trait HorizonMod {
    fn id(&self) -> &'static str;
    fn init(&mut self) -> Result<(), String>;
    fn on_tick(&mut self, tick_count: u64);
}

/// Main mod implementation for `example_mod`.
pub struct ExampleMod {
    pub network: StorageNetworkManager,
    pub initialized: bool,
}

impl Default for ExampleMod {
    fn default() -> Self {
        Self {
            network: StorageNetworkManager::new(),
            initialized: false,
        }
    }
}

impl HorizonMod for ExampleMod {
    fn id(&self) -> &'static str {
        "example_mod"
    }

    fn init(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }

    fn on_tick(&mut self, _tick_count: u64) {
        // Ticking logic: reconcile container updates, decay timers, etc.
    }
}
