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
beads_id: rk-qst.3.16
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block pipe

Differs from the original (attribute, shape, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'conducts fluids under pressure between pumps and grates' has no scenario testing it (a level-4 scenario with "covers": ['conducts fluids under pressure between pumps and grates'] in planning/mods/redpower_2.contract/pipe.json)
- face north is vanilla's minecraft:block/glass; the original is sprite 23 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/glass; the original is sprite 23 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/glass; the original is sprite 23 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/glass; the original is sprite 23 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/glass; the original is sprite 23 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/glass; the original is sprite 23 of eloraam/machine/machine1.png
- the original's shape is [4, 4, 4, 12, 12, 12]; the model is a full cube

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.16`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:46Z
- Closed (Beads): 2026-11-08T14:35:12Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:22:35Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.16
