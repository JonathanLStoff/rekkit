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
beads_id: rk-qst.3.52
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block lumar_fixture_inverted

Differs from the original (attribute, facing, shape, texture), for content-author:
- level 4 (interaction with other blocks): attribute 'wall-mounted light fixture that illuminates by default and turns off with redstone' has no scenario testing it (a level-4 scenario with "covers": ['wall-mounted light fixture that illuminates by default and turns off with redstone'] in planning/mods/redpower_2.contract/lumar_fixture_inverted.json)
- face north is vanilla's minecraft:block/sea_lantern; the original is sprite 32 of eloraam/lighting/lighting1.png
- face east is vanilla's minecraft:block/sea_lantern; the original is sprite 32 of eloraam/lighting/lighting1.png
- face south is vanilla's minecraft:block/sea_lantern; the original is sprite 32 of eloraam/lighting/lighting1.png
- face west is vanilla's minecraft:block/sea_lantern; the original is sprite 32 of eloraam/lighting/lighting1.png
- face up is vanilla's minecraft:block/sea_lantern; the original is sprite 32 of eloraam/lighting/lighting1.png
- face down is vanilla's minecraft:block/sea_lantern; the original is sprite 32 of eloraam/lighting/lighting1.png
- the original's shape is [2, 0, 2, 14, 8, 14]; the model is a full cube
- the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.52`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:59:26Z
- Closed (Beads): 2026-11-08T14:33:41Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:12Z · @beads-import

[2026-11-08T14:26:30Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.52
