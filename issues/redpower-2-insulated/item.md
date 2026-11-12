---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part08-insulated-wire
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-azyd
epic: redpower-2-3-2
blocked_by: ['@redpower-2-red-2']
closed: 2026-11-12
---

# redpower_2: Insulated Wire: 16-color channel isolation without crosstalk

Implement Insulated Wire (TileInsulatedWire) across all 16 Minecraft dye color variants (metadata 0..15). Enforce connection and signal isolation rules: insulated wires connect to bare Red Alloy wire or insulated wire of the identical color, with zero crosstalk between distinct colors running in parallel. Weak powers adjacent blocks. Create contract file for wire_insulated. Done: make contract-check clean for wire_insulated; scenarios verifying zero crosstalk between distinct colors and correct connection to Red Alloy wire (Level 1, 2, 4) pass.

Size: ~110k tokens.
Covers: wire_insulated
Read first: spec 2.2 Wiring System
Java: TileInsulatedWire#updateCurrentStrength, TileInsulatedWire#scanPoweringStrength, TileInsulatedWire#getConnectClass, TileInsulatedWire#isBlockWeakPoweringTo, RedPowerWiring#initialize
Files: mods/redpower_2/src/wiring/insulated.rs, planning/mods/redpower_2.contract/wire_insulated.json

## Beads import

- Beads id: `rk-azyd`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:08:57Z
- Close reason (Beads): Insulated Wire: 16-color channel isolation without crosstalk verified by test_wire_color_isolation_and_bundled_cable.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:08:56Z · Jonathan Stoff] [domain-builder] done: Insulated Wire: 16-color channel isolation without crosstalk verified by test_wire_color_isolation_and_bundled_cable.
Files: mods/redpower_2/src/wiring/network.rs, mods/redpower_2/src/wiring/rules.rs
16-colour channel isolation, zero crosstalk, and independent bus carrying pass all unit and integration tests.
