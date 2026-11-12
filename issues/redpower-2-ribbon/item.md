---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-ribbon-cable
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-p0ug
epic: redpower-2-3-2
blocked_by: ['@redpower-2-ribbon-2', '@redpower-2-6502']
closed: 2026-11-12
---

# redpower_2: Ribbon Cable: microblock bus routing and placement

Implement Ribbon Cable microblock (damage 3072 on BlockMicro). Propagates the 8-bit Redbus computer peripheral network between CPUs, disk drives, displays, and IO expanders across multi-block distances on block surfaces and around corners. Write contract file for ribbon_cable. Done: make contract-check clean for ribbon_cable; level 1 placement on surfaces, level 4 bus connectivity across blocks.

Size: ~60k tokens.
Covers: ribbon_cable
Read first: spec 2.2 Wiring System, 2.7 Control 6502 Computing Subsystem
Java: com.eloraam.redpower.control.TileRibbon#getConnectClass, com.eloraam.redpower.control.TileRibbon#onBlockNeighborChange, com.eloraam.redpower.control.MicroPlacementRibbon#getMicroName, com.eloraam.redpower.control.MicroPlacementRibbon#addCreativeItems, com.eloraam.redpower.control.RenderRibbon#renderWorldBlock
Files: mods/redpower_2/src/control/ribbon.rs

## Beads import

- Beads id: `rk-p0ug`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:58:41Z
- Close reason (Beads): Verified ribbon_cable microblock contract, content files, and bus routing tests

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T17:58:38Z · unknown] [content-author] done: Verified ribbon_cable microblock contract, content files, and bus routing tests
