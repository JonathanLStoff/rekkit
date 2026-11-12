---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-fluid-pump
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-9sqa
epic: redpower-2-3-2
blocked_by: ['@redpower-2-fluid-5']
closed: 2026-11-12
---

# redpower_2: Fluid pump: blutricity suction and pressure pumping

Implement the blutricity-powered fluid pump: creates +1000 outlet pressure and -1000 inlet suction across attached pipes, drawing blutricity when operating. Done: make contract-check clean for pump; L1 placement, L3 power consumption, L4 pressure differential, and L5 chunk persistence scenarios pass.

Size: ~90k tokens.
Covers: pump
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines
Java: TilePump#n_, TilePump#pumpFluid, TilePump#getPipePressure, TilePump#getPipeBuffer, TilePumpRenderer#a
Files: mods/redpower_2/src/fluid/pump.rs, planning/mods/redpower_2.contract/pump.json

## Beads import

- Beads id: `rk-9sqa`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:56:35Z
- Close reason (Beads): Implemented blutricity fluid pump with Java parity, pressure differential (+1000/-1000), power consumption, fluid transfer, and contract scenarios.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:56:34Z · Jonathan Stoff] [domain-builder-pro] done: Implemented blutricity fluid pump with Java parity, pressure differential (+1000/-1000), power consumption, fluid transfer, and contract scenarios.
Files: mods/redpower_2/src/fluid/mod.rs, mods/redpower_2/src/fluid/pump.rs, mods/redpower_2/src/lib.rs, planning/mods/redpower_2.contract/pump.json
TilePump domain model implemented with 1000 outlet pressure and -1000 inlet suction, blutricity endpoint consumption (60V operational threshold, 200W continuous, 10kW pulse at stroke tick 8), buffer transfer, and chunk persistence. All unit tests and contract checks pass.
