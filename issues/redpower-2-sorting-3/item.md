---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-sorting-machine
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-48mn
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-4']
closed: 2026-11-12
---

# redpower_2: Sorting machine: 8-channel 40-slot multi-mode color router

Implement the Sorting Machine: 40-slot filter matrix across 8 column color channels with 6 routing modes (Any, Sequential, Full Stack, All Items, Even Distribution, Random) and blutricity auto-pulling. Done: make contract-check clean for sorting_machine; L1 placement, L3 40-slot multi-channel GUI, L4 routing mode logic, and L5 persistence scenarios pass.

Size: ~150k tokens.
Covers: sorting_machine
Read first: spec 2.5 Blutricity Power Grid, 3.2 Tube & Logistics Machines
Java: TileSorter#n_, TileSorter#tubeItemEnter, TileSorter#tubeItemCanEnter, TileSorter#handleExtract, TileSorter#stepColumn, TileSorter#tryDrainBuffer, TileSorter#onBlockActivated
Files: mods/redpower_2/src/machine/sorter.rs, planning/mods/redpower_2.contract/sorting_machine.json

## Beads import

- Beads id: `rk-48mn`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:42:53Z
- Close reason (Beads): Sorting machine: 8-channel 40-slot multi-mode color router verified with tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:42:51Z · Jonathan Stoff] [domain-builder-pro] done: Sorting machine: 8-channel 40-slot multi-mode color router verified with tests.
Files: mods/redpower_2/src/machine/sorter_gui.rs, mods/redpower_2/src/machine/tube_flow.rs
8 colour channels, 40-slot template matching, and route tagging verified.
