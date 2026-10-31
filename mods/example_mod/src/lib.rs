//! Reference Rust mod implementation for Rekkit targeting the Horizon engine.
//!
//! Horizon splits mods into two artifacts:
//! 1. **Content Mod (`.hmod`, API 0.2)**: Pure data & assets (`content/horizon-mod.toml`,
//!    `content/data/`, `content/assets/`) declaring blocks, items, mob effects, models, recipes, and tags.
//! 2. **Native Mod (`.hnmod`, `NATIVE_HOOKS_API = 40`)**: Source-level mixin bundle
//!    (`native/horizon-native.json` + `native/mixins/<id>/`) hooking into
//!    `crates/horizon-client/src/game/native_hooks.rs`.
//!
//! This crate houses the pure, standalone Rust domain logic and the `HorizonNativeMod`
//! adapter layer so all machine state machines, container inventories, and GUI layouts
//! can be unit- and integration-tested headlessly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current Horizon `NATIVE_HOOKS_API` version targeted by Rekkit mods.
pub const NATIVE_HOOKS_API: u32 = 40;

/// Pinned Horizon engine revision for native mixin compatibility.
pub const HORIZON_REVISION: &str = "38a97ea64e816e1f897d1fbc7c7ded3b0ce87f56";

/// Pinned `horizon-mixins` tool revision (`native_mods::MIXINS_REV`).
pub const MIXINS_REVISION: &str = "3f99a5c60dfa311cc365524c44ae2fd799bfa1de";

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

impl DyeColor {
    /// Convert a 0..16 integer index back into a [`DyeColor`].
    pub fn from_u8(val: u8) -> Self {
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

    /// Cycle to the next dye color in order.
    pub fn next(self) -> Self {
        Self::from_u8((self as u8 + 1) & 0x0F)
    }

    /// Parse a vanilla dye item identifier (e.g. `"minecraft:red_dye"`) into a [`DyeColor`].
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

    pub fn name(self) -> &'static str {
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

    /// Reconstruct a [`FrequencyKey`] from a 12-bit integer index (0..4096).
    pub fn from_index(index: u16) -> Self {
        let c0 = DyeColor::from_u8(((index >> 8) & 0x0F) as u8);
        let c1 = DyeColor::from_u8(((index >> 4) & 0x0F) as u8);
        let c2 = DyeColor::from_u8((index & 0x0F) as u8);
        Self(c0, c1, c2)
    }

    /// Cycle one of the 3 dye pads (`pad` in 0..3).
    pub fn cycle_pad(&mut self, pad: usize) {
        match pad {
            0 => self.0 = self.0.next(),
            1 => self.1 = self.1.next(),
            2 => self.2 = self.2.next(),
            _ => {}
        }
    }
}

/// Channel ownership: public or player-locked via diamond latch.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelOwner {
    Public,
    Private(String), // Player UUID string
}

/// Simulated inventory item stack (mirrors `horizon_client::items::ItemStack`).
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
                let space = 64u32.saturating_sub(slot.count);
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

    pub fn get_or_create(
        &mut self,
        owner: ChannelOwner,
        freq: FrequencyKey,
    ) -> &mut FrequencyStorage {
        let key = (owner.clone(), freq);
        self.channels
            .entry(key)
            .or_insert_with(|| FrequencyStorage::new(freq, owner))
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}

// ============================================================================
// Horizon Native Hooks API (API 15–22) Mirror Types & Trait
// Mirrors `crates/horizon-client/src/game/native_hooks.rs`
// ============================================================================

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
    /// 16-bit synchronized menu data slots (`ContainerSetData`).
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

/// Trait modeling Horizon's `NATIVE_HOOKS_API` (API 13–22) entrypoints in
/// `crates/horizon-client/src/game/native_hooks.rs`.
pub trait HorizonNativeMod {
    /// Register the mod in `Options > Mods` (`Game::native_mods`).
    fn native_mods(&self, mods: &mut Vec<NativeModInfo>);

    /// Attach a block entity when a mod block is placed (`Game::native_block_entity`, API 15).
    fn native_block_entity(
        &mut self,
        block: &str,
        entity: &mut Option<(String, HashMap<String, String>)>,
    );

    /// Declare whether a mod block's block entity ticks (`Game::native_block_entity_ticks`, API 15).
    fn native_block_entity_ticks(&self, block: &str, ticks: &mut bool);

    /// Declare item slots and menu type for a mod block (`Game::native_block_entity_container`, API 17).
    fn native_block_entity_container(&self, block: &str, container: &mut Option<ModContainer>);

    /// Server tick of a mod block entity at `pos` (`Game::native_block_entity_tick`, API 17).
    fn native_block_entity_tick(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity);

    /// Lay out the custom container screen for `block` (`Game::native_screen`, API 19).
    fn native_screen(&self, block: &str, screen: &mut Option<NativeScreen>);

    /// Map a GUI click at `at` to a menu button ID (`Game::native_screen_click`, API 20).
    fn native_screen_click(
        &self,
        block: &str,
        data: &[i32],
        at: [f32; 2],
        right: bool,
        button: &mut Option<i32>,
    );

    /// Provide custom screen tooltips at `at` (`Game::native_screen_tooltip`, API 20).
    fn native_screen_tooltip(&self, block: &str, data: &[i32], at: [f32; 2], lines: &mut Vec<String>);

    /// Handle a server-side menu button click on a mod block entity (`Game::native_menu_button`, API 20).
    fn native_menu_button(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity, id: i32);

    /// Handle right-clicking an item onto a block (`Game::native_use_item_on`, API 13).
    fn native_use_item_on(
        &mut self,
        stack: &ItemStack,
        block_at_pos: &str,
        entity: Option<&mut ModBlockEntity>,
        acting_player_uuid: Option<&str>,
        claimed: &mut bool,
    );

    /// Declare whether `block` connects to redstone dust (`Game::native_is_signal_source`, API 38).
    fn native_is_signal_source(&self, block: &str, source: &mut bool) {
        let _ = (block, source);
    }

    /// Emit weak/strong redstone signal `0..=15` (`Game::native_block_signal`, API 38).
    fn native_block_signal(
        &self,
        block: &str,
        entity: Option<&ModBlockEntity>,
        direct: bool,
        power: &mut i32,
    ) {
        let _ = (block, entity, direct, power);
    }

    /// Receive a custom `horizon:mod_net` packet from a client (`Game::native_net_from_client`, API 40).
    fn native_net_from_client(&mut self, player_uuid: &str, channel: &str, data: &[u8]) {
        let _ = (player_uuid, channel, data);
    }
}

/// Legacy lifecycle hook trait for general mod initialization and world ticks.
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

impl ExampleMod {
    pub const MOD_ID: &'static str = "example_mod";
    pub const BLOCK_ENDER_CHEST: &'static str = "example_mod:ender_chest";
    pub const ITEM_ENDER_POUCH: &'static str = "example_mod:ender_pouch";
    pub const EFFECT_ENDER_LOCK: &'static str = "example_mod:ender_lock";
    pub const RECIPE_TYPE_TUNE: &'static str = "example_mod:frequency_tune";
    pub const NET_CHANNEL_TUNE: &'static str = "example_mod:set_frequency";
    pub const ENTITY_ID: &'static str = "example_mod:ender_chest";

    /// Read the [`FrequencyKey`] stored in a block entity's NBT map.
    pub fn read_frequency(nbt: &HashMap<String, String>) -> FrequencyKey {
        nbt.get("frequency")
            .and_then(|v| v.parse::<u16>().ok())
            .map(FrequencyKey::from_index)
            .unwrap_or(FrequencyKey::DEFAULT)
    }

    /// Write a [`FrequencyKey`] into a block entity's NBT map.
    pub fn write_frequency(nbt: &mut HashMap<String, String>, freq: FrequencyKey) {
        nbt.insert("frequency".to_string(), freq.to_index().to_string());
    }

    /// Read the [`ChannelOwner`] stored in a block entity's NBT map.
    pub fn read_owner(nbt: &HashMap<String, String>) -> ChannelOwner {
        match nbt.get("owner").map(String::as_str) {
            Some(uuid) if !uuid.is_empty() => ChannelOwner::Private(uuid.to_string()),
            _ => ChannelOwner::Public,
        }
    }
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
        Self::MOD_ID
    }

    fn init(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }

    fn on_tick(&mut self, _tick_count: u64) {
        // Global server tick (API 4 `native_server_tick`)
    }
}

impl HorizonNativeMod for ExampleMod {
    fn native_mods(&self, mods: &mut Vec<NativeModInfo>) {
        mods.push(NativeModInfo {
            id: Self::MOD_ID.into(),
            title: "Example Mod 0.1.0".into(),
            description: "Ender Frequency Storage reference mod for Rekkit.".into(),
            keys: Vec::new(),
        });
    }

    fn native_block_entity(
        &mut self,
        block: &str,
        entity: &mut Option<(String, HashMap<String, String>)>,
    ) {
        if entity.is_none() && block == Self::BLOCK_ENDER_CHEST {
            let mut nbt = HashMap::new();
            Self::write_frequency(&mut nbt, FrequencyKey::DEFAULT);
            *entity = Some((Self::ENTITY_ID.into(), nbt));
        }
    }

    fn native_block_entity_ticks(&self, block: &str, ticks: &mut bool) {
        if block == Self::BLOCK_ENDER_CHEST {
            *ticks = true;
        }
    }

    fn native_block_entity_container(&self, block: &str, container: &mut Option<ModContainer>) {
        if container.is_none() && block == Self::BLOCK_ENDER_CHEST {
            let all_slots: Vec<usize> = (0..FrequencyStorage::SLOT_COUNT).collect();
            *container = Some(ModContainer {
                slots: FrequencyStorage::SLOT_COUNT,
                menu: Some("generic_9x3".into()),
                inputs: all_slots.clone(),
                outputs: all_slots.clone(),
                faces: std::array::from_fn(|_| all_slots.clone()),
            });
        }
    }

    fn native_block_entity_tick(&mut self, _pos: [i32; 3], entity: &mut ModBlockEntity) {
        if entity.id != Self::ENTITY_ID {
            return;
        }
        let freq = Self::read_frequency(&entity.nbt);
        let owner = Self::read_owner(&entity.nbt);
        let is_private = i32::from(matches!(owner, ChannelOwner::Private(_)));

        let channel = self.network.get_or_create(owner, freq);
        // If the block entity's slots were modified (e.g., by hoppers or player interaction),
        // push into the channel if non-empty, otherwise pull the channel's shared items.
        if entity.items.len() == FrequencyStorage::SLOT_COUNT && entity.items != channel.items {
            if entity.items.iter().any(Option::is_some) && channel.items.iter().all(Option::is_none)
            {
                channel.items = entity.items.clone();
            } else {
                entity.items = channel.items.clone();
            }
        } else if entity.items.len() != FrequencyStorage::SLOT_COUNT {
            entity.items = channel.items.clone();
        }

        // Synchronize frequency index (data[0]) and private lock flag (data[1]) to open client screens
        entity.data = vec![i32::from(freq.to_index()), is_private];
    }

    fn native_screen(&self, block: &str, screen: &mut Option<NativeScreen>) {
        if screen.is_none() && block == Self::BLOCK_ENDER_CHEST {
            let mut slots = Vec::with_capacity(FrequencyStorage::SLOT_COUNT);
            for row in 0..3 {
                for col in 0..9 {
                    let x = 8.0 + (col as f32) * 18.0;
                    let y = 30.0 + (row as f32) * 18.0;
                    slots.push(Some([x, y]));
                }
            }
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 180.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 98.0,
                own_labels: false,
            });
        }
    }

    fn native_screen_click(
        &self,
        block: &str,
        _data: &[i32],
        at: [f32; 2],
        right: bool,
        button: &mut Option<i32>,
    ) {
        if button.is_some() || right || block != Self::BLOCK_ENDER_CHEST {
            return;
        }
        // Three 16x10 dye frequency buttons in the header bar at y = 14..24
        if (14.0..=24.0).contains(&at[1]) {
            for pad in 0..3i32 {
                let x0 = 62.0 + (pad as f32) * 20.0;
                if (x0..=x0 + 16.0).contains(&at[0]) {
                    *button = Some(pad);
                    return;
                }
            }
        }
    }

    fn native_screen_tooltip(
        &self,
        block: &str,
        data: &[i32],
        at: [f32; 2],
        lines: &mut Vec<String>,
    ) {
        if block != Self::BLOCK_ENDER_CHEST {
            return;
        }
        if (14.0..=24.0).contains(&at[1]) && (62.0..=118.0).contains(&at[0]) {
            let freq = data
                .first()
                .and_then(|&v| u16::try_from(v).ok())
                .map(FrequencyKey::from_index)
                .unwrap_or(FrequencyKey::DEFAULT);
            lines.push(format!(
                "Frequency: {} / {} / {}",
                freq.0.name(),
                freq.1.name(),
                freq.2.name()
            ));
            if data.get(1).copied().unwrap_or(0) != 0 {
                lines.push("§bPrivate Channel (Diamond Locked)".into());
            }
        }
    }

    fn native_menu_button(&mut self, _pos: [i32; 3], entity: &mut ModBlockEntity, id: i32) {
        if entity.id != Self::ENTITY_ID || !(0..3).contains(&id) {
            return;
        }
        let mut freq = Self::read_frequency(&entity.nbt);
        freq.cycle_pad(id as usize);
        Self::write_frequency(&mut entity.nbt, freq);

        let owner = Self::read_owner(&entity.nbt);
        let channel = self.network.get_or_create(owner, freq);
        entity.items = channel.items.clone();
    }

    fn native_use_item_on(
        &mut self,
        stack: &ItemStack,
        block_at_pos: &str,
        entity: Option<&mut ModBlockEntity>,
        acting_player_uuid: Option<&str>,
        claimed: &mut bool,
    ) {
        if *claimed || block_at_pos != Self::BLOCK_ENDER_CHEST {
            return;
        }
        let Some(entity) = entity else {
            return;
        };
        if stack.item_id == "minecraft:diamond" {
            if let Some(uuid) = acting_player_uuid {
                entity.nbt.insert("owner".into(), uuid.into());
                *claimed = true;
            }
        } else if let Some(color) = DyeColor::from_dye_item(&stack.item_id) {
            let mut freq = Self::read_frequency(&entity.nbt);
            freq.0 = color;
            Self::write_frequency(&mut entity.nbt, freq);
            *claimed = true;
        }
    }

    fn native_is_signal_source(&self, block: &str, source: &mut bool) {
        if block == Self::BLOCK_ENDER_CHEST {
            *source = true;
        }
    }

    fn native_block_signal(
        &self,
        block: &str,
        entity: Option<&ModBlockEntity>,
        _direct: bool,
        power: &mut i32,
    ) {
        if block != Self::BLOCK_ENDER_CHEST {
            return;
        }
        let Some(entity) = entity else {
            return;
        };
        let occupied = entity.items.iter().filter(|s| s.is_some()).count();
        if occupied > 0 {
            let signal = 1 + ((occupied * 14) / FrequencyStorage::SLOT_COUNT) as i32;
            *power = (*power).max(signal.clamp(1, 15));
        }
    }

    fn native_net_from_client(&mut self, player_uuid: &str, channel: &str, data: &[u8]) {
        if channel == Self::NET_CHANNEL_TUNE && data.len() == 2 {
            let idx = u16::from_le_bytes([data[0], data[1]]) & 0x0FFF;
            let freq = FrequencyKey::from_index(idx);
            let owner = ChannelOwner::Private(player_uuid.to_string());
            let _ = self.network.get_or_create(owner, freq);
        }
    }
}
