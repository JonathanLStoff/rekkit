---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-core-scenarios
- mod
- mod:redpower_2
- needs-split
- needs:rekkit_core
- path:mods/redpower_2
- role:test-smith
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-gn3a
epic: redpower-2-3-2
blocked_by: ['@redpower-2-redpowercore']
closed: 2026-11-12
---

# redpower_2: Core subsystem integration tests and verification scenarios

Automated scenario tests validating microblock compound placement, pneumatic tube Dijkstra routing with restrictor tubes (+1,000,000 weight) and color tags, and red alloy wire pathfinding across corner boundaries. Done: all scenarios pass at Level 3 and Level 4 verification.

Size: ~100k tokens.
Read first: spec 2.1 Microblocks & Sub-Voxel System, 2.2 Wiring System, 2.4 Pneumatic Tubes & Item Logistics
Java: TileCovered#canAddCover, TubeLib#findRoute, WirePathfinder#iterate
Files: mods/redpower_2/tests/core_systems.rs

## Beads import

- Beads id: `rk-gn3a`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-08T03:29:24Z
- Close reason (Beads): Core subsystem integration tests and verification scenarios implemented in tests/core_systems.rs.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T13:11:56Z · unknown] [test-smith] partial: Token budget reached during brief analysis; ready to write mods/redpower_2/tests/core_systems.rs

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T13:14:08Z · unknown] [test-smith] partial: Core subsystem integration tests analyzed; stopped at token budget limit.

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-07T13:14:14Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-07T13:14:15Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-08T03:29:06Z · unknown] [test-smith] complete: Implemented automated scenario tests in tests/core_systems.rs validating microblock compound placement and sub-voxel occupancy bitmasks, pneumatic tube Dijkstra shortest-path routing with restrictor tubes (+1,000,000 weight penalty) and color tags, and red alloy wire pathfinding, signal decay, and 16-channel bundled cable multiplexing.
