---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-stone-mossifier
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-r8pc
epic: redpower-2-3-2
blocked_by: ['@redpower-2-world-2', '@redpower-2-decorative']
closed: 2026-11-12
---

# redpower_2: World Blocks: Marble, Basalt variants, hardness properties and mossifier spread

Implement Marble and Basalt stone variants (raw, brick, cobblestone, chiseled, paver) with custom hardness and blast resistance. Includes mossifier tick logic for spreading moss onto adjacent cobblestone and stone bricks near water sources. Done: contract-check clean for all 9 stone blocks and aliases; Level 1 scenarios passing.

Size: ~120k tokens.
Covers: marble, block_marble, basalt, block_basalt, marble_brick, basalt_cobblestone, basalt_brick, basalt_chiseled, basalt_paver
Read first: spec 5. World Generation & Resources, 6.1 Blocks (121 entries)
Java: BlockCustomStone#getPrototypicalHardness, BlockCustomStone#getExplosionResistance, BlockBrickMossifier#crackBrick, BlockBrickMossifier#spreadMoss, BlockCobbleMossifier#b
Files: mods/redpower_2/src/world/stone.rs, mods/redpower_2/src/world/mossifier.rs, planning/mods/redpower_2.contract/marble.json, planning/mods/redpower_2.contract/block_marble.json, planning/mods/redpower_2.contract/basalt.json, planning/mods/redpower_2.contract/block_basalt.json, planning/mods/redpower_2.contract/marble_brick.json, planning/mods/redpower_2.contract/basalt_cobblestone.json, planning/mods/redpower_2.contract/basalt_brick.json, planning/mods/redpower_2.contract/basalt_chiseled.json, planning/mods/redpower_2.contract/basalt_paver.json

## Beads import

- Beads id: `rk-r8pc`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:55:28Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-08T02:55:11Z · unknown] [content-author] complete: Implemented Marble and Basalt variants (raw, brick, cobblestone, chiseled, paver) with hardness, blast resistance, drops, and mossifier/brick cracking mechanics.
