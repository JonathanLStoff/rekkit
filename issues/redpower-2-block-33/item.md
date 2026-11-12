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
beads_id: rk-qst.3.92
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block thermopile

Differs from the original (attribute, level, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'conducts blutricity to adjacent conductors' has no scenario testing it (a level-4 scenario with "covers": ['conducts blutricity to adjacent conductors'] in planning/mods/redpower_2.contract/thermopile.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its power state when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its power state when its chunk reloads'] in planning/mods/redpower_2.contract/thermopile.json)
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face up is vanilla's minecraft:block/furnace_side; the original is sprite 140 of eloraam/machine/machine1.png
- face down is vanilla's minecraft:block/furnace_side; the original is sprite 140 of eloraam/machine/machine1.png
- face north is vanilla's minecraft:block/furnace_side; the original is sprite 138 of eloraam/machine/machine1.png
- face south is vanilla's minecraft:block/furnace_side; the original is sprite 138 of eloraam/machine/machine1.png
- face east is vanilla's minecraft:block/furnace_side; the original is sprite 139 of eloraam/machine/machine1.png
- face west is vanilla's minecraft:block/furnace_side; the original is sprite 139 of eloraam/machine/machine1.png

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.92`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:00:05Z
- Closed (Beads): 2026-11-08T14:32:07Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:11Z · @beads-import

[2026-11-08T14:29:43Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.92
