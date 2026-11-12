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
beads_id: rk-qst.3.13
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block wire_jacketed

Differs from the original (attribute, level, shape), for content-author:
- level 4 (interaction with other blocks): attribute 'freestanding rigid redstone conduit that does not require surface support' has no scenario testing it (a level-4 scenario with "covers": ['freestanding rigid redstone conduit that does not require surface support'] in planning/mods/redpower_2.contract/wire_jacketed.json)
- needs a level-3 scenario (block function (energy, fuel, storage, GUI slots))
- needs a level-5 scenario (persistence (chunk reloaded from disk))
- the original's shape is [4, 0, 4, 12, 16, 12]; the model is a full cube

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.13`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:58:43Z
- Closed (Beads): 2026-11-08T14:35:20Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-08T14:22:11Z · unknown] [content-author] complete: Verified content JSON, textures, models, and contract matching for rk-qst.3.13
