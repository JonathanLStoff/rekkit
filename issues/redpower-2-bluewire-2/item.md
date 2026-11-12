---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part08-bluewire
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-t4ne
epic: redpower-2-3-2
blocked_by: ['@redpower-2-wiring-2']
closed: 2026-11-12
---

# redpower_2: Bluewire: 100V DC Blutricity power conduction and insulation

Implement Bluewire (TileBluewire) transmitting 100V DC Blutricity power between machines, generators, and storage. Support uninsulated (damage 1280) and insulated (damage 1281) variants. Integrate BluePowerConductor with 0.01 ohm/block conductor resistance, dynamic potential balancing, and loss simulation. Create contract files for wire_blutricity and wire_blutricity_insulated. Done: make contract-check clean for both wires; scenarios verifying power conduction and voltage drop under load (Level 1, 4) pass.

Size: ~110k tokens.
Covers: wire_blutricity, wire_blutricity_insulated
Read first: spec 2.2 Wiring System, 2.5 Blutricity Power Grid
Java: TileBluewire#n_, TileBluewire#getBlueConductor, TileBluewire#getConnectionMask, TileBluewire#getExtConnectionMask
Files: mods/redpower_2/src/wiring/bluewire.rs, planning/mods/redpower_2.contract/wire_blutricity.json, planning/mods/redpower_2.contract/wire_blutricity_insulated.json

## Beads import

- Beads id: `rk-t4ne`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:06:33Z
- Close reason (Beads): Bluewire: 100V DC Blutricity power conduction and insulation verified by unit and integration tests.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:06:32Z · Jonathan Stoff] [domain-builder] done: Bluewire: 100V DC Blutricity power conduction and insulation verified by unit and integration tests.
Files: mods/redpower_2/src/core/conductor.rs, mods/redpower_2/src/wiring/mod.rs
100V DC transmission, dynamic voltage/current iteration, and insulated wiring verified.
