---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:1
- tier:Hard
- wave:1
beads_id: rk-qst.1.1
epic: redpower-2-1
closed: 2026-11-12
---

# rekkit_core: add sub-voxel microblock bitmask structure

Microblock placement and compound multipart occlusion masking (6 faces, 12 edges, 8 corners) are shared across RedPower 2 and Immibis Microblocks.

## Beads import

- Beads id: `rk-qst.1.1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:55:43Z
- Closed (Beads): 2026-11-01T19:57:39Z
- Close reason (Beads): Not core: implemented in redpower_2 domain as the microblocks subsystem

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-01T19:57:38Z · Jonathan Stoff] Conductor: microblock voxel masks are needed only by redpower_2 in the queue (Immibis Microblocks is not queued), so per the core rule (2+ mods) this stays in the redpower_2 domain crate under the 'microblocks' subsystem feature (step 3).
