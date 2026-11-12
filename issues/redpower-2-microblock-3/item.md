---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-microblock-core
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-2o3l
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-4', '@redpower-2-crafting']
closed: 2026-11-12
---

# redpower_2: Microblock Geometry, Slicing Rules & Placement System

Implement microblock sub-voxel geometric system, slicing recipes, and placement rules: 1/8th thickness hierarchy (cover 1/8 to anticover 7/8), shapes (face plate, hollow face plate, edge strip, corner, post, pillar, column), handsaw cutting recipes (vertical cut halves thickness, horizontal cut slices strips/corners), hollow cover crafting (8 covers around empty center), slice recombination crafting, sub-voxel raytracing placement, face occlusion masking (solid covers block connections, hollow covers permit wire/tube pass-through), BlockMicro and ItemMicro. Done: make contract-check clean for microblock; its level 1, 2, and 4 scenarios pass.

Size: ~130k tokens.
Covers: microblock, microblocks
Read first: spec 2.1 Microblocks & Sub-Voxel System
Java: CoverLib#getThickness, CoverLib#damageToCoverData, CoverLib#checkPlacement, CoverLib#getPlacement, CoverRecipe#getSawRecipe, CoverRecipe#findResult, CoverRenderer#setSize, BlockMicro#<init>, ItemMicro#useCover
Files: mods/redpower_2/src/core/cover_lib.rs

## Beads import

- Beads id: `rk-2o3l`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:41:39Z
- Close reason (Beads): Added core/microblock.rs: CoverRecipe saw/column/merge/hollow recipes (find_result) and CoverLib.getPlacement resolver (get_placement); geometry/validator already in multipart.rs.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:41:34Z · Jonathan Stoff] [domain-builder-pro] done: Added core/microblock.rs: CoverRecipe saw/column/merge/hollow recipes (find_result) and CoverLib.getPlacement resolver (get_placement); geometry/validator already in multipart.rs.
Files: mods/redpower_2/src/core/microblock.rs, mods/redpower_2/src/core/mod.rs
cargo test microblock: 10 pass. Clippy clean for new file. Pre-existing failure: machine::filter::tests::template_gates_input_and_tags_color (not mine). Engine layer must wire find_result to the crafting grid (GridItem), and get_placement's can_add closure to world/TileCovered.can_add_cover (true for replaceable empty block). Saw sharpness/hardness gating uses CoverLib.hardness.
