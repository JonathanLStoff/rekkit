---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-ores-storage
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-bk8f
epic: redpower-2-3-2
blocked_by: ['@redpower-2-blutricity', '@redpower-2-ores']
closed: 2026-11-12
---

# redpower_2: World Blocks: Custom gem and metal ores with metal/gem storage blocks

Implement 8 custom ores (Ruby, Green Sapphire, Sapphire, Silver, Tin, Copper, Tungsten, Nikolite) with Fortune-affected gem drops, XP emission, and Silk Touch behavior, plus Tungsten Ingot and 6 metallic and gemstone 3x3 compact storage blocks. Done: contract-check clean for all 8 ores, tungsten_ingot, and 6 storage blocks; Level 1 and Level 2 scenarios passing.

Size: ~130k tokens.
Covers: ore_ruby, ore_green_sapphire, ore_sapphire, ore_silver, ore_tin, ore_copper, ore_tungsten, ore_nikolite, tungsten_ingot, block_ruby, block_green_sapphire, block_sapphire, block_silver, block_tin, block_copper
Read first: spec 5. World Generation & Resources, 6.1 Blocks (121 entries), 6.2 Items (99 entries)
Java: BlockCustomOre#quantityDropped, BlockCustomOre#a, BlockStorage#a, ItemCustomOre#b, ItemStorage#b
Files: mods/redpower_2/src/world/ores.rs, mods/redpower_2/src/world/storage.rs, planning/mods/redpower_2.contract/ore_ruby.json, planning/mods/redpower_2.contract/ore_green_sapphire.json, planning/mods/redpower_2.contract/ore_sapphire.json, planning/mods/redpower_2.contract/ore_silver.json, planning/mods/redpower_2.contract/ore_tin.json, planning/mods/redpower_2.contract/ore_copper.json, planning/mods/redpower_2.contract/ore_tungsten.json, planning/mods/redpower_2.contract/ore_nikolite.json, planning/mods/redpower_2.contract/tungsten_ingot.json, planning/mods/redpower_2.contract/block_ruby.json, planning/mods/redpower_2.contract/block_green_sapphire.json, planning/mods/redpower_2.contract/block_sapphire.json, planning/mods/redpower_2.contract/block_silver.json, planning/mods/redpower_2.contract/block_tin.json, planning/mods/redpower_2.contract/block_copper.json

## Beads import

- Beads id: `rk-bk8f`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:52:51Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T02:52:34Z · unknown] [content-author] complete: Implemented 8 custom ores (Fortune drops, XP emission, harvest levels, smelting) with Tungsten Ingot and all 6 compact storage blocks (Ruby, Green Sapphire, Sapphire, Silver, Tin, Copper).
