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
beads_id: rk-qst.3.15
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block wire_jacketed_blutricity

Differs from the original (attribute, level, shape, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'freestanding rigid blutricity conduit that does not require surface support' has no scenario testing it (a level-4 scenario with "covers": ['freestanding rigid blutricity conduit that does not require surface support'] in planning/mods/redpower_2.contract/wire_jacketed_blutricity.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- face north is vanilla's minecraft:block/stone; the original is sprite 7 of eloraam/wiring/redpower1.png
- face east is vanilla's minecraft:block/stone; the original is sprite 7 of eloraam/wiring/redpower1.png
- face south is vanilla's minecraft:block/stone; the original is sprite 7 of eloraam/wiring/redpower1.png
- face west is vanilla's minecraft:block/stone; the original is sprite 7 of eloraam/wiring/redpower1.png
- face up is vanilla's minecraft:block/stone; the original is sprite 7 of eloraam/wiring/redpower1.png
- face down is vanilla's minecraft:block/stone; the original is sprite 7 of eloraam/wiring/redpower1.png
- the original's shape is [4, 0, 4, 12, 16, 12]; the model is a full cube

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.15`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:45Z
- Closed (Beads): 2026-11-08T14:35:14Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:22:32Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.15
