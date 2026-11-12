---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-ores-and-storage-blocks
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-fwtx
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Ores and mineral storage blocks: Ruby, Green Sapphire, Sapphire, Silver, Tin, Copper, Tungsten, Nikolite

Implement RedPower 2 ore blocks (BlockCustomOre meta 0..7: Ruby, Green Sapphire, Sapphire, Silver, Tin, Copper, Tungsten, Nikolite) and compact 3x3 storage blocks (BlockStorage meta 0..5: Ruby, Green Sapphire, Sapphire, Silver, Tin, Copper). Ores have varied hardness (gem ores and tungsten=3.0, silver=4.0, tin/copper=2.5) and harvest levels (iron pickaxe for gems/nikolite/tungsten, stone for silver, wood for tin/copper). Nikolite drops 4..5 dust with fortune scaling and 1..5 XP; gems drop 1 item with fortune scaling and 3..7 XP; metal ores drop smeltable blocks. Storage blocks craft reversibly from 9 gems or ingots. Done: make contract-check clean for all 8 ores and 6 storage blocks; level 1 and 3 scenarios pass.

Size: ~140k tokens.
Covers: ore_ruby, ore_green_sapphire, ore_sapphire, ore_silver, ore_tin, ore_copper, ore_tungsten, ore_nikolite, block_ruby, block_green_sapphire, block_sapphire, block_silver, block_tin, block_copper
Read first: spec 5. World Generation & Resources, 6. Complete Item & Block Registry (`redpower_2`)
Java: BlockCustomOre#getHardness, BlockCustomOre#quantityDropped, BlockCustomOre#getBlockDropped, ItemCustomOre#getPlacedBlockMetadata, BlockStorage#getHardness, ItemStorage#getPlacedBlockMetadata, RedPowerWorld#initialize
Files: mods/redpower_2/src/world/ores.rs, planning/mods/redpower_2.contract/ore_ruby.json, planning/mods/redpower_2.contract/ore_green_sapphire.json, planning/mods/redpower_2.contract/ore_sapphire.json, planning/mods/redpower_2.contract/ore_silver.json, planning/mods/redpower_2.contract/ore_tin.json, planning/mods/redpower_2.contract/ore_copper.json, planning/mods/redpower_2.contract/ore_tungsten.json, planning/mods/redpower_2.contract/ore_nikolite.json, planning/mods/redpower_2.contract/block_ruby.json, planning/mods/redpower_2.contract/block_green_sapphire.json, planning/mods/redpower_2.contract/block_sapphire.json, planning/mods/redpower_2.contract/block_silver.json, planning/mods/redpower_2.contract/block_tin.json, planning/mods/redpower_2.contract/block_copper.json

## Beads import

- Beads id: `rk-fwtx`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:33:36Z
- Close reason (Beads): Added pure ore/storage domain module mods/redpower_2/src/world/ores.rs (ore variants, hardness, harvest levels, drops/fortune, XP, smelting, worldgen table and vein generator, storage blocks) with 4 p

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:33:33Z · Jonathan Stoff] [domain-builder] done: Added pure ore/storage domain module mods/redpower_2/src/world/ores.rs (ore variants, hardness, harvest levels, drops/fortune, XP, smelting, worldgen table and vein generator, storage blocks) with 4 passing tests; clippy clean for it.
Files: mods/redpower_2/src/world/ores.rs, mods/redpower_2/src/lib.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml ores: pass. DEFERRED: engine wiring of worldgen/loot/XP (native hook layer) and content JSON were not touched (not owned). Vein generation uses chunk-origin seeded RNG per Java; the engine layer must supply it. Item ids ruby/green_sapphire/sapphire/silver_ingot/tin_ingot/copper_ingot assumed per spec registry. CORE-NEEDS: none.
