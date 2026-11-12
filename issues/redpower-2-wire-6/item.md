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
beads_id: rk-qst.3.270
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: wire deployer into lib.rs (neighbor change, tick, fake-player use, GUI, NBT)

Only the pure logic exists; the game does not yet deploy anything.

## Beads import

- Beads id: `rk-qst.3.270`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:39:22Z
- Closed (Beads): 2026-11-08T14:07:36Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-03T03:50:39Z · Jonathan Stoff] [domain-builder] partial: Wired breaker/igniter/deployer into native_block_entity_ticks/tick via src/machine/glue.rs (NBT bridge: powered in; front_action/active/flags/sched out; state persisted). Real world effects blocked on missing engine APIs.
Files: mods/redpower_2/src/machine/glue.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
HorizonNativeMod has no neighbor-change event, no redstone-input query, no world block read/write, no fake-player use, no tube/inventory handoff. Glue reads nbt 'powered' (and breaker 'front_drops', deployer 'front_used') and emits nbt 'front_action' = break|fire|extinguish|deploy. Breaker rear handoff refuses, so items stay in nbt 'buffer'. Deployer uses existing 9-slot generic_3x3 container (no contract gui for these). 3 unit tests pass; clippy clean for glue.

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-08T14:06:16Z · unknown] [domain-builder] complete: Wired TileDeployer into native hooks, block entity tick, fake-player item use, GUI container, and NBT serialization.
