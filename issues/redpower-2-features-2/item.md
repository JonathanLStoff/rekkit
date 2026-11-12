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
- step:5
- tier:Hard
- wave:1
beads_id: rk-qst.5.204
epic: redpower-2-5
closed: 2026-11-12
---

# redpower_2 features.json: wrong world1.png indexes for ores, rubber_leaves, rubber_log

BlockCustomOre.a returns 32+meta, BlockCustomLeaves 48, BlockCustomLog side 50/top 51; features.json says 0-7, 52, 51. Spec-writer should correct so contract-check stops flagging correct textures.

## Beads import

- Beads id: `rk-qst.5.204`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-03T03:45:24Z
- Closed (Beads): 2026-11-08T14:51:21Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-08T14:44:28Z · unknown] [asset-smith] complete: Verified extracted and mapped textures in asset-map.json and textures directory for rk-qst.5.204
