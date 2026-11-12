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
beads_id: rk-qst.3.86
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block sorting_machine

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'provides 40 filter slots across 8 color channels with mode selection GUI' has no scenario testing it (a level-3 scenario with "covers": ['provides 40 filter slots across 8 color channels with mode selection GUI'] in planning/mods/redpower_2.contract/sorting_machine.json)
- level 4 (interaction with other blocks): attribute 'routes passing tube items to color channels according to selected mode' has no scenario testing it (a level-4 scenario with "covers": ['routes passing tube items to color channels according to selected mode'] in planning/mods/redpower_2.contract/sorting_machine.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its filter slots and channel settings when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its filter slots and channel settings when its chunk reloads'] in planning/mods/redpower_2.contract/sorting_machine.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/blast_furnace_front; the original is sprite 53 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/blast_furnace_front; the original is sprite 113 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/blast_furnace_front; the original is sprite 112 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/blast_furnace_front; the original is sprite 113 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/blast_furnace_front; the original is sprite 117 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/blast_furnace_front; the original is sprite 117 of eloraam/machine/machine1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.86`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:00Z
- Closed (Beads): 2026-11-08T14:32:17Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:29:10Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.86
