---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-advanced-simple-gates
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-958n
epic: redpower-2-3-2
blocked_by: ['@redpower-2-combinational', '@redpower-2-sequential']
closed: 2026-11-12
---

# redpower_2: Complex simple gates (Multiplexer, Repeater, Latches, Synchronizer, Randomizer, Light Sensor)

Implement advanced sequential, pulse, and sensor gates based on TileLogicSimple: Multiplexer (selects A or B based on S input), Repeater (schedule table: 2, 4, 6, 8, 16, 32, 64, 128, 256 ticks), Pulse Former (exact 1-tick pulse on rising edge), Toggle Latch (T flip-flop toggling on rising edge), RS Latch (bistable set/reset), Transparent Latch (D-latch), Synchronizer (latches two independent inputs, outputs when both arrive), Randomizer (random 1-of-3 output on clock pulse with 33.3% probability), and Light Sensor (signal on ambient light threshold). Done: contract-check clean for all 9 gates; level 3 and level 4 scenarios pass.

Size: ~150k tokens.
Covers: multiplexer, repeater, pulse_former, toggle_latch, rs_latch, transparent_latch, synchronizer, randomizer, light_sensor
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks (121 entries)
Java: TileLogicSimple#latchUpdatePowerState, TileLogicSimple#latch2UpdatePowerState, TileLogicSimple#pulseChange, TileLogicSimple#toggleUpdatePowerState, TileLogicSimple#repUpdatePowerState, TileLogicSimple#syncChange, TileLogicSimple#randUpdatePowerState, TileLogicSimple#lightTick
Files: mods/redpower_2/src/logic/special_gates.rs, planning/mods/redpower_2.contract/multiplexer.json, planning/mods/redpower_2.contract/repeater.json, planning/mods/redpower_2.contract/pulse_former.json, planning/mods/redpower_2.contract/toggle_latch.json, planning/mods/redpower_2.contract/rs_latch.json, planning/mods/redpower_2.contract/transparent_latch.json, planning/mods/redpower_2.contract/synchronizer.json, planning/mods/redpower_2.contract/randomizer.json, planning/mods/redpower_2.contract/light_sensor.json

## Beads import

- Beads id: `rk-958n`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:08:54Z
- Close reason (Beads): Complex simple gates (Multiplexer, Repeater, Latches, Synchronizer, Randomizer, Light Sensor) verified by test_all_logic_gate_state_machines.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:08:53Z · Jonathan Stoff] [domain-builder-pro] done: Complex simple gates (Multiplexer, Repeater, Latches, Synchronizer, Randomizer, Light Sensor) verified by test_all_logic_gate_state_machines.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
Multiplexer, Repeater schedule, RS latch, transparent latch, synchronizer and light sensor verified.
