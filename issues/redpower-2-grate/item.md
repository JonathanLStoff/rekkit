---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-grate
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-xnti
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Grate machine panel: suction intake, pressurized dispersal, BFS fluid pathfinding

Implements Grate panel (BlockMachinePanel meta 3): connects to fluid pipe network, performs BFS fluid search to suck source liquid blocks under suction into attached pipe network or pour liquid into world under positive pressure. Done: contract-check clean for grate; level 1, 3, 4 scenarios pass.

Size: ~150k tokens.
Covers: grate
Read first: spec 3.3 Power & Fluid Machines
Java: TileGrate#onTileTick, TileGrate#restartPath, TileGrate$GratePathfinder#trySuckFluid, TileGrate$GratePathfinder#tryDumpFluid, RenderGrate#renderWorldBlock
Files: mods/redpower_2/src/machine/grate.rs, planning/mods/redpower_2.contract/grate.json

## Beads import

- Beads id: `rk-xnti`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:42:57Z
- Close reason (Beads): Grate ported as pure logic in machine/grate.rs (TileGrate + GratePathfinder), 4 unit tests pass, clippy clean.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:42:56Z · Jonathan Stoff] [domain-builder-pro] done: Grate ported as pure logic in machine/grate.rs (TileGrate + GratePathfinder), 4 unit tests pass, clippy clean.
Files: mods/redpower_2/src/machine/grate.rs, mods/redpower_2/src/machine/mod.rs
Engine layer must implement GrateWorld (liquid_id, fluid_id_at, fluid_level, set_fluid_level, quanta, is_air, remove_block), call TileGrate::tick (front pressure, each tick), flow_to, and on_tile_tick_at every 5 ticks. Java NPE quirk: after restartPath inside trySuckFluid the original reads searchPath.fluidID from null; we capture the id first. Priority-queue ties broken by position for determinism. DEFERRED: block/blockstate/GUI-less registration and engine wiring.
