//! Native Mixin Hook Entrypoints for `example_mod` (`NATIVE_HOOKS_API = 40`).
//!
//! When built by `horizon-launcher native-build`, `horizon-mixins` injects HEAD hooks
//! into `crates/horizon-client/src/game/native_hooks.rs` that forward into the
//! `example_mod` state machine.
//!
//! Hook points used:
//! - `Game::native_mods`: Lists `example_mod` in Options > Mods.
//! - `Game::native_block_entity` (API 15): Attaches a `ModBlockEntity` when `example_mod:ender_chest` is placed.
//! - `Game::native_block_entity_ticks` (API 15): Enables server-side ticking for `example_mod:ender_chest`.
//! - `Game::native_block_entity_container` (API 17): Registers 27-slot `generic_9x3` container & hopper rules.
//! - `Game::native_block_entity_tick` (API 17): Synchronizes block entity inventory with the frequency network.
//! - `Game::native_screen` (API 19): Lays out the 27-slot container screen and frequency bar.
//! - `Game::native_screen_draw` (API 19): Renders the 3-dye frequency indicator header.
//! - `Game::native_menu_button` (API 20): Cycles frequency dye pads when clicked in the container screen.
//! - `Game::native_use_item_on` (API 13): Allows right-clicking dye or diamond onto the chest to tune/lock it.
//! - `Game::native_block_signal` & `Game::native_is_signal_source` (API 38): Emits redstone signal proportional to chest fill level.
//! - `Game::native_net_from_client` (API 40): Receives custom `horizon:mod_net` frequency-search packets.

pub const MOD_ID: &str = "example_mod";
pub const BLOCK_ENDER_CHEST: &str = "example_mod:ender_chest";
pub const ITEM_ENDER_POUCH: &str = "example_mod:ender_pouch";
pub const EFFECT_ENDER_LOCK: &str = "example_mod:ender_lock";
pub const HOOKS_API: u32 = 40;
