---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-microblock-system
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-wkvv
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-3']
closed: 2026-11-12
---

# redpower_2: Microblock multi-part system: slicing hierarchy, occupancy, and materials

Implement microblock sub-voxel architecture (1/8th to 7/8th thickness hierarchy: cover, panel, triple cover, slab, cover slab, triple panel, anticover; face plates, hollow plates, edge strips, corners, and columns). Implement 27-subvoxel occupancy bitmasks, cutting recipes via handsaws, cover material registry, and multipart tile container. Done: make contract-check clean for microblock; scenario testing cutting slab into panels and covers and occlusion masking passes.

Size: ~150k tokens.
Covers: microblock, microblocks
Read first: spec Section 2.1 Microblocks & Sub-Voxel System, Section 6.1 Blocks (121 entries)
Java: BlockMicro#getCollisionBoundingBoxFromPool, ItemMicro#onItemUse, BlockCoverable#getCoverSide, BlockMultipart#getParts, BlockMultiblock#onBlockPlaced, CoverLib#addMaterial, CoverLib#getHardness, CoverRecipe#getSawRecipe, MultiLib#splitPart
Files: mods/redpower_2/src/core/microblock.rs

## Beads import

- Beads id: `rk-wkvv`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:31:34Z
- Close reason (Beads): Microblock multi-part system: slicing hierarchy, occupancy, and materials implemented with 10 unit tests and 2 integration tests passing.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-04T03:31:32Z · Jonathan Stoff] [domain-builder-pro] done: Microblock multi-part system: slicing hierarchy, occupancy, and materials implemented with 10 unit tests and 2 integration tests passing.
Files: mods/redpower_2/src/core/microblock.rs, mods/redpower_2/src/core/multipart.rs
Slicing hierarchy, recombine, post/pillar conversion, hollow passthrough, hardness gate, and sub-voxel occupancy all verified.
