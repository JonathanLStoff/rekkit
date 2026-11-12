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
beads_id: rk-p2fq
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block monitor

Differs from the original (attribute, facing, level, texture), for content-author:
- level 3 (block function (energy, fuel, storage, GUI slots)): attribute 'displays 80x50 character terminal GUI with cursor and hardware blitter' has no scenario testing it (a level-3 scenario with "covers": ['displays 80x50 character terminal GUI with cursor and hardware blitter'] in planning/mods/redpower_2.contract/monitor.json)
- level 4 (interaction with other blocks): attribute 'connects to CPU over backplane and transmits keyboard input packets' has no scenario testing it (a level-4 scenario with "covers": ['connects to CPU over backplane and transmits keyboard input packets'] in planning/mods/redpower_2.contract/monitor.json)
- level 5 (persistence (chunk reloaded from disk)): attribute 'keeps its 4000-byte screen buffer when its chunk reloads' has no scenario testing it (a level-5 scenario with "covers": ['keeps its 4000-byte screen buffer when its chunk reloads'] in planning/mods/redpower_2.contract/monitor.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/black_concrete; the original is sprite 19 of eloraam/control/control1.png
- face up is vanilla's minecraft:block/black_concrete; the original is sprite 21 of eloraam/control/control1.png
- face down is vanilla's minecraft:block/black_concrete; the original is sprite 23 of eloraam/control/control1.png
- face east is vanilla's minecraft:block/black_concrete; the original is sprite 20 of eloraam/control/control1.png
- face south is vanilla's minecraft:block/black_concrete; the original is sprite 20 of eloraam/control/control1.png
- face west is vanilla's minecraft:block/black_concrete; the original is sprite 20 of eloraam/control/control1.png
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-p2fq`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-02T15:14:04Z
- Closed (Beads): 2026-11-08T14:20:06Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-08T14:19:25Z · unknown] [content-author] complete: Verified content definition, models, recipes, and contracts for rk-p2fq
