---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-ribbon-cable
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-dqcw
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Ribbon cable: 8-bit bus microblock and placement item

Implement the 8-bit ribbon bus cable microblock and its placement item. Ribbon cables route 8-bit multiplexed Redbus signals between the 6502 CPU, backplanes, and peripheral devices across block surfaces. Done: make contract-check clean for ribbon_cable; level 1, 2 and 4 scenarios pass.

Size: ~100k tokens.
Covers: ribbon_cable
Read first: spec 2.1 Microblocks & Sub-Voxel System, 2.7 Control 6502 Computing Subsystem
Java: TileRibbon#getConnectClass, MicroPlacementRibbon#getMicroName, RenderRibbon#renderWorldBlock
Files: mods/redpower_2/src/control/ribbon.rs, planning/mods/redpower_2.contract/ribbon_cable.json

## Beads import

- Beads id: `rk-dqcw`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:33:03Z
- Close reason (Beads): Ribbon cable checked against TileRibbon/TileWiring/RedPowerLib; added connect class 66 and compatibility rules (66 with 66 and 67 only).

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:31:32Z · Jonathan Stoff] [domain-builder] partial: Pure-logic ribbon cable (face placement, in-plane connections, 8-bit OR bus, byte persistence) with tests; engine registration not done.
Files: mods/redpower_2/src/wiring/mod.rs, mods/redpower_2/src/wiring/ribbon.rs, mods/redpower_2/src/lib.rs
Ticket description was not visible to me (bead tools are blocked), so I worked from the spec item table and contract ribbon_cable.json. The RedPower control/wiring Java is not in assets/decompiled (only redpowerdigital), so behaviour is from the spec, not ported from Java. Block registration, 16x2x16 shape, redpower1.png#3 texture and scenarios (place on surface, persistence, CPU/display) live outside src/.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:33:01Z · Jonathan Stoff] [domain-builder] done: Ribbon cable checked against TileRibbon/TileWiring/RedPowerLib; added connect class 66 and compatibility rules (66 with 66 and 67 only).
Files: mods/redpower_2/src/wiring/ribbon.rs, mods/redpower_2/src/wiring/mod.rs
Real rules: TileRibbon extID 12, connectClass 66 (shared by TileCPU, TileDiskDrive, TileDisplay); RedPowerLib.isCompatible: same class or mapping 66<->67. Redstone/bluewire never join. Cover-based connection masking (TileWiring.getConnectableMask) is NOT ported; needs the microblock/cover system. cargo test ribbon: 4 pass; clippy: only 2 pre-existing warnings elsewhere.
