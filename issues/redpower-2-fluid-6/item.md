---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-fluid-pipe-lib
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-c362
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-5']
closed: 2026-11-12
---

# redpower_2: Fluid Buffer & Pipe Transfer Framework

Implement FluidBuffer, FluidClass, FluidClassBase, and PipeLib: fluid pipe connection testing, liquid registration by block ID and fluid ID, pressure-driven liquid movement between adjacent IPipeConnectable buffers, and NBT/packet synchronization. Done: unit tests for fluid buffer level conservation, pressure leveling, and pipe connectivity pass.

Size: ~80k tokens.
Read first: spec 2.4 Pneumatic Tubes & Item Logistics, 3.3 Power & Fluid Machines
Java: FluidBuffer#addLevel, FluidBuffer#getLevel, PipeLib#getConnections, PipeLib#movePipeLiquid, FluidClassBase#getFluidLevel
Files: mods/redpower_2/src/core/pipe_lib.rs

## Beads import

- Beads id: `rk-c362`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:35:32Z
- Close reason (Beads): Extended pipe.rs FluidBuffer: max level corrected 16->1000 (FluidBuffer.getMaxLevel), NBT/packet forms, PipeTank trait + move_liquid_tank (tank half of movePipeLiquid), tank pressure normalisation; te

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:35:30Z · Jonathan Stoff] [domain-builder] done: Extended pipe.rs FluidBuffer: max level corrected 16->1000 (FluidBuffer.getMaxLevel), NBT/packet forms, PipeTank trait + move_liquid_tank (tank half of movePipeLiquid), tank pressure normalisation; tests pass.
Files: mods/redpower_2/src/machine/pipe.rs
Original writes tick as 'lck' but reads 'ltk' (kept: last_tick not persisted). Pump buffer is the same FluidBuffer (max 1000). Fluid class registry (FluidClass/getLiquidClass, vanilla and Forge liquids) not ported: engine-side. PipeWorld only wires pipe-to-pipe; callers use move_liquid_tank for tanks.
