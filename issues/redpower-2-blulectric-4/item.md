---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-blulectric-alloy-furnace
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-vqev
epic: redpower-2-3-2
blocked_by: ['@redpower-2-blulectric-2']
closed: 2026-11-12
---

# redpower_2: Blulectric alloy furnace: fuel-less 3x3 alloy smelting powered by Blutricity

Implements Blulectric Alloy Furnace (BlockAppliance meta 4): 3x3 input matrix (slots 0..8) and output slot 9, fuel-less smelting using Blutricity (>= 60V, 1,000 W-t/tick, 100 ticks per cook), ContainerBlueAlloyFurnace and GuiBlueAlloyFurnace with charge and flow indicators. Done: contract-check clean for blulectric_alloy_furnace; level 1, 3, 5, 6 scenarios pass.

Size: ~150k tokens.
Covers: blulectric_alloy_furnace
Read first: spec 3.1 Appliance Machines, 7.1 Custom Machine Recipes, 2.5 Blutricity Power Grid
Java: TileBlueAlloyFurnace#canSmelt, TileBlueAlloyFurnace#smeltItem, ContainerBlueAlloyFurnace#<init>, GuiBlueAlloyFurnace#<init>
Files: mods/redpower_2/src/machine/blue_alloy_furnace.rs, planning/mods/redpower_2.contract/blulectric_alloy_furnace.json

## Beads import

- Beads id: `rk-vqev`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:41:25Z
- Close reason (Beads): Blulectric alloy furnace: fuel-less 3x3 alloy smelting powered by Blutricity implemented and tested.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-04T03:41:24Z · Jonathan Stoff] [domain-builder-pro] done: Blulectric alloy furnace: fuel-less 3x3 alloy smelting powered by Blutricity implemented and tested.
Files: mods/redpower_2/src/lib.rs
AlloyFurnaceState BLULECTRIC_SMELT_TICKS operation without solid fuel verified by integration test test_alloy_furnace_recipes_solid_fuel_and_blulectric.
