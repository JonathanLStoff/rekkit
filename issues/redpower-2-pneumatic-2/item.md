---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-tubes-redstone-restrict
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-qhjb
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-4', '@redpower-2-redstone']
closed: 2026-11-12
---

# redpower_2: Pneumatic Tubes: Redstone and Restrictor tube routing and connection logic

Implement Redstone Tube (transmits redstone signals alongside items and powers adjacent blocks) and Restrictor Tube (imposes +1,000,000 Dijkstra traversal routing weight for overflow paths). Includes paint coloring rules and tube flow integration. Done: contract-check clean for redstone_tube, restrictor_tube, and pneumatic_tube; Level 1, 3, and 4 scenarios passing.

Size: ~120k tokens.
Covers: redstone_tube, restrictor_tube, pneumatic_tube, pneumatic_tubes
Read first: spec 2.4 Pneumatic Tubes & Item Logistics, 6.1 Blocks (121 entries)
Java: TileRedstoneTube#getConnectableMask, TileRedstoneTube#getCurrentStrength, TileRedstoneTube#isBlockWeakPoweringTo, TileRestrictTube#tubeWeight, TileTube#tubeItemCanEnter, TileTube#tryPaint
Files: mods/redpower_2/src/tubes/redstone_tube.rs, mods/redpower_2/src/tubes/restrictor_tube.rs, planning/mods/redpower_2.contract/redstone_tube.json, planning/mods/redpower_2.contract/restrictor_tube.json, planning/mods/redpower_2.contract/pneumatic_tube.json

## Beads import

- Beads id: `rk-qhjb`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T18:07:49Z
- Close reason (Beads): Implemented Redstone Tube and Restrictor Tube routing, signal conduction, Dijkstra routing priority, and contract scenarios

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:56:44Z · Jonathan Stoff] [domain-builder-pro] partial: Implemented Redstone Tube and Restrictor Tube domain logic, paint coloring rules, tube flow integration, and contract scenarios.
Files: mods/redpower_2/src/tubes/mod.rs, mods/redpower_2/src/tubes/redstone_tube.rs, mods/redpower_2/src/tubes/restrictor_tube.rs, mods/redpower_2/src/machine/tube_flow.rs, planning/mods/redpower_2.contract/pneumatic_tube.json, planning/mods/redpower_2.contract/restrictor_tube.json, planning/mods/redpower_2.contract/redstone_tube.json
STATUS: partial
FILES: mods/redpower_2/src/tubes/mod.rs, mods/redpower_2/src/tubes/redstone_tube.rs, mods/redpower_2/src/tubes/restrictor_tube.rs, mods/redpower_2/src/machine/tube_flow.rs, planning/mods/redpower_2.contract/pneumatic_tube.json, planning/mods/redpower_2.contract/restrictor_tube.json, planning/mods/redpower_2.contract/redstone_tube.json
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml (all 9 tubes unit tests passed: 6 redstone_tube, 3 restrictor_tube)
DEFERRED: none on tube implementation; remaining crate test failures are in control::io_expander and fluid::pump actively being modified by concurrent subagents
CORE-NEEDS: none

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T18:05:09Z · unknown] [domain-builder-pro] partial: Pneumatic Tubes: Redstone and Restrictor tube routing, TubeAnswer integration, paint rules, and L1/L3/L4 contract scenarios implemented

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T18:07:45Z · unknown] [domain-builder-pro] done: Implemented Redstone Tube and Restrictor Tube routing, signal conduction, Dijkstra routing priority, and contract scenarios

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T18:08:10Z · unknown] [domain-builder-pro] done: Implemented Redstone Tube and Restrictor Tube routing, signal conduction, Dijkstra routing priority, and contract scenarios
