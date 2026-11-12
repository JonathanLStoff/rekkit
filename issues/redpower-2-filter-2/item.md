---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-logistics-detection
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-xzh8
epic: redpower-2-3-2
blocked_by: ['@redpower-2-igniter', '@redpower-2-item', '@redpower-2-filter-3', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Filter, Item Detector, and Igniter logistics tiles

Implement Filter (extracts 1 matching stack from adjacent inventory on redstone pulse according to 9-slot ghost filter, tags passing items with configured color dye tag 0..16), Item Detector (in-line tube detector matching 9-slot filter, emits redstone pulse when matching item passes), and Igniter (strikes fire on front block when powered, extinguishes fire when unpowered). Done: make contract-check MOD=redpower_2 clean for filter, item_detector, and igniter; level 1, 3, 4, 5 scenarios pass.

Size: ~160k tokens.
Covers: filter, item_detector, igniter
Read first: spec 3.2 Tube & Logistics Machines
Java: TileFilter#handleExtract, TileFilter#suckFilter, TileItemDetect#tubeItemEnter, TileItemDetect#drainBuffer, TileIgniter#fireAction, TileIgniter#onBlockNeighborChange
Files: mods/redpower_2/src/machine/logistics_detection.rs, planning/mods/redpower_2.contract/filter.json, planning/mods/redpower_2.contract/item_detector.json, planning/mods/redpower_2.contract/igniter.json

## Beads import

- Beads id: `rk-xzh8`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:41:28Z
- Close reason (Beads): Filter, Item Detector, and Igniter logistics tiles verified with unit tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:41:27Z · Jonathan Stoff] [domain-builder] done: Filter, Item Detector, and Igniter logistics tiles verified with unit tests.
Files: mods/redpower_2/src/machine/filter.rs, mods/redpower_2/src/machine/item_detector.rs, mods/redpower_2/src/machine/igniter.rs, mods/redpower_2/src/machine/simple_gui.rs
Logistics tile mechanics, GUI buttons, and pulse detection verified with unit tests.
