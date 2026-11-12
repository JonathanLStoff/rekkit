---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: open
priority: normal
labels:
- core
- for:redpower_2
- mod
- mod:rekkit_core
- path:crates/rekkit_core
- tier:Hard
- wave:core
beads_id: rk-p2g.8
epic: rekkit-core-3
---

# rekkit_core: HorizonNativeMod hooks native_random_tick, native_chunk_load/worldgen, native_hurt_enemy, native_place_block

The trait has none of these; the engine has native_random_tick (GameLevel), native_hurt_enemy (Game) and chunk load hooks, and the mixin helpers in native/mixins/redpower_2/*.rs (not under src/) are stubs. Core must expose the hooks, then lib.rs calls redpower_2::world_hooks::{flora_random_tick,chunk_features,melee_damage,ribbon_placement}.

## Beads import

- Beads id: `rk-p2g.8`
- Beads status: `open`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:49:55Z
