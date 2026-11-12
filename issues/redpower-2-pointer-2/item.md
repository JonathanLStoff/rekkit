---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-pointer-tiles
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-332s
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pointer', '@redpower-2-logic-3']
closed: 2026-11-12
---

# redpower_2: Pointer logic tiles (Timer, Sequencer, State Cell) and interval GUI

Implement pointer-based rotating logic tiles (TileLogicPointer): Timer (default interval 40 ticks / 2.0s, adjustable from 4 ticks up, emits 2-tick pulse on completion), Sequencer (4 outputs cycled sequentially at interval, rotating pointer, random.click audio on step), and State Cell (monostable pulse timer with configurable active duration). Implement GuiTimer for adjusting interval via GUI buttons. Done: contract-check clean for timer, sequencer, state_cell; level 3 GUI and level 4 timer pulse scenarios pass.

Size: ~140k tokens.
Covers: timer, sequencer, state_cell
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks (121 entries)
Java: TileLogicPointer#timerTick, TileLogicPointer#timerUpdate, TileLogicPointer#sequencerUpdate, TileLogicPointer#stateCellUpdate, TileLogicPointer#getPointerDirection, TileLogicPointer#SetInterval, TileLogicPointer#GetInterval, GuiTimer#a, mod_RedPowerLogic#init_guihandler
Files: mods/redpower_2/src/logic/pointer.rs, mods/redpower_2/native/mixins/redpower_2/gui_timer.rs, planning/mods/redpower_2.contract/timer.json, planning/mods/redpower_2.contract/sequencer.json, planning/mods/redpower_2.contract/state_cell.json

## Beads import

- Beads id: `rk-332s`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:06:18Z
- Close reason (Beads): Pointer logic tiles (Timer, Sequencer, State Cell) and interval GUI verified by test_all_logic_gate_state_machines.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:06:17Z · Jonathan Stoff] [domain-builder-pro] done: Pointer logic tiles (Timer, Sequencer, State Cell) and interval GUI verified by test_all_logic_gate_state_machines.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
Timer interval pulses, Sequencer 4-phase step, and State Cell mechanics verified.
