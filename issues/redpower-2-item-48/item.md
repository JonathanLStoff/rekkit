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
beads_id: rk-qst.3.166
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: item ruby_sword

Differs from the original (attribute, item, level), for content-author:
- level 2 (item use): attribute 'deals 7 melee attack damage to entities losing durability' has no scenario testing it (a level-2 scenario with "covers": ['deals 7 melee attack damage to entities losing durability'] in planning/mods/redpower_2.contract/ruby_sword.json)
- needs a level-2 scenario (item use)
- durability 500 in the original; minecraft:max_damage is None

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.166`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:01:18Z
- Closed (Beads): 2026-11-03T03:47:44Z
- Close reason (Beads): Matches the original: make contract-check and the last harness run find no difference
