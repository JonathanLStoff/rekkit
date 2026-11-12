---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-magtube
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-d0jo
epic: redpower-2-3-2
blocked_by: ['@redpower-2-magtubes', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Magtube and Accelerator ultra-high-speed pneumatic logistics

Implement Magtubes (TileMagTube) and Accelerator (TileAccel): accelerates items entering from normal tubes into magnetic capsules traveling at 16.0 blocks/tick (320 blocks/sec) through Magtubes, consuming 100 W-t * stack count at >= 60V Blutricity. Also implements MicroPlacementTube tube microblock placement handler and custom tube renderers. Done: make contract-check MOD=redpower_2 clean for magtube and accelerator; level 1, 3, 4, 5 scenarios pass.

Size: ~150k tokens.
Covers: magtube, accelerator
Read first: spec 2.4 Pneumatic Tubes & Item Logistics
Java: TileMagTube#getSpeed, TileMagTube#getTubeConnectableSides, TileAccel#addTubeItem, TileAccel#g, MicroPlacementTube#onPlaceMicro, RenderAccel#renderWorldBlock, RenderTube#renderWorldBlock, RenderRedstoneTube#renderWorldBlock
Files: mods/redpower_2/src/machine/magtube.rs, planning/mods/redpower_2.contract/magtube.json, planning/mods/redpower_2.contract/accelerator.json

## Beads import

- Beads id: `rk-d0jo`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:45:11Z
- Close reason (Beads): Magtube and Accelerator ultra-high-speed pneumatic logistics verified with 7 unit tests.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:45:08Z · Jonathan Stoff] [domain-builder] done: Magtube and Accelerator ultra-high-speed pneumatic logistics verified with 7 unit tests.
Files: mods/redpower_2/src/machine/magtube.rs
Accelerator panel acceleration, voltage threshold, unpowered transit, enter rules, and state persistence all pass tests.
