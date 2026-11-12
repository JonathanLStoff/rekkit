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
beads_id: rk-qst.3.2
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block project_table

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'provides persistent 3x3 crafting grid with plan slot and 18 storage slots' has no scenario testing it (a level-3 scenario with "covers": ['provides persistent 3x3 crafting grid with plan slot and 18 storage slots'] in planning/mods/redpower_2.contract/project_table.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its crafting matrix and storage contents when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its crafting matrix and storage contents when its chunk reloads'] in planning/mods/redpower_2.contract/project_table.json)
- level 6 (automation & sides (hoppers, tubes, sided access)): attribute 'allows sided insertion and extraction to internal storage' has no scenario testing it (a level-6 scenario with "covers": ['allows sided insertion and extraction to internal storage'] in planning/mods/redpower_2.contract/project_table.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face up is vanilla's minecraft:block/crafting_table_top; the original is sprite 32 of eloraam/base/base1.png
- face north is vanilla's minecraft:block/crafting_table_top; the original is sprite 33 of eloraam/base/base1.png
- face down is vanilla's minecraft:block/crafting_table_top; the original is sprite 35 of eloraam/base/base1.png
- face east is vanilla's minecraft:block/crafting_table_top; the original is sprite 34 of eloraam/base/base1.png
- face south is vanilla's minecraft:block/crafting_table_top; the original is sprite 34 of eloraam/base/base1.png
- face west is vanilla's minecraft:block/crafting_table_top; the original is sprite 34 of eloraam/base/base1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.2`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:34Z
- Closed (Beads): 2026-11-08T14:35:46Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:23:56Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.2
