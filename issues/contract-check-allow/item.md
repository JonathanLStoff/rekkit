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
beads_id: rk-qst.3.255
epic: redpower-2-3-2
closed: 2026-11-12
---

# contract-check: allow atlas-UV models (blulectric_engine uses compat1.png sub-rects, not sprite #0)

face texture check reports 'not sprite 0 of compat1.png' for all faces; needs contract/checker support for sub-rect UVs or asset-smith to split compat1 into per-face textures

## Beads import

- Beads id: `rk-qst.3.255`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-03T03:36:06Z
- Closed (Beads): 2026-11-08T14:13:36Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-08T14:11:40Z · unknown] [domain-builder] complete: Completed implementation, verification, and tests for rk-qst.3.255
