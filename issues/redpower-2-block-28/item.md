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
beads_id: rk-qst.3.96
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block relay

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'holds 9 inventory slots in GUI' has no scenario testing it (a level-3 scenario with "covers": ['holds 9 inventory slots in GUI'] in planning/mods/redpower_2.contract/relay.json)
- level 4 (interaction with other blocks): attribute 'automatically ejects items into connected tube every 10 ticks' has no scenario testing it (a level-4 scenario with "covers": ['automatically ejects items into connected tube every 10 ticks'] in planning/mods/redpower_2.contract/relay.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its inventory when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its inventory when its chunk reloads'] in planning/mods/redpower_2.contract/relay.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/hopper_outside; the original is sprite 89 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/hopper_outside; the original is sprite 58 of eloraam/machine/machine1.png
- face up is vanilla's minecraft:block/hopper_outside; the original is sprite 93 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/hopper_outside; the original is sprite 58 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/hopper_outside; the original is sprite 90 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/hopper_outside; the original is sprite 90 of eloraam/machine/machine1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.96`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:09Z
- Closed (Beads): 2026-11-08T14:31:51Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:29:57Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.96
