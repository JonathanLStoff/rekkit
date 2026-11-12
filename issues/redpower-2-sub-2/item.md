---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-multipart-tile-covered
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-k8qc
epic: redpower-2-3-2
blocked_by: ['@redpower-2-base-2']
closed: 2026-11-12
---

# redpower_2: Sub-Voxel Multi-Part Container & Microblock Tile: TileMultipart, TileCoverable, TileCovered

Implement the sub-voxel multi-part tile entity container holding up to 29 microblock sub-parts: 6 face plates, 12 edge strips, 8 vertex corners, 3 structural poles, and center wire/tube conduit. Manages placement checks (canAddCover preventing opposing thicknesses > 8/8ths), dynamic part addition/removal (tryAddCover, tryRemoveCover), individual part selection hitboxes, harvest drop calculation, hollow cover conduit pass-through vs solid cover face occlusion masking, and support frame movement serialization (getFramePacket/handleFramePacket). Writes the microblock contract file. Done: make contract-check clean for microblock; scenario tests pass verifying face plate addition, hollow conduit allowance, solid occlusion, and removal.

Size: ~170k tokens.
Covers: microblocks, microblock
Read first: spec 2.1 Microblocks & Sub-Voxel System, 11. Uses from `rekkit_core`
Java: TileMultipart#onHarvestPart, TileMultipart#onPartActivateSide, TileMultipart#setPartBounds, TileCoverable#canAddCover, TileCoverable#tryAddCover, TileCoverable#tryRemoveCover, TileCoverable#isSideSolid, TileCoverable#isSideNormal, TileCovered#replaceWithCovers, TileCovered#getFramePacket, TileCovered#handleFramePacket
Files: mods/redpower_2/src/microblock/tile_covered.rs, planning/mods/redpower_2.contract/microblock.json

## Beads import

- Beads id: `rk-k8qc`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:10:18Z
- Close reason (Beads): Sub-Voxel Multi-Part Container & Microblock Tile: TileMultipart, TileCoverable, TileCovered verified with 13 unit tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:10:16Z · Jonathan Stoff] [domain-builder-pro] done: Sub-Voxel Multi-Part Container & Microblock Tile: TileMultipart, TileCoverable, TileCovered verified with 13 unit tests.
Files: mods/redpower_2/src/core/multipart.rs, mods/redpower_2/src/core/microblock.rs
Add/remove parts, bounds calculation, colliding boxes, raytrace picking, and packet serialization all pass tests.
