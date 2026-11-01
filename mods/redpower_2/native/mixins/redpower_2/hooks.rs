//! Native Mixin Hook Entrypoints for `redpower_2` (`NATIVE_HOOKS_API = 40`).
//!
//! Every hook guards its namespace (`block.starts_with("redpower_2:")` or `item.starts_with("redpower_2:")`)
//! before acting and never overwrites out-parameters claimed by another mod.
//!
//! Hook points wired:
//! - `Game::native_mods`: Lists `redpower_2` in `Options > Mods`.
//! - `Game::native_use_item_on` (API 13): Screwdriver rotation/mode cycling and Paintbrush coloring.
//! - `Game::native_block_entity` & `Game::native_block_entity_ticks` (API 15): Attaches and enables ticking block entities.
//! - `Game::native_block_entity_container` & `Game::native_block_entity_tick` (API 16-17): Alloy Furnace, Buffer, Filter, Sorting Machine, Battery Box, Logic Gates, and 6502 CPU ticking.
//! - `Game::native_screen`, `Game::native_screen_click`, `Game::native_screen_tooltip`, `Game::native_menu_button` (API 19-20): Custom screens and Timer/Filter controls.
//! - `Game::native_set_block` (API 24): Block Breaker, Deployer, and Frame Motor structure movement.
//! - `Game::native_update_shape` & `Game::native_schedule_block_tick` (API 32): Wire/tube 6-face connections and gate delays.
//! - `Game::native_tick_block` (API 34): Scheduled logic gate pulses.
//! - `Game::native_recipes_of_type` (API 36): Custom `"redpower_2:alloy_furnace"` recipes.
//! - `Game::native_is_signal_source` & `Game::native_block_signal` (API 38): Red Alloy Wire and Logic Gate redstone output (`0..=15`).
//! - `Game::native_net_from_client` (API 40): 6502 Monitor keyboard input on `horizon:mod_net`.

pub const MOD_ID: &str = "redpower_2";
pub const HOOKS_API: u32 = 40;

pub fn owns_id(id: &str) -> bool {
    id.starts_with("redpower_2:")
}
