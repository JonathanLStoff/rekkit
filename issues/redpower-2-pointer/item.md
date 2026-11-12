---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-logic-pointer-tiles
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-cmt8
epic: redpower-2-3-2
blocked_by: ['@redpower-2-logic']
closed: 2026-11-12
---

# redpower_2: Pointer logic tiles: Timer, Sequencer, and State Cell with GUI

Implement pointer-based logic tiles: Timer (configurable interval 4..N ticks, 2-tick output pulse, GUI interval adjustment), Sequencer (rotates pointer continuously across 4 outputs, random.click audio), and State Cell (monostable pulse extender). Done: contract-check clean for pointer tiles; level 1, 3 and 4 timer/sequencer scenarios pass.

Size: ~110k tokens.
Covers: timer, sequencer, state_cell
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks
Java: TileLogicPointer#timerTick, TileLogicPointer#sequencerUpdate, TileLogicPointer#stateCellTick, IPointerTile#getPointerDirection, ContainerTimer#handleGuiEvent, GuiTimer#changeInterval, RenderLogicPointer#renderWorldBlock
Files: mods/redpower_2/src/logic/pointer.rs, planning/mods/redpower_2.contract/timer.json, planning/mods/redpower_2.contract/sequencer.json, planning/mods/redpower_2.contract/state_cell.json

## Beads import

- Beads id: `rk-cmt8`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:47:38Z
- Close reason (Beads): Pointer logic tiles: Timer, Sequencer, and State Cell with GUI implemented and tested.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:47:35Z · Jonathan Stoff] [domain-builder-pro] done: Pointer logic tiles: Timer, Sequencer, and State Cell with GUI implemented and tested.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
Timer interval pulses, Sequencer rotation and latch state, and State Cell mechanics verified by test_all_logic_gate_state_machines.
