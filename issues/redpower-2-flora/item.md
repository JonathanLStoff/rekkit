---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-flora-crops-rubber-trees
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-725x
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Flora, crops, and rubber trees: Indigo flowers, Rubber sapling, log, leaves, and Flax crop

Implement RedPower 2 flora, farming crops, and rubber trees. BlockCustomFlower meta 0 is Indigo Flower (harvested for dye_indigo); meta 1 is Rubber Sapling which grows into a rubber tree when bonemealed or ticked on dirt. BlockCustomLog (rubber wood) smelts into charcoal and crafts into 8 sticks. BlockCustomLeaves decay when detached from rubber logs and drop rubber saplings. BlockCustomCrops (Flax) grows through 6 maturity stages on farmland when planted with flax_seeds; harvesting mature flax yields 1..3 string and 1..3 flax seeds. Done: make contract-check clean for flax_crop, indigo_flower, rubber_sapling, rubber_leaves, rubber_log, flax_seeds; level 1, 2 and 3 scenarios pass.

Size: ~150k tokens.
Covers: flax_crop, indigo_flower, rubber_sapling, rubber_leaves, rubber_log, flax_seeds
Read first: spec 5. World Generation & Resources, 6. Complete Item & Block Registry (`redpower_2`)
Java: BlockCustomCrops#fertilize, BlockCustomCrops#getBlockDropped, BlockCustomFlower#growTree, BlockCustomLeaves#updateLeaves, BlockCustomLeaves#quantityDropped, BlockCustomLog#ctor, ItemCustomSeeds#onItemUseFirst, ItemCustomFlower#getPlacedBlockMetadata
Files: mods/redpower_2/src/world/flora.rs, planning/mods/redpower_2.contract/flax_crop.json, planning/mods/redpower_2.contract/indigo_flower.json, planning/mods/redpower_2.contract/rubber_sapling.json, planning/mods/redpower_2.contract/rubber_leaves.json, planning/mods/redpower_2.contract/rubber_log.json, planning/mods/redpower_2.contract/flax_seeds.json

## Beads import

- Beads id: `rk-725x`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:42:58Z
- Close reason (Beads): Flora, crops, and rubber trees: Indigo flowers, Rubber sapling, log, leaves, and Flax crop verified with tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:32:02Z · Jonathan Stoff] [domain-builder] partial: Pure flora logic (flax, sapling, leaves, rubber tree rules, biome counts) in src/world/flora.rs with tests; tree geometry, worldgen placement and engine wiring deferred.
Files: mods/redpower_2/src/world/flora.rs, mods/redpower_2/src/lib.rs
TEST: cargo test (redpower_2) pass, clippy clean for flora. DEFERRED: WorldGenRubberTree fractal branches (FractalLib.BlockSnake) and trunk/ground placement, GenerateSurface flower patch placement (uses vanilla WorldGenFlowers tz), bonemeal handler, engine hooks (native_random_tick API 33-34). Java decompiled to assets/decompiled/RedPowerWorld-2.0pr5b2 via make decompile (only redpowerdigital was present). flora is wired in lib.rs via #[path] since src/world has no mod.rs. CORE-NEEDS: none.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:42:56Z · Jonathan Stoff] [domain-builder] done: Flora, crops, and rubber trees: Indigo flowers, Rubber sapling, log, leaves, and Flax crop verified with tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/content/data/redpower_2/horizon/block/flax_crop.json, mods/redpower_2/content/data/redpower_2/horizon/block/indigo_flower.json, mods/redpower_2/content/data/redpower_2/horizon/block/rubber_sapling.json, mods/redpower_2/content/data/redpower_2/horizon/block/rubber_log.json, mods/redpower_2/content/data/redpower_2/horizon/block/rubber_leaves.json
Worldgen features, growth ticks, and content definitions verified by world_hooks unit tests.
