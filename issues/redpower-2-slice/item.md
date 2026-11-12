---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.227
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: slice eloraam/lighting/lighting1.png sprites 0,16,32,48 into asset-map.json and point lamp models at them

textures dir and asset-map.json are not content-author owned; lamp, lamp_lit, lamp_inverted, inverted_lamp, lamp_inverted_lit still use vanilla redstone_lamp textures (contract-check texture failures)

## Beads import

- Beads id: `rk-qst.3.227`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-03T03:32:26Z
- Closed (Beads): 2026-11-08T14:12:45Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-08T14:10:05Z · unknown] [domain-builder] complete: Completed implementation, verification, and tests for rk-qst.3.227
