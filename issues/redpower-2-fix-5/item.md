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
beads_id: rk-qst.3.216
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: fix sound block references for wool and grass

sounds property in block declarations must reference valid vanilla blocks

## Beads import

- Beads id: `rk-qst.3.216`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T23:21:46Z
- Closed (Beads): 2026-11-01T23:23:50Z
- Close reason (Beads): Updated sound block references for wool and grass across RedPower 2 block definitions.

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-01T23:23:48Z · Jonathan Stoff] [content-author] done: Updated sound block references for wool and grass across RedPower 2 block definitions.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/ribbon_cable.json, mods/redpower_2/content/data/redpower_2/horizon/block/rubber_leaves.json, mods/redpower_2/content/data/redpower_2/horizon/block/rubber_sapling.json, mods/redpower_2/content/data/redpower_2/horizon/block/wire_blutricity_insulated.json, mods/redpower_2/content/data/redpower_2/horizon/block/wire_bundled.json, mods/redpower_2/content/data/redpower_2/horizon/block/wire_insulated.json
STATUS: done
COUNTS: 121 blocks, 99 items, 136 recipes, 121 loot tables, 221 lang keys
CHECK: make check-mods MOD=redpower_2 COMPLETE=1 pass
MISSING: none
