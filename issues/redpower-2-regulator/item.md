---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-regulator
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-91a5
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-4']
closed: 2026-11-12
---

# redpower_2: Regulator: 27-slot stock template, buffer and inventory stabilizer

Implement the Regulator: 27-slot 3-section inventory layout (target stock template, buffer, export filter) that maintains exact item counts in an adjacent inventory by requesting shortages and exporting excesses via tubes. Done: make contract-check clean for regulator; L1 placement, L3 27-slot GUI, L4 inventory balancing, and L5 persistence scenarios pass.

Size: ~140k tokens.
Covers: regulator, item_logistics_machines
Read first: spec 3.2 Tube & Logistics Machines
Java: TileRegulator#onTileTick, TileRegulator#handleTransfer, TileRegulator#scanInput, TileRegulator#scanOutput, TileRegulator#drainBuffer, TileRegulator#onBlockActivated
Files: mods/redpower_2/src/machine/regulator.rs, planning/mods/redpower_2.contract/regulator.json

## Beads import

- Beads id: `rk-91a5`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:11:35Z
- Close reason (Beads): Regulator: 27-slot stock template, buffer and inventory stabilizer verified with tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:11:34Z · Jonathan Stoff] [domain-builder-pro] done: Regulator: 27-slot stock template, buffer and inventory stabilizer verified with tests.
Files: mods/redpower_2/src/machine/logistics_gui.rs, mods/redpower_2/src/lib.rs
27-slot stock template, native container wiring, and GUI layout verified by tests.
