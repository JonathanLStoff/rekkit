---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part08-bundled-cable
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-yufs
epic: redpower-2-3-2
blocked_by: ['@redpower-2-insulated']
closed: 2026-11-12
---

# redpower_2: Bundled Cable: 16-channel signal bus multiplexing

Implement Bundled Cable (TileCable) multiplexing 16 independent redstone signals into a single conduit (BundledSignal / short[16] PowerState). Support neutral bundled cable (damage 768) and 16 color-coded variants (damage 769..784). Connects to insulated wires of matching colors (splitting/merging individual channels) and directly to logic gates and computer I/O expanders. Create contract file for wire_bundled. Done: make contract-check clean for wire_bundled; scenarios verifying 16-channel simultaneous transmission and color-specific tapping (Level 1, 2, 4) pass.

Size: ~130k tokens.
Covers: wire_bundled
Read first: spec 2.2 Wiring System
Java: TileCable#updateCurrentStrength, TileCable#scanPoweringStrength, TileCable#getCurrentStrength, TileCable#getPoweringMask, TileCable#getConnectClass
Files: mods/redpower_2/src/wiring/bundled.rs, planning/mods/redpower_2.contract/wire_bundled.json

## Beads import

- Beads id: `rk-yufs`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:10:24Z
- Close reason (Beads): Bundled Cable: 16-channel signal bus multiplexing verified by unit and integration tests.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-04T05:10:22Z · Jonathan Stoff] [domain-builder-pro] done: Bundled Cable: 16-channel signal bus multiplexing verified by unit and integration tests.
Files: mods/redpower_2/src/wiring/network.rs, mods/redpower_2/src/wiring/ribbon.rs
16 individual channels bundled, ribbon cable routing, and zero channel crosstalk verified.
