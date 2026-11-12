---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-array-logic
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-5wyt
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pulse', '@redpower-2-logic-3']
closed: 2026-11-12
---

# redpower_2: Array logic cells (Null Cell, Invert Cell, Non-Invert Cell)

Implement array logic cells (TileLogicArray): Null Cell (passive pass-through spacer with top wire crossing), Invert Cell (inverting buffer cell connecting top/bottom signals), and Non-Invert Cell (non-inverting buffer cell). Support orthogonal signal propagation without cross-axis interference and integrated topwire connections. Done: contract-check clean for null_cell, invert_cell, non_invert_cell; level 4 array signal routing scenarios pass.

Size: ~110k tokens.
Covers: null_cell, invert_cell, non_invert_cell
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks (121 entries)
Java: TileLogicArray#updatePowerState, TileLogicArray#getTopwireMask, TileLogicArray#onBlockNeighborChange, RenderLogicArray#isArrayTopwire
Files: mods/redpower_2/src/logic/array.rs, planning/mods/redpower_2.contract/null_cell.json, planning/mods/redpower_2.contract/invert_cell.json, planning/mods/redpower_2.contract/non_invert_cell.json

## Beads import

- Beads id: `rk-5wyt`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:08:51Z
- Close reason (Beads): Array logic cells (Null Cell, Invert Cell, Non-Invert Cell) verified with block definitions and tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:08:50Z · Jonathan Stoff] [domain-builder] done: Array logic cells (Null Cell, Invert Cell, Non-Invert Cell) verified with block definitions and tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/content/data/redpower_2/horizon/block/null_cell.json, mods/redpower_2/content/data/redpower_2/horizon/block/invert_cell.json, mods/redpower_2/content/data/redpower_2/horizon/block/non_invert_cell.json
Cell definitions, top/bottom wiring pass-through, and array cell state stepping verified.
