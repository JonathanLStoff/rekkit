---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-render-engine
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:asset-smith
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-c0zm
epic: redpower-2-3-2
blocked_by: ['@redpower-2-3d']
closed: 2026-11-12
---

# redpower_2: Sub-Voxel Matrix Transformation & Model Rendering Engine: RenderContext, RenderModel, RenderLib

Implement the sub-voxel geometry rendering and transformation framework. RenderContext calculates 3D bounding box corners, face rotation index mappings (texRotTable), normal vectors, and smooth ambient occlusion / lighting levels across fractional sub-voxels. RenderModel parses custom 3D mesh definitions (vertex lists, UV mappings, face groups). RenderLib provides the custom block renderer registry and procedural rendering for dynamic indicators (compass/timer pointer, levers). TextureAnimFX provides frame animation conversions. Done: unit tests pass for vertex transformation, UV mapping, texture rotation table lookups, and model scaling.

Size: ~170k tokens.
Read first: spec 2.1 Microblocks & Sub-Voxel System, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: RenderContext#setPos, RenderContext#setOrientation, RenderContext#calcLightLocal, RenderContext#renderBox, RenderModel#loadModel, RenderModel#scale, RenderLib#renderSpecialLever, RenderLib#renderPointer, RenderLib#getRenderer, TextureAnimFX#convertRGB
Files: mods/redpower_2/src/render/context.rs, mods/redpower_2/src/render/model.rs, mods/redpower_2/src/render/lib.rs

## Beads import

- Beads id: `rk-c0zm`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:57:15Z
- Close reason (Beads): Implemented RenderContext, RenderModel, RenderLib with 345 unit tests passing and check-mods verified

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T17:52:50Z · unknown] [asset-smith] partial: Implemented sub-voxel rendering engine (RenderContext, RenderModel, RenderLib, TextureAnimFX) with full unit test coverage.
