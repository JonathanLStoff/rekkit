---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-thermopile
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-smut
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Thermopile: thermal gradient blutricity generator

Implement the solid-block Thermopile generating 0.005 * min(Thot, Tcold) Amps when voltage <= 100V based on temperature differences across all 6 faces, scanning environment every 20 ticks. Done: make contract-check clean for thermopile; L1 placement, L3 thermal gradient generation, L4 conduction, and L5 chunk persistence scenarios pass.

Size: ~90k tokens.
Covers: thermopile
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines
Java: TileThermopile#updateTemps, TileThermopile#n_, TileThermopile#getBlueConductor
Files: mods/redpower_2/src/blutricity/thermopile.rs, planning/mods/redpower_2.contract/thermopile.json

## Beads import

- Beads id: `rk-smut`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:33:40Z
- Close reason (Beads): Thermopile logic rewritten to match TileThermopile (per-block heat, air by dimension, rescan when ticks>20, generation 0.005*min from held temps, voltage<=100 gate, lava/melt predicates).

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:33:38Z · Jonathan Stoff] [domain-builder] done: Thermopile logic rewritten to match TileThermopile (per-block heat, air by dimension, rescan when ticks>20, generation 0.005*min from held temps, voltage<=100 gate, lava/melt predicates).
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
TEST: cargo test redpower_2 pass; clippy no thermopile warnings. tests/mod_tests.rs edited only to follow new ThermopileState::tick(scan closure) signature. DEFERRED: engine-side world effects (lava->obsidian/stone 1/300, snow/ice/water melt 1/300 when hot>=100) need the native layer to call lava_cools/melts_neighbors/will_rescan; not in domain crate.
