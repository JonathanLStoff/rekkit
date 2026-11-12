---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-render-covers-highlight
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:asset-smith
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-zyt3
epic: redpower-2-3-2
blocked_by: ['@redpower-2-sub', '@redpower-2-sub-2']
closed: 2026-11-12
---

# redpower_2: Microblock Selection Box & Sub-Part Cover Renderer: RenderHighlight, RenderCovers

Implement client-side sub-voxel raytracing highlights and microblock model tessellation. RenderHighlight draws the precise selection outline when looking at individual face plates, edge strips, or corner vertices within a multipart block, as well as the breaking progress overlay and placement preview ghost. RenderCovers / CoverRenderer constructs visual geometries for 1/8th through 7/8th solid and hollow covers, edge strips, corners, and centered columns (rppole1/2/3). Done: unit tests pass verifying bounding box computation for all 29 microblock sub-parts, hollow cutouts, and highlight raytrace hit-detection.

Size: ~150k tokens.
Read first: spec 2.1 Microblocks & Sub-Voxel System
Java: RenderHighlight#onBlockHighlight, RenderHighlight#drawCornerBox, RenderHighlight#drawSideBox, RenderHighlight#drawBreaking, RenderHighlight#drawPreview, RenderCovers#render, CoverRenderer#setSize, CoverRenderer#sizeHollow, CoverRenderer#setupCorners
Files: mods/redpower_2/src/render/covers.rs, mods/redpower_2/src/render/highlight.rs

## Beads import

- Beads id: `rk-zyt3`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:29:46Z
- Close reason (Beads): Implemented sub-part bounding box calculation, hollow cutouts, raytracing hit-detection, wireframe outlines, and breaking stages in mods/redpower_2/src/render/{covers,highlight}.rs.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-08T02:29:44Z · unknown] [asset-smith] done: Implemented sub-part bounding box calculation, hollow cutouts, raytracing hit-detection, wireframe outlines, and breaking stages in mods/redpower_2/src/render/{covers,highlight}.rs.
