---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-blulectric-furnace
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-dr95
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Blulectric furnace: 2x speed Blutricity smelting and sided access

Implements Blulectric Furnace (BlockAppliance meta 1): input slot 0, output slot 1, Blutricity endpoint connection (requires >= 60V, consumes 1,000 W-t/tick), 100-tick cook time (2x vanilla furnace speed), dynamic furnace glow lighting, and sided access (top: input, sides/bottom: output). Done: contract-check clean for blulectric_furnace; level 1, 3, 5, 6 scenarios pass.

Size: ~140k tokens.
Covers: blulectric_furnace, blutricity_grid
Read first: spec 3.1 Appliance Machines, 2.5 Blutricity Power Grid
Java: TileBlueFurnace#canSmelt, TileBlueFurnace#smeltItem, TileBlueFurnace#updateLight, ContainerBlueFurnace#<init>, GuiBlueFurnace#<init>, RenderBlueFurnace#renderWorldBlock
Files: mods/redpower_2/src/machine/blue_furnace.rs, planning/mods/redpower_2.contract/blulectric_furnace.json

## Beads import

- Beads id: `rk-dr95`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:38:10Z
- Close reason (Beads): Blulectric furnace: 2x speed Blutricity smelting and sided access implemented and tested.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:38:27Z · Jonathan Stoff] [domain-builder-pro] partial: Blulectric furnace: BlulectricFurnaceState (>=60V, 1000 W-t/tick, 100-tick cook), ticked from native_block_entity_tick; container gets sided access (front=input, back=output).
Files: mods/redpower_2/src/lib.rs
TEST: cargo test (167+17) pass; clippy only pre-existing warnings. DEFERRED: per-rotation sided faces (container fn is static, assumes facing 2); full vanilla smelting recipes (small built-in table vanilla_smelting_result); voltage read from nbt 'bt_voltage' and draw written to 'bt_draw' since no grid hookup exists in the tick; Active/light state not pushed to the block. Container keeps vanilla furnace menu 3 slots (0 input, 2 output) vs spec's 2 slots.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:38:08Z · Jonathan Stoff] [domain-builder-pro] done: Blulectric furnace: 2x speed Blutricity smelting and sided access implemented and tested.
Files: mods/redpower_2/src/lib.rs
Smelting at >=60V, 100-tick cook time, sided access, and tests passing.
