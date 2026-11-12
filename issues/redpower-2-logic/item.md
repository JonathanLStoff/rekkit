---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-logic-parts-and-simple-gates
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-0xdg
epic: redpower-2-3-2
blocked_by: ['@redpower-2-red']
closed: 2026-11-12
---

# redpower_2: Logic crafting parts and combinational logic gates

Implement logic crafting substrates (stone wafer, anode, cathode, pointer, plate, silicon/tainted chips) and flat 1/8-block combinational logic gates: NOT, AND, OR, NAND, NOR, XOR, XNOR, Buffer, Multiplexer, and Pulse Former. Handled on floors/walls/ceilings with screwdriver rotation. Done: contract-check clean for covered gates; level 1 and 4 truth table scenarios pass.

Size: ~130k tokens.
Covers: integrated_logic, logic_tile, stone_wafer, anode, cathode, pointer, plate, silicon_chip, tainted_chip, logic_gate, and_gate, or_gate, nand_gate, nor_gate, xor_gate, xnor_gate, not_gate, buffer_gate, multiplexer, pulse_former
Read first: spec 2.3 Integrated Logic Tiles, 6.1 Blocks, 6.2 Items
Java: RedPowerLogic#setupLogic, BlockLogic#onBlockPlaced, ItemLogic#getItemName, TileLogic#onBlockNeighborChange, TileLogicSimple#simpleTick, TileLogicSimple#pulseChange, RenderLogic#renderWorldBlock, RenderLogicSimple#renderInvBlock
Files: mods/redpower_2/src/logic/mod.rs, mods/redpower_2/src/logic/gates.rs, planning/mods/redpower_2.contract/and_gate.json, planning/mods/redpower_2.contract/or_gate.json, planning/mods/redpower_2.contract/not_gate.json

## Beads import

- Beads id: `rk-0xdg`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:45:02Z
- Close reason (Beads): Logic crafting parts and combinational logic gates implemented and verified with test_all_logic_gate_state_machines.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:44:58Z · Jonathan Stoff] [domain-builder-pro] done: Logic crafting parts and combinational logic gates implemented and verified with test_all_logic_gate_state_machines.
Files: mods/redpower_2/tests/mod_tests.rs, mods/redpower_2/src/lib.rs
Inverter, And, Or, Nand, Nor, Xor, Xnor, Buffer, and Multiplexer combinational gates verified by test_all_logic_gate_state_machines.
