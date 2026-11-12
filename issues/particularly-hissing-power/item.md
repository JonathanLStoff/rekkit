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
beads_id: rk-qst.3.24
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block state_cell

Differs from the original (attribute, facing, level, shape, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'configurable active duration via GUI' has no scenario testing it (a level-3 scenario with "covers": ['configurable active duration via GUI'] in planning/mods/redpower_2.contract/state_cell.json)
- level 4 (interaction with other blocks): attribute 'monostable pulse timer emitting signal for set duration after trigger pulse' has no scenario testing it (a level-4 scenario with "covers": ['monostable pulse timer emitting signal for set duration after trigger pulse'] in planning/mods/redpower_2.contract/state_cell.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps timer configuration and state when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps timer configuration and state when its chunk reloads'] in planning/mods/redpower_2.contract/state_cell.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face up is vanilla's minecraft:block/repeater; the original is sprite 48 of eloraam/logic/logic1.png
- face down is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face north is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face east is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face south is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face west is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- the original's shape is [0, 0, 0, 16, 2, 16]; the model is a full cube
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.24`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:53Z
- Closed (Beads): 2026-11-08T14:34:53Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:24:16Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.24
