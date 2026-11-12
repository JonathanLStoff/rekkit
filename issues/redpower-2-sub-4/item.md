---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-subvoxel-rendering
- mod
- mod:redpower_2
- needs-split
- needs:rekkit_core
- path:mods/redpower_2
- role:hook-engineer
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-s8n3
epic: redpower-2-3-2
blocked_by: ['@redpower-2-model', '@redpower-2-sub-3']
closed: 2026-11-12
---

# redpower_2: Sub-voxel rendering context, microblock mesh generation and highlight preview

Implement sub-voxel mesh rendering context (RenderContext), microblock cover rendering (RenderCovers), raycast selection box rendering, and placement preview highlights (RenderHighlight). Computes matrix rotations, face UV alignments, smooth lighting interpolation, and pointer rendering for analog dials. Done: rendering unit tests and mesh generation tests verify exact sub-voxel vertices and UV coordinates matching original RedPower box dimensions.

Size: ~150k tokens.
Read first: spec 2.1 Microblocks & Sub-Voxel System, 10. Hooks Needed
Java: RenderContext#setupBox, RenderContext#renderBox, RenderContext#orientTextures, RenderContext#lightSmoothFace, RenderCovers#renderCovers, RenderHighlight#onBlockHighlight, RenderHighlight#drawPreview, RenderLib#renderPointer, TextureAnimFX#convertRGB
Files: mods/redpower_2/src/render/subvoxel.rs

## Beads import

- Beads id: `rk-s8n3`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-07T12:39:09Z
- Close reason (Beads): Duplicate of rk-c0zm and rk-zyt3

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-07T12:26:25Z · unknown] [hook-engineer] partial: Ticket dispatched; handed back to conductor due to session token budget reaching limit

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-07T12:29:21Z · unknown] [hook-engineer] partial: Handed off due to token budget threshold; validated test suite and prepared requirements for subvoxel rendering

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-07T12:29:23Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-07T12:39:00Z · unknown] Covered by part12 tickets: rk-c0zm (RenderContext, RenderModel, RenderLib) and rk-zyt3 (RenderHighlight, RenderCovers). Closing as duplicate.
