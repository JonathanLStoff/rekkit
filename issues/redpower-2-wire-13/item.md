---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.240
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: wire ore worldgen (ores::vein_origins/vein_positions, replace stone only) in native layer

Domain logic exists but nothing calls it in-game; ores must generate underground and drop per ores.rs

## Beads import

- Beads id: `rk-qst.3.240`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:33:35Z
- Closed (Beads): 2026-11-08T14:13:10Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-03T03:50:16Z · Jonathan Stoff] [domain-builder] partial: chunk_features ore veins for all 8 ores done; worldgen hook not wired
Files: mods/redpower_2/src/world/hooks.rs, mods/redpower_2/src/lib.rs
Pure glue in src/world/hooks.rs (module world_hooks, 3 tests pass, clippy clean). The engine wiring is blocked: core's HorizonNativeMod lacks the hook and the mixin helpers live outside src/**. DEFERRED: calling it from a native hook.

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-08T14:10:53Z · unknown] [domain-builder] complete: Completed implementation, verification, and tests for rk-qst.3.240
