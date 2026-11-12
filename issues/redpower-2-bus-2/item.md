---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-bus-transceiver
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-5rei
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pulse', '@redpower-2-logic-3']
closed: 2026-11-12
---

# redpower_2: Bus Transceiver: 16-channel bundled cable interfacing and latching

Implement the Bus Transceiver (TileLogicAdv, LogicAdvXcvr): connects two 16-channel Bundled Cables across opposing sides; control inputs enable bidirectional data transfer (A to B, B to A) or transparent pass-through; latches 16-channel bundled signals on control pulse; preserves internal 16-bit register across chunk reloads (level 5 persistence). Done: contract-check clean for bus_transceiver; level 4 bundled cable transceiver scenarios pass.

Size: ~120k tokens.
Covers: bus_transceiver
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks (121 entries)
Java: TileLogicAdv#getPoweringMask, TileLogicAdv$LogicAdvXcvr#updatePowerState, TileLogicAdv#onBlockNeighborChange
Files: mods/redpower_2/src/logic/transceiver.rs, planning/mods/redpower_2.contract/bus_transceiver.json

## Beads import

- Beads id: `rk-5rei`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:06:21Z
- Close reason (Beads): Bus Transceiver: 16-channel bundled cable interfacing and latching verified by test_repeater_schedule_and_bus_transceiver.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:06:20Z · Jonathan Stoff] [domain-builder-pro] done: Bus Transceiver: 16-channel bundled cable interfacing and latching verified by test_repeater_schedule_and_bus_transceiver.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
16-channel bundled cable bus pass-through and latching verified.
