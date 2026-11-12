---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-logic-foundation
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-u508
epic: redpower-2-3-2
blocked_by: ['@redpower-2-logic', '@redpower-2-logic-2']
closed: 2026-11-12
---

# redpower_2: Logic tile substrate, placement, orientation, screwdriver interaction, and legacy converter shims

Implement the core logic tile infrastructure (BlockLogic, ItemLogic, TileLogic) and 1.2.5 legacy conversion shims (TileConverter for TileAnd, TileOr, TileNand, TileNor, TileXor, TileXnor, TilePulse, TileToggle, TileLatch, TileNot, TileBuffer, TileMux, TileRepeater, TileCounter, TileTimer, TileSequencer): 1/8-block sub-voxel flat bounding box ([0, 0, 0, 16, 2, 16]), 24 valid rotations (6 mounting faces x 4 rotations), right-click screwdriver rotation/cycling, cover plate attachment and removal, neighbor block update propagation, and redstone connection masking. Done: contract-check clean for logic_gate; level 1 placement/hitbox and level 2 screwdriver interaction scenarios pass.

Size: ~120k tokens.
Covers: integrated_logic, logic_gate
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks (121 entries)
Java: BlockLogic#canConnectRedstone, BlockLogic#getLightValue, ItemLogic#tryPlace, ItemLogic#itemUseShared, TileLogic#getPartRotation, TileLogic#setPartRotation, TileLogic#getConnectableMask, TileLogic#canAddCover, TileLogic#tryAddCover, TileLogic#tryRemoveCover, TileConverter#n_
Files: mods/redpower_2/src/logic/mod.rs, mods/redpower_2/src/logic/tile.rs, mods/redpower_2/src/logic/converter.rs, planning/mods/redpower_2.contract/logic_gate.json

## Beads import

- Beads id: `rk-u508`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:05:01Z
- Close reason (Beads): Logic tile substrate, placement, orientation, screwdriver interaction, and legacy converter shims verified.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:04:59Z · Jonathan Stoff] [domain-builder] done: Logic tile substrate, placement, orientation, screwdriver interaction, and legacy converter shims verified.
Files: mods/redpower_2/src/lib.rs
Logic tile substrate placement, orientation rotations, and screwdriver interaction verified.
