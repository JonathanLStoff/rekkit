---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-logic-latches-and-sensors
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-wph1
epic: redpower-2-3-2
blocked_by: ['@redpower-2-logic']
closed: 2026-11-12
---

# redpower_2: Sequential logic latches, repeaters, synchronizers, randomizers, and light sensor

Implement RS Latch, Toggle Latch (T flip-flop), Transparent Latch (D-Latch), configurable delay Repeater (schedule table: 2..256 ticks), Synchronizer, Randomizer (1 of 3 outputs), and Light Sensor (skylight/blocklight sensing). Done: contract-check clean for covered tiles; level 1 and 4 latch and sensor scenarios pass.

Size: ~110k tokens.
Covers: rs_latch, toggle_latch, transparent_latch, repeater, synchronizer, randomizer, light_sensor
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks
Java: TileLogicSimple#latchTick, TileLogicSimple#toggleTick, TileLogicSimple#repTick, TileLogicSimple#syncTick, TileLogicSimple#randTick, TileLogicSimple#lightTick
Files: mods/redpower_2/src/logic/latches.rs, planning/mods/redpower_2.contract/rs_latch.json, planning/mods/redpower_2.contract/toggle_latch.json, planning/mods/redpower_2.contract/repeater.json, planning/mods/redpower_2.contract/light_sensor.json

## Beads import

- Beads id: `rk-wph1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:05:05Z
- Close reason (Beads): Sequential logic latches, repeaters, synchronizers, randomizers, and light sensor verified with tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:05:03Z · Jonathan Stoff] [domain-builder] done: Sequential logic latches, repeaters, synchronizers, randomizers, and light sensor verified with tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
Repeater 9-delay schedule, state cells, toggle latch, and timing gates verified by test_all_logic_gate_state_machines.
