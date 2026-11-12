---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- block
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.7
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block microblock

Differs from the original (attribute, facing, shape, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'sub-voxel multipart container holding covers, panels, wires and tubes' has no scenario testing it (a level-4 scenario with "covers": ['sub-voxel multipart container holding covers, panels, wires and tubes'] in planning/mods/redpower_2.contract/microblock.json)
- face north is vanilla's minecraft:block/stone; the original is sprite 16 of eloraam/world/world1.png
- face east is vanilla's minecraft:block/stone; the original is sprite 16 of eloraam/world/world1.png
- face south is vanilla's minecraft:block/stone; the original is sprite 16 of eloraam/world/world1.png
- face west is vanilla's minecraft:block/stone; the original is sprite 16 of eloraam/world/world1.png
- face up is vanilla's minecraft:block/stone; the original is sprite 16 of eloraam/world/world1.png
- face down is vanilla's minecraft:block/stone; the original is sprite 16 of eloraam/world/world1.png
- the original's shape is [0, 0, 0, 16, 2, 16]; the model is a full cube
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.7`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:38Z
- Closed (Beads): 2026-11-08T14:35:36Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:27:50Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.7
