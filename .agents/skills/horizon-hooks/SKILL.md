---
name: horizon-hooks
description: How Rekkit native mixin bundles hook into Horizon's native_hooks.rs (NATIVE_HOOKS_API 40). Use when writing or reviewing mods/<id>/native/mixins/<id>/*.rs, choosing which hook a mod feature needs, or logging an engine gap.
---

# horizon-hooks

## How hooks work

`horizon/crates/horizon-client/src/game/native_hooks.rs` defines `Game::native_*` methods (about 2,600 lines). `horizon-mixins` injects a mod's `native/mixins/<id>/` code at the head of those methods when `horizon-client` is built. Most hooks use **out-parameters**: a mod fills `&mut Option<…>`, sets `&mut bool` flags such as `claimed` or `ran`, or pushes onto a `&mut Vec<…>`.

Always read the real signature in `native_hooks.rs` before writing a hook. Examples from the current revision:

```rust
fn native_block_entity_ticks(&self, block: &str, ticks: &mut bool)
fn native_block_entity_tick(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity)
fn native_screen(&self, block: &str, screen: &mut Option<NativeScreen>)
fn native_tooltip(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>)
fn native_use_item(&mut self, stack: &ItemStack, claimed: &mut bool)
fn native_net_from_client(&mut self, player: u128, channel: &str, data: &[u8])
```

## The namespace rule

Every hook returns early unless the call is for this mod:

```rust
if !block.starts_with("<id>:") { return; }
```

Never set `claimed`, `ran` or an `Option` out-param for an id you do not own, and never clear a value another mod already set. Phase 2 of the harness audits this.

## Which hook for what

| API | Entrypoints | Typical use |
|---|---|---|
| 8–10 | `native_register_commands`, `native_perform_command`, `native_open_menu` | `/emc`, `/cc`, virtual chest menus |
| 12 | `native_tooltip` | EMC values, EU charge, frequency labels |
| 13–14 | `native_use_item_on`, `native_use_item`, `native_destroy_block`, `native_place_block` | wrenches, treetaps, Philosopher's Stone |
| 15 | `native_block_entity`, `native_block_entity_ticks` | attach and enable ticking block entities |
| 16–17 | `native_block_entity_container`, `native_block_entity_tick` | machine slots, hopper rules, server ticks |
| 18 | `native_with_mod_block_entity`, `native_set_block_property` | energy/item transfer between blocks, `lit`/`facing` |
| 19–20 | `native_screen`, `native_screen_draw`, `native_screen_click`, `native_screen_tooltip`, `native_menu_button`, `native_insert_item`, `native_block` | machine GUIs, auto-output |
| 21–22 | `native_right_click_block`, `native_block_spread`, `native_piston_move`, `native_explosion_detonate` | protection, reactor/engine explosions |
| 23–26 | `native_level_tick_pre/post`, `native_set_block`, `native_play_sound`, `native_config_dir` | quarries, frames, machine audio, config |
| 27–30 | `native_health_hud`, `native_block_in_tag`, `native_item_in_tag`, `native_interact_entity`, `native_left_click_block` | armor HUD, `#c:ores` instead of OreDictionary |
| 31–35 | `native_player_tick`, `native_update_shape`, `native_schedule_block_tick`, `native_random_tick`, `native_tick_block`, `native_commands` | jetpacks, cable/pipe shapes, crops, logic gate ticks |
| 36 | `native_recipes_of_type`, `native_recipe_ingredient`, `native_recipe_result` | custom machine recipe types |
| 37 | `native_spawn_mob`, `native_mob`, `native_mobs`, `native_teleport_mob`, `native_push_mob`, `native_set_mob_health`, `native_kill_mob` | interdiction, tesla coil, soul shards |
| 38 | `native_block_signal`, `native_is_signal_source`, `native_update_signal_neighbors` | logic gates, detector cables, CC redstone |
| 39 | `native_hurt_enemy`, `native_inventory_tick`, `native_carried_stack`, `native_on_equip` | talismans, Klein stars, nano saber |
| 40 | `native_net_send_to_player`, `native_net_send_to_server`, `native_net_from_client`, `native_net_from_server`, lifecycle events | terminals, ME terminal, 6502, tablet search (30 KB/msg) |

Source: `planning/workflow.md` §1. If this table and `native_hooks.rs` disagree, the code wins.

## Logging an engine gap

Never edit `horizon/`. An engine gap is a Beads bug labelled `horizon` (`bd list --label horizon` lists them). A role subagent puts it in its report as a follow_up; the conductor files it:

```json
{"title": "horizon: <feature> for <id>", "type": "bug",
 "why": "Needed for: spec section / registry entry. Missing: what the engine lacks (no hook, wrong signature...). Proposed hook: fn native_<name>(&mut self, ...). Workaround: what the mod does meanwhile, or deferred."}
```

The conductor files it with `bd create "horizon: ..." -t bug -l horizon,mod:<id> -d "..."`.
