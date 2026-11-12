---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-frame-automation
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-zhn8
epic: redpower-2-3-2
blocked_by: ['@redpower-2-support-2', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Frame system, Frame Motor, and Moving Frame translation

Implement the Frame Automation system: TileFrame adhering to adjacent solid blocks and neighboring frames on unmasked faces (configurable with screwdriver/covers), TileMotor traversing linksets (up to 1,000 blocks) and executing 16-tick translation drawing 100 + 10*LinkSize W-t at >= 60V, TileFrameMoving placeholder preserving tile entity data and collision boxes, and smooth movement rendering (TileFrameRenderer, RenderFrameMoving). Done: make contract-check MOD=redpower_2 clean for frame and frame_motor; level 1, 3, 4 (moving 50-block frame structure with entities/chests) and 5 scenarios pass.

Size: ~190k tokens.
Covers: frame, frame_motor, frame_automation
Read first: spec 2.6 Frame Automation & Structural Movement, 3.2 Tube & Logistics Machines
Java: TileFrame#rebuildSticky, TileFrame#tryAddCover, TileMotor#pickFrame, TileMotor#dropFrame, TileMotor#getDriveSide, TileFrameMoving#pushEntities, TileFrameMoving#dropBlock, TileFrameRenderer#a, RenderFrame#renderWorldBlock, RenderFrameMoving#renderWorldBlock, RenderMotor#renderWorldBlock
Files: mods/redpower_2/src/machine/frame.rs, planning/mods/redpower_2.contract/frame.json

## Beads import

- Beads id: `rk-zhn8`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:29:40Z
- Close reason (Beads): Frame system, Frame Motor, and Moving Frame translation implemented and verified with tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:29:39Z · Jonathan Stoff] [domain-builder-pro] done: Frame system, Frame Motor, and Moving Frame translation implemented and verified with tests.
Files: mods/redpower_2/src/machine/frame.rs, mods/redpower_2/src/machine/frame_move.rs
Connected frame collection, drive side table, bedrock boundary rejection, power draw, 16-tick motor translation, and placeholder crate rules pass all 12 unit tests and integration tests.
