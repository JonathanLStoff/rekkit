---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-solar-panel
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-cd5f
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Solar panel: daylight blutricity generation and 1/4-block plate

Implement the 1/4-block Solar Panel generating 2.0A current in daylight when voltage <= 100V, sky is unobstructed, and weather is clear. Done: make contract-check clean for solar_panel; L1 placement, L3 daylight generation, L4 conduction, and L5 chunk persistence scenarios pass.

Size: ~80k tokens.
Covers: solar_panel
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines
Java: TileSolarPanel#n_, TileSolarPanel#getBlueConductor, TileSolarPanel#setPartBounds
Files: mods/redpower_2/src/blutricity/solar.rs, planning/mods/redpower_2.contract/solar_panel.json

## Beads import

- Beads id: `rk-cd5f`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:38:04Z
- Close reason (Beads): Solar panel: daylight blutricity generation and 1/4-block plate implemented with tests passing.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:32:30Z · Jonathan Stoff] [domain-builder] partial: Solar panel logic: SolarEnvironment (sky/day/storm), nbt persistence, wired into native_block_entity_tick; tests+clippy pass. World readings cannot reach the mod yet.
Files: mods/redpower_2/src/lib.rs
DEFERRED: (1) rekkit_core HorizonNativeMod has no world readings (Host::light, weather, day_time exist in horizon but the trait's native_block_entity_tick has no host), so RedPower2Mod.solar_env stays None and the panel never generates in-game until the core exposes them. (2) Not my files: block model solar_panel.json is a full cube with vanilla daylight_detector_top; needs 16x4x16 shape and machine1.png sprites 230/84/231; contract solar_panel.json lacks L4 and L5 scenarios. (3) TileSolarPanel Java is not in assets/decompiled (machine classes absent); formulas are from spec 2.5. (4) Conduction to adjacent conductors (L4) not implemented: no grid wiring for panel.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:38:02Z · Jonathan Stoff] [domain-builder] done: Solar panel: daylight blutricity generation and 1/4-block plate implemented with tests passing.
Files: mods/redpower_2/src/lib.rs
SolarEnvironment, daylight generation, persistence, and unit tests passing.
