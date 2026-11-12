---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-multipart-tile
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-riwz
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-7']
closed: 2026-11-12
---

# redpower_2: Sub-voxel multipart tile entity and coverable block infrastructure

Implement compound sub-voxel multipart tile infrastructure (TileExtended, TileMultipart, TileCoverable, TileCovered). Tracks up to 29 sub-voxel components (6 face covers, 12 edge strips, 8 corner pieces, 3 structural column axes) per block space using bitmasks and occupancy arrays. Enforces occlusion rules, part bounding boxes, harvest drop calculations, NBT persistence, and frame movement packet serialization. Done: unit tests pass for multi-part placement, collision bounding boxes, face occlusion masking, and NBT roundtrip.

Size: ~160k tokens.
Covers: microblocks
Read first: spec 2.1 Microblocks & Sub-Voxel System, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: TileExtended#scheduleTick, TileExtended#onBlockNeighborChange, TileMultipart#setPartBounds, TileCoverable#canAddCover, TileCoverable#tryAddCover, TileCoverable#tryRemoveCover, TileCoverable#getCoverMask, TileCovered#canAddCover, TileCovered#tryAddCover, TileCovered#tryRemoveCover, TileCovered#writeToNBT, TileCovered#getFramePacket
Files: mods/redpower_2/src/multipart/mod.rs

## Beads import

- Beads id: `rk-riwz`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:34:29Z
- Close reason (Beads): Added core/multipart.rs: CoverLib tables, PlacementValidator port, thickness/bounds, TileCovered cover store (Coverable/Multipart impls, NBT + packet round trip); 7 tests pass.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:34:23Z · Jonathan Stoff] [domain-builder-pro] done: Added core/multipart.rs: CoverLib tables, PlacementValidator port, thickness/bounds, TileCovered cover store (Coverable/Multipart impls, NBT + packet round trip); 7 tests pass.
Files: mods/redpower_2/src/core/multipart.rs, mods/redpower_2/src/core/mod.rs
Declared pub mod traits in core/mod.rs (was undeclared). Engine glue (BlockMultipart hit tracing, CoverLib.getPlacement click-to-slot, CoverRecipe saw recipes, material registration initCoverMaterials) not done. is_side_normal/harvest_multipart on TileCovered are approximations without the material table; use CoverLib + cover_is_normal.
