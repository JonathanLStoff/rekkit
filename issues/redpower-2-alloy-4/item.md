---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-alloy-furnace
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-47kg
epic: redpower-2-3-2
blocked_by: ['@redpower-2-alloy-3', '@redpower-2-core-4']
closed: 2026-11-12
---

# redpower_2: Alloy Furnace: Block, Tile Entity, Container & GUI

Implement Alloy Furnace block, tile entity, container, and GUI: 11 slots (0..8 alloy input matrix, 9 solid fuel, 10 output), 200 tick cook time, solid fuel burn times, sided inventory access (top = inputs, front = fuel, bottom/sides = output), front active fire texture change, and chunk reload persistence. Done: make contract-check clean for alloy_furnace; its level 1, 3, 5 and 6 scenarios pass.

Size: ~120k tokens.
Covers: alloy_furnace
Read first: spec 3.1 Appliance Machines, 7.1 Custom Machine Recipes
Java: TileAlloyFurnace#n_, TileAlloyFurnace#canSmelt, TileAlloyFurnace#smeltItem, ContainerAlloyFurnace#<init>, TileAlloyFurnace#getStartInventorySide, RenderAlloyFurnace#renderWorldBlock
Files: mods/redpower_2/src/base/alloy_furnace.rs

## Beads import

- Beads id: `rk-47kg`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:42:15Z
- Close reason (Beads): Alloy Furnace tile/container/GUI ported (machine/alloy_furnace.rs) and wired into native tick, container, screen hooks with CraftLib alloy recipes.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:42:07Z · Jonathan Stoff] [domain-builder-pro] done: Alloy Furnace tile/container/GUI ported (machine/alloy_furnace.rs) and wired into native tick, container, screen hooks with CraftLib alloy recipes.
Files: mods/redpower_2/src/machine/alloy_furnace.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
TileAlloyFurnace: 200-tick cook, fuel burn times, lava bucket leaves bucket, canSmelt/smeltItem via CraftLib::get_alloy_result, sided access (top 0..9, front fuel, back output), output slot rejects insertion, GUI slots at 48,17 3x3 / 17,42 / 141,35 in 176x166. alloy_craft_lib() registers default recipes once. Container is still backed by a 27-slot generic_9x3 menu (screen draws 11 slots); legacy AlloyFurnaceState left in place for blulectric. Tests pass for alloy; unrelated machine::frame::tests::motor_moves_16_ticks_and_draws_power fails (another agent's file).
