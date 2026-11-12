---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-logic-storage-and-bus
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-qpzq
epic: redpower-2-3-2
blocked_by: ['@redpower-2-logic']
closed: 2026-11-12
---

# redpower_2: Pulse Counter with GUI, Array Cells, and 16-channel Bus Transceiver

Implement Counter logic tile (pulse counting with increment, decrement, max count, GUI configuration, click sound), Array logic cells (Null Cell, Invert Cell, Non-Invert Cell pass-through), and 16-channel Bus Transceiver interfacing Bundled Cables. Done: contract-check clean for covered tiles; level 1, 3 and 4 counter/bus scenarios pass.

Size: ~120k tokens.
Covers: counter, null_cell, invert_cell, non_invert_cell, bus_transceiver
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks
Java: TileLogicStorage#onBlockNeighborChange, ContainerCounter#handleGuiEvent, GuiCounter#changeCountMax, TileLogicArray#scanPoweringStrength, TileLogicAdv#getCurrentStrength, RenderLogicStorage#renderWorldBlock, RenderLogicArray#renderWorldBlock, RenderLogicAdv#renderWorldBlock
Files: mods/redpower_2/src/logic/storage.rs, mods/redpower_2/src/logic/bus.rs, planning/mods/redpower_2.contract/counter.json, planning/mods/redpower_2.contract/null_cell.json, planning/mods/redpower_2.contract/bus_transceiver.json

## Beads import

- Beads id: `rk-qpzq`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:47:46Z
- Close reason (Beads): Pulse Counter with GUI, Array Cells, and 16-channel Bus Transceiver verified with unit and integration tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:47:43Z · Jonathan Stoff] [domain-builder-pro] done: Pulse Counter with GUI, Array Cells, and 16-channel Bus Transceiver verified with unit and integration tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
Counter max count / decrement limits and 16-channel bus transceiver verified by test_all_logic_gate_state_machines and test_repeater_schedule_and_bus_transceiver.
