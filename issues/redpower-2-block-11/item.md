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
beads_id: rk-qst.3.113
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block blulectric_engine

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'converts blutricity into BuildCraft mechanical power' has no scenario testing it (a level-3 scenario with "covers": ['converts blutricity into BuildCraft mechanical power'] in planning/mods/redpower_2.contract/blulectric_engine.json)
- level 4 (interaction with other blocks): attribute 'provides kinetic power to adjacent BuildCraft pipes and machines' has no scenario testing it (a level-4 scenario with "covers": ['provides kinetic power to adjacent BuildCraft pipes and machines'] in planning/mods/redpower_2.contract/blulectric_engine.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its energy and flywheel charge when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its energy and flywheel charge when its chunk reloads'] in planning/mods/redpower_2.contract/blulectric_engine.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/copper_block; the original is sprite 0 of eloraam/compat/compat1.png
- face east is vanilla's minecraft:block/copper_block; the original is sprite 0 of eloraam/compat/compat1.png
- face south is vanilla's minecraft:block/copper_block; the original is sprite 0 of eloraam/compat/compat1.png
- face west is vanilla's minecraft:block/copper_block; the original is sprite 0 of eloraam/compat/compat1.png
- face up is vanilla's minecraft:block/copper_block; the original is sprite 0 of eloraam/compat/compat1.png
- face down is vanilla's minecraft:block/copper_block; the original is sprite 0 of eloraam/compat/compat1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.113`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:26Z
- Closed (Beads): 2026-11-03T03:47:44Z
- Close reason (Beads): Matches the original: make contract-check and the last harness run find no difference
