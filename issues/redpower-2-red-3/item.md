---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-wire-pathfinder
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-di8f
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-7']
closed: 2026-11-12
---

# redpower_2: Red alloy and bundled wire network BFS pathfinder

Implement wire graph traversal and flood-fill pathfinding (WirePathfinder). Resolves wire connectivity across surface faces, planar 90-degree corners, wrapped edge corners, and jacketed cable conduits. Propagates signal search fronts across connected wire clusters. Done: unit tests verify connected component pathfinding over complex corner transitions and jacketed columns matching RedPower wiring rules.

Size: ~110k tokens.
Covers: red_alloy_wiring
Read first: spec 2.2 Wiring System
Java: WirePathfinder#init, WirePathfinder#addSearchBlock, WirePathfinder#addIndBl, WirePathfinder#addSearchBlocks, WirePathfinder#iterate, WirePathfinder#step
Files: mods/redpower_2/src/wiring/pathfinder.rs

## Beads import

- Beads id: `rk-di8f`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:34:17Z
- Close reason (Beads): Ported RedPowerLib wire signal search (updateCurrent relaxation, getMaxCurrentStrength, getIndCur corner links, addSearchBlocks, updateBlockCurrentStrength) as a pure WireWorld for red alloy, insulate

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:34:14Z · Jonathan Stoff] [domain-builder] done: Ported RedPowerLib wire signal search (updateCurrent relaxation, getMaxCurrentStrength, getIndCur corner links, addSearchBlocks, updateBlockCurrentStrength) as a pure WireWorld for red alloy, insulated wire and 16-channel bundled cable.
Files: mods/redpower_2/src/wiring/network.rs, mods/redpower_2/src/wiring/mod.rs
cargo test wiring: 10 pass; clippy clean for network.rs. Channel 0 = red alloy, 1..=16 = colours; signal 255 at source, minus 1 per hop. External power is modelled as WireNode.powering per channel (stand-in for RedPowerLib.isPowered scan) and Tile::Source{powering_mask}. DEFERRED: connection-mask computation (getConnections/getConSides/getExtConSides) and isPowered/vanilla redstone scanning; engine wiring (world to WireWorld, block-update notification of returned positions).
