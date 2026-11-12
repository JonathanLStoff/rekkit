---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- item
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-wdlu
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: item handsaw

Differs from the original (attribute, item, level), for content-author:
- level 2 (item use): attribute 'cuts solid blocks into covers and panels in crafting grid losing durability' has no scenario testing it (a level-2 scenario with "covers": ['cuts solid blocks into covers and panels in crafting grid losing durability'] in planning/mods/redpower_2.contract/handsaw.json)
- needs a level-2 scenario (item use)
- durability 320 in the original; minecraft:max_damage is None

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-wdlu`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-02T15:14:04Z
- Closed (Beads): 2026-11-03T03:47:43Z
- Close reason (Beads): Matches the original: make contract-check and the last harness run find no difference
