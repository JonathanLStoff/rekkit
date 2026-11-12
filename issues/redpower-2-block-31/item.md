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
beads_id: rk-qst.3.93
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block igniter

Differs from the original (attribute, facing, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'creates fire in front on redstone pulse and extinguishes it when unpowered' has no scenario testing it (a level-4 scenario with "covers": ['creates fire in front on redstone pulse and extinguishes it when unpowered'] in planning/mods/redpower_2.contract/igniter.json)
- face north is vanilla's minecraft:block/dispenser_front; the original is sprite 164 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/dispenser_front; the original is sprite 48 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/dispenser_front; the original is sprite 167 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/dispenser_front; the original is sprite 48 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/dispenser_front; the original is sprite 166 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/dispenser_front; the original is sprite 166 of eloraam/machine/machine1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.93`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:06Z
- Closed (Beads): 2026-11-08T14:32:02Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:29:47Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.93
