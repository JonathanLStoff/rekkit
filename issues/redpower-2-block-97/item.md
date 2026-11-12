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
beads_id: rk-qst.3.27
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block or_gate

Differs from the original (attribute, facing, shape, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'outputs high when any active input is high' has no scenario testing it (a level-4 scenario with "covers": ['outputs high when any active input is high'] in planning/mods/redpower_2.contract/or_gate.json)
- face up is vanilla's minecraft:block/repeater; the original is sprite 128 of eloraam/logic/logic2.png
- face down is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face north is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face east is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face south is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face west is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- the original's shape is [0, 0, 0, 16, 2, 16]; the model is a full cube
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.27`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:56Z
- Closed (Beads): 2026-11-08T14:34:44Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:24:29Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.27
