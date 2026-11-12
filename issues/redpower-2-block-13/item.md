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
beads_id: rk-qst.3.111
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block disk_drive

Differs from the original (attribute, facing, level, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'communicates over Redbus with CPU' has no scenario testing it (a level-4 scenario with "covers": ['communicates over Redbus with CPU'] in planning/mods/redpower_2.contract/disk_drive.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its inserted disk when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its inserted disk when its chunk reloads'] in planning/mods/redpower_2.contract/disk_drive.json)
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/iron_block; the original is sprite 28 of eloraam/control/control1.png
- face up is vanilla's minecraft:block/iron_block; the original is sprite 29 of eloraam/control/control1.png
- face down is vanilla's minecraft:block/iron_block; the original is sprite 23 of eloraam/control/control1.png
- face east is vanilla's minecraft:block/iron_block; the original is sprite 22 of eloraam/control/control1.png
- face south is vanilla's minecraft:block/iron_block; the original is sprite 22 of eloraam/control/control1.png
- face west is vanilla's minecraft:block/iron_block; the original is sprite 22 of eloraam/control/control1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.111`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:24Z
- Closed (Beads): 2026-11-08T14:31:15Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:22:01Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.111
