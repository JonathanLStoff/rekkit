---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part08-wiring-core
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-t4zf
epic: redpower-2-3-2
blocked_by: ['@redpower-2-bluewire']
closed: 2026-11-12
---

# redpower_2: Wiring Subsystem: TileWiring, sub-voxel faces, connection masks & placement

Implement base TileWiring sub-voxel logic and MicroPlacementWire for mounting wires on 6 block faces (ConSides 0..5). Compute 24-direction connection masks, edge corner traversal, solid face support checking, and part bounding boxes (0.125 height). Integrate with microblock cover occlusion masks (TileCovered). Done: connection mask bit arithmetic matches TileWiring/RedPowerLib; wire placement on walls, floors, and ceilings passes Level 1 hitbox and Level 4 connection tests.

Size: ~140k tokens.
Covers: red_alloy_wiring
Read first: spec 2.1 Microblocks & Sub-Voxel System, 2.2 Wiring System
Java: TileWiring#getConnectableMask, TileWiring#getConnectionMask, TileWiring#getExtConnectionMask, TileWiring#setPartBounds, TileWiring#refreshBlockSupport, MicroPlacementWire#onPlaceMicro, MicroPlacementWire#initialPlace
Files: mods/redpower_2/src/wiring/mod.rs, mods/redpower_2/src/wiring/tile_wiring.rs, mods/redpower_2/src/wiring/placement.rs

## Beads import

- Beads id: `rk-t4zf`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:48:02Z
- Close reason (Beads): Wiring Subsystem: TileWiring, sub-voxel faces, connection masks & placement verified with 26 unit tests.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:47:59Z · Jonathan Stoff] [domain-builder-pro] done: Wiring Subsystem: TileWiring, sub-voxel faces, connection masks & placement verified with 26 unit tests.
Files: mods/redpower_2/src/wiring/mod.rs, mods/redpower_2/src/wiring/rules.rs, mods/redpower_2/src/wiring/network.rs
Sub-voxel faces, connection masks, corner connections, and signal conduction verified by wiring unit tests.
