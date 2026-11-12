---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-blutricity-core
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-6nyt
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-3']
closed: 2026-11-12
---

# redpower_2: Blutricity electrical network: conductor and endpoint domain logic

Implement Blutricity circuit physics domain model (100V DC nominal, 60V threshold, 120V safe peak, 0.01 ohm resistance, 0.07 inductive scaling dynamic transfer step). Connect with rekkit_core::energy::BlutricityNode and handle potential difference, current flow, and over-voltage hazard detection. Done: Blutricity node tests for voltage decay, charge equalization, and conductor network step resolution pass.

Size: ~125k tokens.
Covers: blutricity_grid
Read first: spec Section 2.5 Blutricity Power Grid
Java: IBluePowerConnectable#getBlueConductor, IChargeable#getCharge, BluePowerConductor#calculateCurrent, BluePowerConductor#update, BluePowerEndpoint#takePower, BluePowerLib#interact
Files: mods/redpower_2/src/core/blutricity.rs

## Beads import

- Beads id: `rk-6nyt`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:31:22Z
- Close reason (Beads): Blutricity electrical network: conductor and endpoint domain logic implemented with 14 unit tests passing.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:31:21Z · Jonathan Stoff] [domain-builder-pro] done: Blutricity electrical network: conductor and endpoint domain logic implemented with 14 unit tests passing.
Files: mods/redpower_2/src/core/conductor.rs
Exact voltage calculation, dynamic current iteration step, endpoint charge storage, and grid tick verified.
