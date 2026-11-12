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
beads_id: rk-qst.3.83
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block transposer

Differs from the original (attribute, facing, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'vacuums ground entities or extracts 1 stack from adjacent inventory on redstone pulse' has no scenario testing it (a level-4 scenario with "covers": ['vacuums ground entities or extracts 1 stack from adjacent inventory on redstone pulse'] in planning/mods/redpower_2.contract/transposer.json)
- face north is vanilla's minecraft:block/dropper_front_vertical; the original is sprite 50 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/dropper_front_vertical; the original is sprite 58 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/dropper_front_vertical; the original is sprite 57 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/dropper_front_vertical; the original is sprite 58 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/dropper_front_vertical; the original is sprite 59 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/dropper_front_vertical; the original is sprite 59 of eloraam/machine/machine1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.83`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:59:57Z
- Closed (Beads): 2026-11-08T14:32:24Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:29:00Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.83
