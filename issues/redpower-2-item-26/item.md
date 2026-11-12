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
beads_id: rk-qst.3.188
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: item empty_paint_can

Differs from the original (attribute, item), for content-author:
- level 2 (item use): attribute 'is filled with dye to become a paint can' has no scenario testing it (a level-2 scenario with "covers": ['is filled with dye to become a paint can'] in planning/mods/redpower_2.contract/empty_paint_can.json)
- stacks to 64 in the original; max_stack_size is 16

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-qst.3.188`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T20:01:41Z
- Closed (Beads): 2026-11-03T03:47:43Z
- Close reason (Beads): Matches the original: make contract-check and the last harness run find no difference
