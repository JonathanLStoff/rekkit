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
beads_id: rk-qst.3.189
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: item paint_can

Differs from the original (attribute, item, level), for content-author:
- level 2 (item use): attribute 'coats dry paint brush with 15 applications of colored paint' has no scenario testing it (a level-2 scenario with "covers": ['coats dry paint brush with 15 applications of colored paint'] in planning/mods/redpower_2.contract/paint_can.json)
- needs a level-2 scenario (item use)
- stacks to 1 in the original; max_stack_size is 16
- durability 15 in the original; minecraft:max_damage is None

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.189`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:01:42Z
- Closed (Beads): 2026-11-03T03:47:43Z
- Close reason (Beads): Matches the original: make contract-check and the last harness run find no difference
