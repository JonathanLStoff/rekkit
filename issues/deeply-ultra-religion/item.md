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
beads_id: rk-qst.3.12
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block wire_blutricity_insulated

Differs from the original (attribute, level, shape, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'conducts 100V DC blutricity with insulation preventing bridging' has no scenario testing it (a level-4 scenario with "covers": ['conducts 100V DC blutricity with insulation preventing bridging'] in planning/mods/redpower_2.contract/wire_blutricity_insulated.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/blue_wool; the original is sprite 99 of eloraam/wiring/redpower1.png
- face east is vanilla's minecraft:block/blue_wool; the original is sprite 99 of eloraam/wiring/redpower1.png
- face south is vanilla's minecraft:block/blue_wool; the original is sprite 99 of eloraam/wiring/redpower1.png
- face west is vanilla's minecraft:block/blue_wool; the original is sprite 99 of eloraam/wiring/redpower1.png
- face up is vanilla's minecraft:block/blue_wool; the original is sprite 99 of eloraam/wiring/redpower1.png
- face down is vanilla's minecraft:block/blue_wool; the original is sprite 99 of eloraam/wiring/redpower1.png
- the original's shape is [0, 0, 0, 16, 4, 16]; the model is a full cube

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.12`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:42Z
- Closed (Beads): 2026-11-08T14:35:27Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:22:07Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.12
