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
beads_id: rk-qst.3.10
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block wire_bundled

Differs from the original (attribute, level, shape), for content-author:
- level 4 (interaction with other blocks): attribute 'multiplexes 16 insulated redstone channels into a single cable' has no scenario testing it (a level-4 scenario with "covers": ['multiplexes 16 insulated redstone channels into a single cable'] in planning/mods/redpower_2.contract/wire_bundled.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- the original's shape is [0, 0, 0, 16, 4, 16]; the model is a full cube

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.10`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:41Z
- Closed (Beads): 2026-11-08T14:35:31Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:21:01Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.10
