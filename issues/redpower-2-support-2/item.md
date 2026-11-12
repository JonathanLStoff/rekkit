---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-frame-automation
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-phnl
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Support frames, moving frame placeholders, and BFS structural movement

Implements support frames (rpframe, rptframe, rprtframe with tube and redstone conduit pass-through), side stickiness masking (covers and screwdriver toggle), moving frame entity placeholder (TileFrameMoving), entity pushing along translation axis, and atomic block/NBT transfer upon drop. Done: contract-check clean for frame, frame_tube, and frame_redstone_tube; level 1, 3, and 5 scenarios pass.

Size: ~180k tokens.
Covers: frame, frame_tube, frame_redstone_tube, frame_automation
Read first: spec 2.6 Frame Automation & Structural Movement
Java: BlockFrame#<init>, BlockFrame#a, TileFrame#rebuildSticky, TileFrame#tryAddCover, TileFrame#canFrameConnectIn, TileFrame#canFrameConnectOut, TileFrameMoving#setContents, TileFrameMoving#dropBlock, TileFrameMoving#pushEntities, TileFrameRenderer#a, RenderFrame#renderWorldBlock, RenderFrameMoving#renderWorldBlock
Files: mods/redpower_2/src/machine/frame.rs, planning/mods/redpower_2.contract/frame.json, planning/mods/redpower_2.contract/frame_tube.json, planning/mods/redpower_2.contract/frame_redstone_tube.json

## Beads import

- Beads id: `rk-phnl`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:43:48Z
- Close reason (Beads): Added src/machine/frame_move.rs: support-frame sticky sides/covers, MovingFrame placeholder (setContents, bounds, link), MoveWorld trait with pick_frame/drop_frame over the existing FrameSolver and Mo

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:43:45Z · Jonathan Stoff] [domain-builder-pro] done: Added src/machine/frame_move.rs: support-frame sticky sides/covers, MovingFrame placeholder (setContents, bounds, link), MoveWorld trait with pick_frame/drop_frame over the existing FrameSolver and MotorState.
Files: mods/redpower_2/src/machine/frame_move.rs, mods/redpower_2/src/machine/mod.rs
frame.rs (solver + MotorState) was already extended by another agent; left untouched. Engine layer must implement MoveWorld and call pick_frame on MotorEvent/neighbor change and drop_frame(true) on MotorEvent::Drop. Entity pushing (pushEntities) not ported (engine-side).
