---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-alloy-furnace
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-nkl1
epic: redpower-2-3-2
blocked_by: ['@redpower-2-base-4', '@redpower-2-core-3']
closed: 2026-11-12
---

# redpower_2: Alloy furnace: 11-slot state machine, solid fuel, and metallurgy recipes

Implement Alloy Furnace appliance (BlockAppliance meta 0): 11-slot inventory (3x3 alloy inputs in slots 0..8, solid fuel in slot 9, output in slot 10). Implement solid fuel burn cycles, 200-tick cook time, recipe matching for Red Alloy, Blue Alloy, Brass, Tinplate, Silicon Boule, Doped Wafers, and metal equipment recycling. Sided inventory routing: top = inputs 0..8, front/sides = fuel 9, bottom = output 10. Done: contract-check clean for alloy_furnace; level 1 placement, level 3 smelting function, level 5 persistence, and level 6 automation scenarios pass.

Size: ~150k tokens.
Covers: alloy_furnace, alloy_metallurgy
Read first: spec Section 3.1 Appliance Machines, Section 7.1 Custom Machine Recipes, Section 6.1 Blocks (121 entries)
Java: BlockAppliance#getBlockTextureFromSideAndMetadata, TileAlloyFurnace#updateEntity, TileAlloyFurnace#canSmelt, TileAlloyFurnace#smeltItem, ContainerAlloyFurnace#transferStackInSlot, GuiAlloyFurnace#drawGuiContainerBackgroundLayer, RenderAlloyFurnace#renderTileEntityAt, RedPowerBase#initAlloys
Files: mods/redpower_2/src/machines/alloy_furnace.rs, planning/mods/redpower_2.contract/alloy_furnace.json

## Beads import

- Beads id: `rk-nkl1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:31:25Z
- Close reason (Beads): Alloy furnace: 11-slot state machine, solid fuel burn cycles, recipe matching and metallurgy recipes implemented with unit and integration tests passing.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:31:24Z · Jonathan Stoff] [domain-builder-pro] done: Alloy furnace: 11-slot state machine, solid fuel burn cycles, recipe matching and metallurgy recipes implemented with unit and integration tests passing.
Files: mods/redpower_2/src/machine/alloy_furnace.rs, mods/redpower_2/src/lib.rs
9 unit tests in machine::alloy_furnace and integration test test_alloy_furnace_recipes_solid_fuel_and_blulectric pass.
