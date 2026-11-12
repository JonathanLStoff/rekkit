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
beads_id: rk-qst.3.90
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block wind_turbine

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'generates blutricity from mounted turbine rotor based on unobstructed wind clearance' has no scenario testing it (a level-3 scenario with "covers": ['generates blutricity from mounted turbine rotor based on unobstructed wind clearance'] in planning/mods/redpower_2.contract/wind_turbine.json)
- level 4 (interaction with other blocks): attribute 'engages mechanical brake halting generation when powered by redstone' has no scenario testing it (a level-4 scenario with "covers": ['engages mechanical brake halting generation when powered by redstone'] in planning/mods/redpower_2.contract/wind_turbine.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its rotor item and power state when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its rotor item and power state when its chunk reloads'] in planning/mods/redpower_2.contract/wind_turbine.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/iron_block; the original is sprite 157 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/iron_block; the original is sprite 157 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/iron_block; the original is sprite 158 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/iron_block; the original is sprite 150 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/iron_block; the original is sprite 156 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/iron_block; the original is sprite 156 of eloraam/machine/machine1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.90`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:04Z
- Closed (Beads): 2026-11-08T14:32:09Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:29:32Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.90
