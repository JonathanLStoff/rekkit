---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-sorting-machine
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-9sm1
epic: redpower-2-3-2
blocked_by: ['@redpower-2-sorting-3', '@redpower-2-logistics-3', '@redpower-2-routing']
closed: 2026-11-12
---

# redpower_2: Sorting Machine: 8-channel routing, operational modes, color tagging & GUI

Implement Sorting Machine with 40-slot filter inventory (8 columns x 5 rows), 8-channel color selector buttons, operational routing modes (Any, Sequential, Full Stack, All Items, Even Distribution, Random), Blutricity automatic extraction, and GUI networking event sync. Done: contract-check clean for sorting_machine; Level 1, 3, 4, 5 scenarios passing.

Size: ~140k tokens.
Covers: sorting_machine
Read first: spec 3.2 Tube & Logistics Machines, 6.1 Blocks (121 entries)
Java: TileSorter#handleExtract, TileSorter#tubeItemEnter, TileSorter#stepColumn, ContainerSorter#handleGuiEvent, GuiSorter#actionPerformed
Files: mods/redpower_2/src/machines/sorter.rs, planning/mods/redpower_2.contract/sorting_machine.json

## Beads import

- Beads id: `rk-9sm1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:19:58Z
- Close reason (Beads): Implemented Sorting Machine domain state machine with 40-slot filter inventory, 8-channel color routing, 7 operational modes, Blutricity power draw, and GUI event handling.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T02:19:56Z · unknown] [domain-builder-pro] done: Implemented Sorting Machine domain state machine with 40-slot filter inventory, 8-channel color routing, 7 operational modes, Blutricity power draw, and GUI event handling.
