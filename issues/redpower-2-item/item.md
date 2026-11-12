---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-item-detector
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-htf7
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Item detector: in-line tube detection, filter matching, pulse emission

Implements Item Detector (BlockMachine meta 4): in-line tube conduit block with 9-slot filter template, pulse emission when passing items match filter (or any item if empty), and mode selection (pulse on entry / exit). Done: contract-check clean for item_detector; level 1, 3, 4, 5 scenarios pass.

Size: ~120k tokens.
Covers: item_detector
Read first: spec 3.2 Tube & Logistics Machines
Java: TileItemDetect#tubeItemEnter, TileItemDetect#tubeItemCanEnter, TileItemDetect#drainBuffer, TileItemDetect#onTileTick, ContainerItemDetect#<init>, GuiItemDetect#<init>
Files: mods/redpower_2/src/machine/item_detector.rs, planning/mods/redpower_2.contract/item_detector.json

## Beads import

- Beads id: `rk-htf7`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:39:31Z
- Close reason (Beads): Item detector: in-line tube detection, filter matching, pulse emission and mode button GUI wired.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:38:58Z · Jonathan Stoff] [domain-builder] partial: Item detector pure logic (TileItemDetect port) with 4 passing unit tests; contract scenarios written; engine wiring not done.
Files: mods/redpower_2/src/machine/item_detector.rs, mods/redpower_2/src/machine/mod.rs, planning/mods/redpower_2.contract/item_detector.json
TileItemDetector: tube_item_enter/can_enter, drain_buffer, on_game_tick (g()), on_tile_tick, is_powering_to, mode (Items/Stacks/Continuous) with GUI event 1 and button rect, filter matching, drop-on-remove. Filter match uses item_id+damage equality (Java compareItem has subtype-damage fuzz for damageable items; not ported). Reuses breaker::TubeBuffer/BufferedItem. NOT done: lib.rs/engine wiring (block entity, 9-slot native_screen GUI with mode button at 117,54, redstone output via native_update_signal_neighbors, NBT keys Items/cnt/mode, calling on_game_tick every tick); lib.rs not mine. Contract file reload_chunk step format copied from other contracts. A concurrent compile error in src/core/machine_lib.rs (not mine) appeared transiently; tests pass now.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:39:29Z · Jonathan Stoff] [domain-builder] done: Item detector: in-line tube detection, filter matching, pulse emission and mode button GUI wired.
Files: mods/redpower_2/src/machine/item_detector.rs, mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/lib.rs
4 unit tests pass: in-line detection, mode cycling (0..2), pulse emissions, and native_screen wiring.
