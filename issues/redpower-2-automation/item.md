---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-automation-helpers
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ff1l
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-5']
closed: 2026-11-12
---

# redpower_2: Automation Framework: Frame Solver, Redbus & Disk Helpers

Implement FrameLib::FrameSolver structural movement linkage graph solver (IFrameLink traversal), RedbusLib 8-bit bus addressing and ribbon pathfinding, and DiskLib serial number generation and disk directory resolution. Done: unit tests for frame structure linkage resolution, redbus address query, and disk serial generation pass.

Size: ~80k tokens.
Read first: spec 2.6 Frame Automation & Structural Movement, 2.7 Control 6502 Computing Subsystem
Java: FrameLib$FrameSolver#solve, RedbusLib#getAddr, DiskLib#generateSerialNumber, DiskLib#getSaveDir
Files: mods/redpower_2/src/core/automation.rs

## Beads import

- Beads id: `rk-ff1l`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:38:57Z
- Close reason (Beads): Added pure-logic FrameSolver (FrameLib), Redbus address search (RedbusLib/WirePathfinder) and DiskDrive register/command logic (TileDiskDrive), with unit tests.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:38:56Z · Jonathan Stoff] [domain-builder] done: Added pure-logic FrameSolver (FrameLib), Redbus address search (RedbusLib/WirePathfinder) and DiskDrive register/command logic (TileDiskDrive), with unit tests.
Files: mods/redpower_2/src/machine/frame.rs, mods/redpower_2/src/machine/redbus.rs, mods/redpower_2/src/machine/disk.rs, mods/redpower_2/src/machine/mod.rs
cargo test --lib machine:: passes (72). Engine layer must implement FrameWorld, RedbusWorld, DiskStore. FrameWorld.is_loaded maps obfuscated yc.f(); can_replace maps yc.a(..replace..). Disk image resources (redforth.img) read via DiskStore.
