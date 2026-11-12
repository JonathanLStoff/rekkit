---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-wire-pathfinder-lib
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-rtra
epic: redpower-2-3-2
blocked_by: ['@redpower-2-3d']
closed: 2026-11-12
---

# redpower_2: Wire Pathfinder & Redwire Signal Propagation Engine: WirePathfinder, RedPowerLib, PowerLib

Implement the core RedPower 2 wiring logic and signal propagation algorithm. WirePathfinder traverses wire networks across block surfaces and around 90-degree corners using indirect coordinate steps (addIndBl). RedPowerLib handles 6-direction face connection masks, surface orientation mappings (mapConToLocal, mapLocalToRot), wire support checks (canSupportWire on solid/normal faces), strong and weak power queries, and lossless 0..=255 internal signal propagation up to 256 blocks, translating to vanilla redstone via strength / 17. PowerLib provides bitwise mask cutting for connection topologies. Done: unit tests pass for corner wire pathfinding, surface mapping, lossless 256-block propagation, and Red Alloy to vanilla signal scaling.

Size: ~180k tokens.
Covers: red_alloy_wiring
Read first: spec 2.2 Wiring System, 2.5 Blutricity Power Grid
Java: RedPowerLib#updateCurrent, RedPowerLib#updateBlockCurrentStrength, RedPowerLib#getMaxCurrentStrength, RedPowerLib#getTileCurrentStrength, RedPowerLib#getConnections, RedPowerLib#getExtConSides, RedPowerLib#canSupportWire, RedPowerLib#mapConToLocal, RedPowerLib#mapLocalToRot, WirePathfinder#addSearchBlocks, WirePathfinder#addIndBl, WirePathfinder#iterate, PowerLib#cutBits
Files: mods/redpower_2/src/wire/propagation.rs, mods/redpower_2/src/wire/pathfinder.rs

## Beads import

- Beads id: `rk-rtra`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:31:28Z
- Close reason (Beads): Wire Pathfinder & Redwire Signal Propagation Engine: WirePathfinder, RedPowerLib, PowerLib implemented with 26 unit tests passing.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:31:27Z · Jonathan Stoff] [domain-builder-pro] done: Wire Pathfinder & Redwire Signal Propagation Engine: WirePathfinder, RedPowerLib, PowerLib implemented with 26 unit tests passing.
Files: mods/redpower_2/src/wiring/mod.rs, mods/redpower_2/src/wiring/network.rs, mods/redpower_2/src/wiring/rules.rs, mods/redpower_2/src/wiring/ribbon.rs
Red alloy wire propagation, insulated colour isolation, bundled cable multiplexing, and connectable rules verified.
