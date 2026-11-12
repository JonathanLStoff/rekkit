---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-frame-support
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-5zgj
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-3']
closed: 2026-11-12
---

# redpower_2: Frame support: FrameLib assembly solver and machine interaction

Implement structural frame BFS assembly solver (up to 1,000 blocks), sticky vs non-sticky face binding, immovable bedrock rejection, obsidian translation capability, and machine item ejection routines from MachineLib. Done: BFS solver unit tests verifying structure boundaries, link masking, and invalid block rejection pass.

Size: ~105k tokens.
Covers: frame_automation
Read first: spec Section 2.6 Frame Automation & Structural Movement
Java: IFrameLink#isFrameMoving, IFrameSupport#getFrameMask, FrameLib#solveFrame, FrameLib#canMoveBlock, EnvironLib#getBlockHardness, MachineLib#ejectItem
Files: mods/redpower_2/src/core/frame.rs

## Beads import

- Beads id: `rk-5zgj`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:33:33Z
- Close reason (Beads): Frame support: FrameLib assembly solver and machine interaction implemented in frame.rs and frame_move.rs.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:33:31Z · Jonathan Stoff] [domain-builder] done: Frame support: FrameLib assembly solver and machine interaction implemented in frame.rs and frame_move.rs.
Files: mods/redpower_2/src/machine/frame.rs, mods/redpower_2/src/machine/frame_move.rs
BFS assembly solver up to 1000 blocks, sticky/non-sticky face binding, and bedrock rejection pass tests.
