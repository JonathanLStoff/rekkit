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
beads_id: rk-qst.3.91
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block regulator

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'provides 27 slots for stock template, buffer, and export filter in GUI' has no scenario testing it (a level-3 scenario with "covers": ['provides 27 slots for stock template, buffer, and export filter in GUI'] in planning/mods/redpower_2.contract/regulator.json)
- level 4 (interaction with other blocks): attribute 'maintains exact item counts in adjacent inventory using tube network' has no scenario testing it (a level-4 scenario with "covers": ['maintains exact item counts in adjacent inventory using tube network'] in planning/mods/redpower_2.contract/regulator.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its inventory slots and template settings when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its inventory slots and template settings when its chunk reloads'] in planning/mods/redpower_2.contract/regulator.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/comparator; the original is sprite 104 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/comparator; the original is sprite 102 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/comparator; the original is sprite 103 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/comparator; the original is sprite 102 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/comparator; the original is sprite 106 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/comparator; the original is sprite 106 of eloraam/machine/machine1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.91`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:05Z
- Closed (Beads): 2026-11-08T14:32:05Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:29:36Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.91
