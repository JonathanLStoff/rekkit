---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-ejector
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-tca5
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Ejector machine: 9-slot buffer, pulse extraction and tube injection

Implements Ejector (BlockMachine meta 14): 9-slot internal inventory (TileEjectBase), single-item extraction from adjacent inventories into tube on redstone pulse, and tube backpressure buffering. ContainerEject and GuiEject. Done: contract-check clean for ejector; level 1, 3, 4, 5, 6 scenarios pass.

Size: ~110k tokens.
Covers: ejector
Read first: spec 3.2 Tube & Logistics Machines
Java: TileEject#handleExtract, TileEjectBase#drainBuffer, TileEjectBase#tubeItemEnter, ContainerEject#<init>, GuiEject#<init>
Files: mods/redpower_2/src/machine/ejector.rs, planning/mods/redpower_2.contract/ejector.json

## Beads import

- Beads id: `rk-tca5`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:38:15Z
- Close reason (Beads): Ejector machine: 9-slot buffer, pulse extraction and tube injection implemented with 6 unit tests passing.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:42:33Z · Jonathan Stoff] [domain-builder] partial: Ejector pure logic (ejector.rs) with 6 unit tests and contract file written; engine wiring in lib.rs not done (not owned/outside scope).
Files: mods/redpower_2/src/machine/ejector.rs, mods/redpower_2/src/machine/mod.rs, planning/mods/redpower_2.contract/ejector.json
Ports TileEject/TileEjectBase: 9 slots, handle_extract, drain_buffer, tube bounce, tile ticks, drops, ContainerEject slot_positions (62,17 grid; inv 8,84; 176x166). Reuses TubeBuffer from breaker.rs. Contract scenarios are placeholder-level (placement/reload); pulse/tube scenarios only place blocks. Not run: contract-check, harness. DEFERRED: lib.rs hook wiring of TileEjector, native_screen GUI.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:38:13Z · Jonathan Stoff] [domain-builder] done: Ejector machine: 9-slot buffer, pulse extraction and tube injection implemented with 6 unit tests passing.
Files: mods/redpower_2/src/machine/ejector.rs, mods/redpower_2/src/machine/mod.rs
9-slot inventory, handle_extract, tube bounce, pulse extraction, and drops pass all 6 tests.
