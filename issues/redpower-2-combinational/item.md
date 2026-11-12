---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-simple-gates
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ipgx
epic: redpower-2-3-2
blocked_by: ['@redpower-2-logic', '@redpower-2-logic-3']
closed: 2026-11-12
---

# redpower_2: Combinational logic gates (NOT, AND, OR, NAND, NOR, XOR, XNOR, Buffer)

Implement combinational logic gates based on TileLogicSimple: NOT Gate / Inverter (1 input, 3 outputs), AND Gate (3 inputs, 1 output), OR Gate (3 inputs, 1 output), NAND Gate, NOR Gate, XOR Gate (odd parity), XNOR Gate (even parity / equivalence), and Buffer Gate (1 tick delay, unidirectional). Support screwdriver deadmap cycling to disable unused input sides. Done: contract-check clean for all 8 gates; level 3 block logic and level 4 redstone interaction scenarios pass.

Size: ~140k tokens.
Covers: not_gate, and_gate, or_gate, nand_gate, nor_gate, xor_gate, xnor_gate, buffer_gate
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks (121 entries)
Java: TileLogicSimple#simpleWantsPower, TileLogicSimple#simpleUpdatePowerState, TileLogicSimple#simpleTick, TileLogicSimple#getPoweringMask, TileLogicSimple#onPartActivateSide, RenderLogicSimple#getTorchState
Files: mods/redpower_2/src/logic/simple_gates.rs, planning/mods/redpower_2.contract/not_gate.json, planning/mods/redpower_2.contract/and_gate.json, planning/mods/redpower_2.contract/or_gate.json, planning/mods/redpower_2.contract/nand_gate.json, planning/mods/redpower_2.contract/nor_gate.json, planning/mods/redpower_2.contract/xor_gate.json, planning/mods/redpower_2.contract/xnor_gate.json, planning/mods/redpower_2.contract/buffer_gate.json

## Beads import

- Beads id: `rk-ipgx`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:06:27Z
- Close reason (Beads): Combinational logic gates (NOT, AND, OR, NAND, NOR, XOR, XNOR, Buffer) verified by test_all_logic_gate_state_machines.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:06:26Z · Jonathan Stoff] [domain-builder-pro] done: Combinational logic gates (NOT, AND, OR, NAND, NOR, XOR, XNOR, Buffer) verified by test_all_logic_gate_state_machines.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
Truth tables and output signal strengths for all combinational gates verified.
