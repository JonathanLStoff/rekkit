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
beads_id: rk-qst.3.218
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2 empty_paint_can texture is sprite 24 not 23 of items1.png

contract-check reports swapped sprite; asset-smith owns textures

## Beads import

- Beads id: `rk-qst.3.218`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-03T03:31:03Z
- Closed (Beads): 2026-11-08T14:16:14Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T14:15:58Z · unknown] [content-author] complete: Verified empty_paint_can texture sprite against Java RedPowerWorld.java:373: itemPaintCanEmpty is instantiated with icon index 23 matching contract and asset-mapping. make contract-check passes cleanly.
