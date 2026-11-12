---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-fluids
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-vksm
epic: redpower-2-3-2
blocked_by: ['@redpower-2-fluid-3', '@redpower-2-machine-2', '@redpower-2-grate', '@redpower-2-fluid-5']
closed: 2026-11-12
---

# redpower_2: Fluid Pipe, Blulectric Pump, and Grate intake/drainage

Implement RedPower fluid transport mechanics: Fluid Pipe (TilePipe, pressure-driven flow, flanges, and covers), Blulectric Pump (TilePump, creates +1000 outlet and -1000 inlet pressure, pumping up to 1 bucket/tick at >= 60V Blutricity), and Grate (TileGrate with GratePathfinder: breadth-first flood search to suck liquid blocks from the world into pipes or dump piped liquid into open pools). Done: make contract-check MOD=redpower_2 clean for pipe, pump, and grate; level 1, 3, 4, 5 scenarios pass.

Size: ~190k tokens.
Covers: pipe, pump, grate
Read first: spec 3.3 Power & Fluid Machines
Java: TilePipe#g, TilePipe#cacheCon, TilePump#g, TilePump#pumpFluid, TileGrate#g, TileGrate#onTileTick, GratePathfinder#trySuckFluid, GratePathfinder#tryDumpFluid, RenderPipe#renderWorldBlock, RenderPump#renderWorldBlock, RenderGrate#renderWorldBlock, TilePipeRenderer#a, TilePumpRenderer#a
Files: mods/redpower_2/src/machine/fluids.rs, planning/mods/redpower_2.contract/pipe.json, planning/mods/redpower_2.contract/pump.json, planning/mods/redpower_2.contract/grate.json

## Beads import

- Beads id: `rk-vksm`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:23:42Z
- Close reason (Beads): Implemented unified RedPower fluid transport mechanics (Fluid Pipe, Blulectric Pump, Grate) in mods/redpower_2/src/machine/fluids.rs with pressure propagation, flow simulation, and chunk persistence.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T18:23:53Z · unknown] [domain-builder-pro] partial: Verified pipe, pump, grate contracts and domain models; ready for fluids.rs unified simulation

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-08T02:23:40Z · unknown] [domain-builder-pro] done: Implemented unified RedPower fluid transport mechanics (Fluid Pipe, Blulectric Pump, Grate) in mods/redpower_2/src/machine/fluids.rs with pressure propagation, flow simulation, and chunk persistence.
