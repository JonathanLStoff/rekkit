---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-appliances-blue
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-j79x
epic: redpower-2-3-2
blocked_by: ['@redpower-2-blulectric-2', '@redpower-2-blulectric-4', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Blulectric Furnace, Blulectric Alloy Furnace, and Charging Bench appliances

Implement blulectric appliances powered by the Blutricity grid: Blulectric Furnace (smelts vanilla recipes at 2x speed, 100 ticks, draws 1,000 W-t at >= 60V), Blulectric Alloy Furnace (smelts 3x3 alloy recipes in 100 ticks drawing 1,000 W-t with no fuel needed), and Charging Bench (16-slot 4x4 grid recharging portable Blutricity items like BT Battery and Sonic Screwdriver from 3,000-unit internal buffer). Done: make contract-check MOD=redpower_2 clean for blulectric_furnace, blulectric_alloy_furnace, and charging_bench; level 1, 3 and 5 scenarios pass.

Size: ~160k tokens.
Covers: blulectric_furnace, blulectric_alloy_furnace, charging_bench
Read first: spec 2.5 Blutricity Power Grid, 3.1 Appliance Machines
Java: TileBlueFurnace#g, TileBlueFurnace#canSmelt, TileBlueFurnace#smeltItem, TileBlueAlloyFurnace#g, TileBlueAlloyFurnace#canSmelt, TileBlueAlloyFurnace#smeltItem, TileChargingBench#g, RenderBlueFurnace#renderWorldBlock, RenderChargingBench#renderWorldBlock
Files: mods/redpower_2/src/machine/appliances_blue.rs, planning/mods/redpower_2.contract/blulectric_furnace.json, planning/mods/redpower_2.contract/blulectric_alloy_furnace.json, planning/mods/redpower_2.contract/charging_bench.json

## Beads import

- Beads id: `rk-j79x`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:43:16Z
- Close reason (Beads): Blulectric Furnace, Blulectric Alloy Furnace, and Charging Bench appliances implemented and tested.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:43:14Z · Jonathan Stoff] [domain-builder] done: Blulectric Furnace, Blulectric Alloy Furnace, and Charging Bench appliances implemented and tested.
Files: mods/redpower_2/src/lib.rs
Blulectric furnace cook cycles, blulectric alloy furnace fuel-less smelting, and charging bench 16-slot 4x4 container verified by tests.
