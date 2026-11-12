---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part08-jacketed-wiring
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-9t6u
epic: redpower-2-3-2
blocked_by: ['@redpower-2-bluewire-2', '@redpower-2-bundled']
closed: 2026-11-12
---

# redpower_2: Jacketed Conduits: freestanding rigid wire, cable, and bluewire columns

Implement Jacketed Wire, Jacketed Cable, and Jacketed Bluewire (MicroPlacementJacket). Freestanding center-post columnar conduits (ConSides bit 64) that do not require attachment to a solid supporting block face. Render center post jacket textures wrapped in microblock cover materials. Create contract files for wire_jacketed, cable_jacketed, and wire_jacketed_blutricity. Done: make contract-check clean for all three jacketed conduits; scenarios testing freestanding placement in air, connection to surface wires, and jacket recipe crafting (Level 1, 2, 4) pass.

Size: ~120k tokens.
Covers: wire_jacketed, cable_jacketed, wire_jacketed_blutricity
Read first: spec 2.2 Wiring System
Java: MicroPlacementJacket#onPlaceMicro, MicroPlacementJacket#tryAddingJacket, MicroPlacementJacket#getWireMeta, RedPowerWiring#initJacketRecipes, RenderWiring#renderCenterBlock
Files: mods/redpower_2/src/wiring/jacket.rs, planning/mods/redpower_2.contract/wire_jacketed.json, planning/mods/redpower_2.contract/cable_jacketed.json, planning/mods/redpower_2.contract/wire_jacketed_blutricity.json

## Beads import

- Beads id: `rk-9t6u`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:11:38Z
- Close reason (Beads): Jacketed Conduits: freestanding rigid wire, cable, and bluewire columns verified.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:11:37Z · Jonathan Stoff] [domain-builder] done: Jacketed Conduits: freestanding rigid wire, cable, and bluewire columns verified.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/wire_jacketed.json, mods/redpower_2/content/data/redpower_2/horizon/block/cable_jacketed.json, mods/redpower_2/content/data/redpower_2/horizon/block/wire_jacketed_blutricity.json
Columnar conduits, jacket bounds, and transmission tests verified.
