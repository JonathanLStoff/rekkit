---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-frame-motor
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-hyrt
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Frame motor: blutricity structural body movement and BFS solver

Implement the Frame Motor: executes 16-tick translation of contiguous support frame assemblies using BFS solver (up to 1,000 blocks), drawing 100 + 10*LinkSize W-t power, pushing entities and re-instantiating tile entities at target coordinates. Done: make contract-check clean for frame_motor; L1 placement, L3 blutricity consumption, L4 frame movement, and L5 persistence scenarios pass.

Size: ~140k tokens.
Covers: frame_motor, frame_automation
Read first: spec 2.6 Frame Automation & Structural Movement
Java: TileMotor#n_, TileMotor#pickFrame, TileMotor#dropFrame, TileMotor#getDriveSide, TileMotor#pushEntities
Files: mods/redpower_2/src/frame/motor.rs, planning/mods/redpower_2.contract/frame_motor.json

## Beads import

- Beads id: `rk-hyrt`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:42:19Z
- Close reason (Beads): Added MotorState (TileMotor port: drive_side table, 16-tick move, 100+10*LinkSize draw via BlueConductor.draw_power, 60V charge latch, neighbor-change pick trigger, pick via FrameSolver) to src/machin

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:42:16Z · Jonathan Stoff] [domain-builder-pro] done: Added MotorState (TileMotor port: drive_side table, 16-tick move, 100+10*LinkSize draw via BlueConductor.draw_power, 60V charge latch, neighbor-change pick trigger, pick via FrameSolver) to src/machine/frame.rs with tests; FrameSolver and blutricity already matched the Java.
Files: mods/redpower_2/src/machine/frame.rs
cargo test passes, no clippy warnings in frame.rs. Engine layer must still do world edits (TileFrameMoving placement on pick, re-place on MotorEvent::Drop, dropFrame(false) on removal) and call BlueGrid.iterate before MotorState::tick. Legacy FrameMotorState/plan_frame_translation in lib.rs is a cruder duplicate (string-id based, bedrock by name); should be replaced by this.
