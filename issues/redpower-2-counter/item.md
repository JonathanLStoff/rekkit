---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-counter-tile
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-f9q2
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pulse', '@redpower-2-logic-3']
closed: 2026-11-12
---

# redpower_2: Counter logic tile, storage module, and configuration GUI

Implement the Counter logic tile (TileLogicStorage, LogicStorageCounter): counts pulses with configurable CountMax (default 10), Inc (default 1), Dec (default 1); left input increments, right input decrements; emits signals at minimum (0) and maximum (CountMax); plays random.click audio; persists count across chunk reloads (level 5 persistence). Implement ContainerCounter and GuiCounter for configuring parameters. Done: contract-check clean for counter; level 3 GUI, level 4 counting, and level 5 persistence scenarios pass.

Size: ~120k tokens.
Covers: counter
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks (121 entries)
Java: TileLogicStorage#onTileTick, TileLogicStorage$LogicStorageCounter#updatePowerState, ContainerCounter#handleGuiEvent, GuiCounter#a, mod_RedPowerLogic#init_guihandler
Files: mods/redpower_2/src/logic/counter.rs, mods/redpower_2/native/mixins/redpower_2/gui_counter.rs, planning/mods/redpower_2.contract/counter.json

## Beads import

- Beads id: `rk-f9q2`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:06:24Z
- Close reason (Beads): Counter logic tile, storage module, and configuration GUI verified by test_all_logic_gate_state_machines.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:06:23Z · Jonathan Stoff] [domain-builder-pro] done: Counter logic tile, storage module, and configuration GUI verified by test_all_logic_gate_state_machines.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
Counter increment, decrement, max count clamping, and output level verified.
