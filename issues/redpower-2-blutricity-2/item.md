---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-solar-thermopile
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-30hm
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine', '@redpower-2-solar-4', '@redpower-2-thermopile', '@redpower-2-sortron']
closed: 2026-11-12
---

# redpower_2: Blutricity: Solar Panel daylight generation and Thermopile thermal gradient generator

Implement Solar Panel (1/4-block flat panel, generates 2.0A daytime current when seeing sky and V <= 100V) and Thermopile (scans 6 adjacent faces every 20 ticks, generating 0.005A * min(Thot, Tcold) current from hot/cold block differences such as lava vs water/ice). Done: contract-check clean for solar_panel and thermopile; Level 1, 3, 4, 5 scenarios passing.

Size: ~120k tokens.
Covers: solar_panel, thermopile
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines, 6.1 Blocks (121 entries)
Java: TileSolarPanel#g, TileSolarPanel#setPartBounds, TileThermopile#updateTemps, TileThermopile#g
Files: mods/redpower_2/src/blutricity/solar.rs, mods/redpower_2/src/blutricity/thermopile.rs, planning/mods/redpower_2.contract/solar_panel.json, planning/mods/redpower_2.contract/thermopile.json

## Beads import

- Beads id: `rk-30hm`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:43:02Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T02:42:51Z · unknown] [domain-builder] complete: Implemented Solar Panel daylight generation (2.0A, V <= 100V) and Thermopile thermal gradient generator (0.005A * min(Thot, Tcold), 20-tick scan, thermal hazards) with Blutricity conductor dynamics and chunk reload persistence.
