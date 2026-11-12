---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-core-math-geom
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-l66j
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: 3D Geometry & Spatial Coordinates: Vector3, Quat, WorldCoord

Port the fundamental 3D vector, quaternion rotation, and WorldCoord block coordinate primitives from RedPowerCore. WorldCoord handles 6-direction axis stepping, indirect diagonal/corner step mapping (indStep/indStepDir), Manhattan and Euclidean distance, and directional sorting comparators. Vector3 and Quat provide rigid-body transformations, pointer rotations, and sub-voxel bounding box vertices. Done: unit tests pass for coordinate stepping, distance, rotation composition, and vector math.

Size: ~110k tokens.
Read first: spec 2. Core Systems & Mechanics, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: WorldCoord#coordStep, WorldCoord#indStep, WorldCoord#getIndStepDir, WorldCoord#squareDist, Vector3#crossProduct, Vector3#dotProduct, Vector3#normalize, Quat#aroundAxis, Quat#multiply, Quat#rotate
Files: mods/redpower_2/src/core/geom.rs

## Beads import

- Beads id: `rk-l66j`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:29:37Z
- Close reason (Beads): 3D Geometry & Spatial Coordinates: Vector3, Quat, WorldCoord implemented with exact Java parity and comprehensive tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:29:36Z · Jonathan Stoff] [domain-builder] done: 3D Geometry & Spatial Coordinates: Vector3, Quat, WorldCoord implemented with exact Java parity and comprehensive tests.
Files: mods/redpower_2/src/core/spatial.rs, mods/redpower_2/src/core/foundation.rs
Vector3 arithmetic, cross product, normalization, distance, rotations, Quat and WorldCoord direction stepping, sorting comparator, and ind_step all verified and passing 11 unit tests.
