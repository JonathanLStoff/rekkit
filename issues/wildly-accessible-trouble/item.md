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
beads_id: rk-qst.3.23
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block sequencer

Differs from the original (attribute, facing, level, shape, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'configurable rotation interval via GUI' has no scenario testing it (a level-3 scenario with "covers": ['configurable rotation interval via GUI'] in planning/mods/redpower_2.contract/sequencer.json)
- level 4 (interaction with other blocks): attribute 'sequentially pulses 4 outputs in continuous rotation' has no scenario testing it (a level-4 scenario with "covers": ['sequentially pulses 4 outputs in continuous rotation'] in planning/mods/redpower_2.contract/sequencer.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps sequence state and interval when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps sequence state and interval when its chunk reloads'] in planning/mods/redpower_2.contract/sequencer.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face up is vanilla's minecraft:block/repeater; the original is sprite 3 of eloraam/logic/logic1.png
- face down is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face north is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face east is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face south is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- face west is vanilla's minecraft:block/repeater; the original is sprite 0 of eloraam/logic/logic1.png
- the original's shape is [0, 0, 0, 16, 2, 16]; the model is a full cube
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.23`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:52Z
- Closed (Beads): 2026-11-08T14:34:57Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:24:10Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.23
