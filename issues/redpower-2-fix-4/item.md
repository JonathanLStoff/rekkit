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
beads_id: rk-qst.3.217
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: fix missing texture references in buffer, buffer_chest and relay models

replace invalid minecraft:block/chest with minecraft:block/barrel_side and hopper_side with hopper_outside

## Beads import

- Beads id: `rk-qst.3.217`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-02T00:06:47Z
- Closed (Beads): 2026-11-02T00:07:55Z
- Close reason (Beads): Fix missing texture references in buffer, buffer_chest and relay models

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-02T00:07:54Z · Jonathan Stoff] [content-author] done: Fix missing texture references in buffer, buffer_chest and relay models
Files: mods/redpower_2/content/assets/redpower_2/models/block/buffer.json, mods/redpower_2/content/assets/redpower_2/models/block/buffer_chest.json, mods/redpower_2/content/assets/redpower_2/models/block/relay.json
STATUS: done
COUNTS: 121 blocks, 99 items, 136 recipes, 121 loot tables, 221 lang keys
CHECK: make check-mods MOD=redpower_2 COMPLETE=1 passed
MISSING: none
