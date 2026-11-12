---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-fluid-framework
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-dn5i
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-3']
closed: 2026-11-12
---

# redpower_2: Fluid system: FluidBuffer, FluidClass, and pipe connection traits

Implement fluid system types and buffer management matching FluidBuffer, FluidClass, and IPipeConnectable. Support liquid transfer, pressure levels, side connectivity rules, and container draining/filling. Done: unit tests for fluid buffer capacity limits, drainage, and class registration pass.

Size: ~80k tokens.
Read first: spec Section 3.3 Power & Fluid Machines
Java: IPipeConnectable#getPipeConnectableSides, FluidBuffer#addFluid, FluidBuffer#drainFluid, FluidClass#getFluidId, FluidClassVanilla#register, FluidClassItem#getEmptyItem
Files: mods/redpower_2/src/core/fluid.rs

## Beads import

- Beads id: `rk-dn5i`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:33:59Z
- Close reason (Beads): Fluid system: FluidBuffer, FluidClass, and pipe connection traits implemented with pressure rules and 9 unit tests passing.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:33:56Z · Jonathan Stoff] [domain-builder] done: Fluid system: FluidBuffer, FluidClass, and pipe connection traits implemented with pressure rules and 9 unit tests passing.
Files: mods/redpower_2/src/machine/pipe.rs
Pressure propagation, buffer delta deferrals, liquid levelling, and connection rules pass all 9 unit tests.
