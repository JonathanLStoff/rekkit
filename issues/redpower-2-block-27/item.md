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
beads_id: rk-qst.3.97
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block sortron

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'draws blutricity to operate Redbus sorting interface' has no scenario testing it (a level-3 scenario with "covers": ['draws blutricity to operate Redbus sorting interface'] in planning/mods/redpower_2.contract/sortron.json)
- level 4 (interaction with other blocks): attribute 'memory-mapped redbus sorting interface for 6502 computer' has no scenario testing it (a level-4 scenario with "covers": ['memory-mapped redbus sorting interface for 6502 computer'] in planning/mods/redpower_2.contract/sortron.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its sorting registers when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its sorting registers when its chunk reloads'] in planning/mods/redpower_2.contract/sortron.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/observer_top; the original is sprite 197 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/observer_top; the original is sprite 198 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/observer_top; the original is sprite 199 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/observer_top; the original is sprite 198 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/observer_top; the original is sprite 201 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/observer_top; the original is sprite 201 of eloraam/machine/machine1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.97`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:10Z
- Closed (Beads): 2026-11-08T14:31:49Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:30:05Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.97
