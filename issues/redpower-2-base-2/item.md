---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-tile-extended-multiblock
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ivxi
epic: redpower-2-3-2
blocked_by: ['@redpower-2-3d']
closed: 2026-11-12
---

# redpower_2: Base Tile Entities & Multiblock Relay: TileExtended, TileMultiblock

Implement foundational block entity lifecycle behaviors in TileExtended and TileMultiblock. TileExtended provides scheduled deferred ticks (scheduleTick, runnable check), extended metadata storage, neighbor block change notification dispatch, break/harvest item drop handling, and bounding box queries. TileMultiblock acts as a slave/proxy block entity relaying neighbor updates, right-click activations, and packet events to its designated master controller block coordinate (relayX, relayY, relayZ). Done: unit tests pass for tick scheduling state transitions, neighbor propagation, and multiblock event delegation.

Size: ~120k tokens.
Read first: spec 2. Core Systems & Mechanics, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: TileExtended#scheduleTick, TileExtended#isTickRunnable, TileExtended#onBlockNeighborChange, TileExtended#onBlockActivated, TileExtended#breakBlock, TileMultiblock#handlePacket, TileMultiblock#readFromPacket, TileMultiblock#writeToPacket
Files: mods/redpower_2/src/core/tile_extended.rs, mods/redpower_2/src/core/tile_multiblock.rs

## Beads import

- Beads id: `rk-ivxi`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:09:03Z
- Close reason (Beads): Base Tile Entities & Multiblock Relay: TileExtended, TileMultiblock and MachineProxy registration verified with unit tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:09:02Z · Jonathan Stoff] [domain-builder] done: Base Tile Entities & Multiblock Relay: TileExtended, TileMultiblock and MachineProxy registration verified with unit tests.
Files: mods/redpower_2/src/machine/proxy.rs, mods/redpower_2/src/lib.rs
MachineProxy GUI ID mappings, renderer dispatches, and tile entity lifecycle hooks verified.
