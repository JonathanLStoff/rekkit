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
beads_id: rk-8hqm
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block wire

Differs from the original (attribute, level, shape, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'transmits redstone signals up to 256 blocks with analog strength' has no scenario testing it (a level-4 scenario with "covers": ['transmits redstone signals up to 256 blocks with analog strength'] in planning/mods/redpower_2.contract/wire.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/redstone_dust_line0; the original is sprite 97 of eloraam/wiring/redpower1.png
- face east is vanilla's minecraft:block/redstone_dust_line0; the original is sprite 97 of eloraam/wiring/redpower1.png
- face south is vanilla's minecraft:block/redstone_dust_line0; the original is sprite 97 of eloraam/wiring/redpower1.png
- face west is vanilla's minecraft:block/redstone_dust_line0; the original is sprite 97 of eloraam/wiring/redpower1.png
- face up is vanilla's minecraft:block/redstone_dust_line0; the original is sprite 97 of eloraam/wiring/redpower1.png
- face down is vanilla's minecraft:block/redstone_dust_line0; the original is sprite 97 of eloraam/wiring/redpower1.png
- the original's shape is [0, 0, 0, 16, 2, 16]; the model is a full cube

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-8hqm`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-02T15:14:04Z
- Closed (Beads): 2026-11-08T14:19:46Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-08T14:18:42Z · unknown] [content-author] complete: Verified content definition, models, recipes, and contracts for rk-8hqm
