---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-flora-crops
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-bkse
epic: redpower-2-3-2
blocked_by: ['@redpower-2-flora', '@redpower-2-world-4']
closed: 2026-11-12
---

# redpower_2: World Flora: Flax crop growth, Indigo flowers, and Rubber tree logs/leaves

Implement Flax crops (5 growth stages, drops String and Flax Seeds on harvest, bonemeal acceleration), wild Indigo flowers (harvested for Indigo dye), and Rubber trees (rubber logs smelting into charcoal, decaying leaves dropping rubber saplings with growth into rubber trees). Done: contract-check clean for all flora blocks and flax seeds; Level 1, 2, and 3 scenarios passing.

Size: ~130k tokens.
Covers: flax_crop, flax_seeds, indigo_flower, rubber_sapling, rubber_leaves, rubber_log
Read first: spec 5. World Generation & Resources, 6.1 Blocks (121 entries), 6.2 Items (99 entries)
Java: BlockCustomCrops#fertilize, BlockCustomCrops#getBlockDropped, BlockCustomFlower#growTree, BlockCustomLeaves#b, BlockCustomLog#onBlockRemoval, ItemCustomSeeds#a
Files: mods/redpower_2/src/world/crops.rs, mods/redpower_2/src/world/trees.rs, planning/mods/redpower_2.contract/flax_crop.json, planning/mods/redpower_2.contract/flax_seeds.json, planning/mods/redpower_2.contract/indigo_flower.json, planning/mods/redpower_2.contract/rubber_sapling.json, planning/mods/redpower_2.contract/rubber_leaves.json, planning/mods/redpower_2.contract/rubber_log.json

## Beads import

- Beads id: `rk-bkse`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:58:42Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T02:58:30Z · unknown] [content-author] complete: Implemented Flax crops (growth stages, string/seed drops, bonemeal acceleration), Flax seeds planting, Rubber tree lifecycle (saplings, leaves decay, logs), and Indigo flowers.
