---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-bluewire-and-jacketed
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-q506
epic: redpower-2-3-2
blocked_by: ['@redpower-2-red']
closed: 2026-11-12
---

# redpower_2: Bluewire Blutricity transmission and jacketed wire microblock columns

Implement Bluewire (uninsulated and insulated 100V DC transmission with 0.01 Ohm/block conductor resistance) and freestanding rigid Jacketed Wire and Jacketed Cable columnar conduits. Done: make contract-check clean for bluewire and jacketed conduits; level 1 and 4 connection scenarios pass.

Size: ~90k tokens.
Covers: wire_blutricity, wire_blutricity_insulated, wire_jacketed, cable_jacketed, wire_jacketed_blutricity
Read first: spec 2.2 Wiring System, 2.5 Blutricity Power Grid, 6.1 Blocks
Java: TileBluewire#scanPoweringStrength, MicroPlacementJacket#tryAddingJacket, MicroPlacementJacket#onPlaceMicro, TileWiring#setPartBounds
Files: mods/redpower_2/src/wiring/bluewire.rs, mods/redpower_2/src/wiring/jacketed.rs, planning/mods/redpower_2.contract/wire_blutricity.json, planning/mods/redpower_2.contract/wire_jacketed.json, planning/mods/redpower_2.contract/cable_jacketed.json

## Beads import

- Beads id: `rk-q506`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:45:36Z
- Close reason (Beads): Bluewire Blutricity transmission and jacketed wire microblock columns registered and verified.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:45:33Z · Jonathan Stoff] [domain-builder] done: Bluewire Blutricity transmission and jacketed wire microblock columns registered and verified.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/wire_blutricity.json, mods/redpower_2/content/data/redpower_2/horizon/block/wire_blutricity_insulated.json, mods/redpower_2/content/data/redpower_2/horizon/block/wire_jacketed.json, mods/redpower_2/content/data/redpower_2/horizon/block/cable_jacketed.json, mods/redpower_2/content/data/redpower_2/horizon/block/wire_jacketed_blutricity.json
Block definitions, connection masks, models, and blutricity transmission rules verified.
