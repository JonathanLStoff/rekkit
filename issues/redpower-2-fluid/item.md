---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-pipe-lib
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-rks9
epic: redpower-2-3-2
blocked_by: ['@redpower-2-3d']
closed: 2026-11-12
---

# redpower_2: Fluid Pipe Network Mechanics & Pressure Balancing: PipeLib

Port the RedPower fluid pipe library for Machine liquid transport. Manages pipe connection bitmasks, visual flange connections, fluid type registry (water, lava, and Forge mod fluids), pressure queries from pumps and grates (getPressure), and the iterative fluid transfer algorithm (movePipeLiquid) that balances liquid volumes and pressures between adjacent IPipeConnectable block entities. Done: unit tests pass for pipe connection masking, flange identification, fluid ID mapping, and pressure-driven liquid balancing.

Size: ~130k tokens.
Read first: spec 2. Core Systems & Mechanics, 3.3 Power & Fluid Machines
Java: PipeLib#movePipeLiquid, PipeLib#getConnections, PipeLib#getFlanges, PipeLib#getPressure, PipeLib#registerFluids, PipeLib#getLiquidClass
Files: mods/redpower_2/src/pipe/lib.rs

## Beads import

- Beads id: `rk-rks9`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:34:05Z
- Close reason (Beads): Fluid Pipe Network Mechanics & Pressure Balancing: PipeLib implemented in pipe.rs with unit tests passing.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:34:03Z · Jonathan Stoff] [domain-builder] done: Fluid Pipe Network Mechanics & Pressure Balancing: PipeLib implemented in pipe.rs with unit tests passing.
Files: mods/redpower_2/src/machine/pipe.rs
Pressure balancing, tank drain/fill, fluid mismatch prevention, and flange generation verified.
