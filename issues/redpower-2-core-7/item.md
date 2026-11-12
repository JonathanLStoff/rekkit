---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-spatial-math
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-nq9h
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Core spatial math: 3D vectors and WorldCoord directional stepping

Implement 3D vector arithmetic (Vector3) and block coordinate spatial navigation (WorldCoord). Includes vector transformations, cross/dot products, normalization, orientation-based coordinate steps (0..5: -Y, +Y, -Z, +Z, -X, +X), indirect corner stepping (indStep), and square distance calculations. Done: unit tests pass for Vector3 operations and WorldCoord directional stepping matching original RedPowerCore behavior.

Size: ~80k tokens.
Read first: spec 2.1 Microblocks & Sub-Voxel System, 2.2 Wiring System, 2.4 Pneumatic Tubes & Item Logistics
Java: Vector3#crossProduct, Vector3#dotProduct, Vector3#normalize, WorldCoord#step, WorldCoord#indStep, WorldCoord#coordStep
Files: mods/redpower_2/src/core/spatial.rs

## Beads import

- Beads id: `rk-nq9h`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:31:18Z
- Close reason (Beads): Vector3 and WorldCoord (step, coord_step, ind_step, get_ind_step_dir, square_dist, comparators) already in mods/redpower_2/src/core/spatial.rs; tests pass, clippy clean.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:31:15Z · Jonathan Stoff] [domain-builder] done: Vector3 and WorldCoord (step, coord_step, ind_step, get_ind_step_dir, square_dist, comparators) already in mods/redpower_2/src/core/spatial.rs; tests pass, clippy clean.
Files: mods/redpower_2/src/core/spatial.rs, mods/redpower_2/src/core/mod.rs
No code changes needed. cargo test (11 spatial unit tests + tests/mod_tests.rs parity test) pass; cargo clippy clean. Could not compare against the Java: WorldCoord is not in assets/decompiled (only redpowerdigital archive decompiled; RedPowerCore missing). Checked against the ticket's direction table (0..5: -Y,+Y,-Z,+Z,-X,+X) only.
