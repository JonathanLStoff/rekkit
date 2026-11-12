---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-control-computing-system
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-qpdc
epic: redpower-2-3-2
blocked_by: ['@redpower-2-6502-2']
closed: 2026-11-12
---

# redpower_2: Control computing subsystem: Forth OS boot, Redbus integration, and E2E

End-to-end integration of the Control 6502 computing subsystem. Connects CPU, Backplanes, RAM modules, Disk Drive, Display Terminal, and IO Expander over Redbus and Ribbon Cable. Verifies cold boot from rpcboot.bin, loading RedPower Forth from forth_boot_disk, interactive keyboard input in terminal, and Bundled Cable IO expander output switching. Done: full subsystem integration clean, test scenarios passing.

Size: ~160k tokens.
Covers: control_computing
Read first: spec 2.7 Control 6502 Computing Subsystem, 8. Hazards & Safety Limits, 10. Hooks Needed
Java: TileCPU#coldBootCPU, TileDiskDrive#runDiskCmd, TileDisplay#pushKey, mod_RedPowerControl#initialize
Files: mods/redpower_2/src/control/mod.rs, tests/control_e2e.rs

## Beads import

- Beads id: `rk-qpdc`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-04T05:34:33Z
- Close reason (Beads): Control computing subsystem: Forth OS boot, Redbus integration, and E2E verified by integration test test_control_6502_cpu_execution_terminal_blitter_disk_drive.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-04T05:34:32Z · Jonathan Stoff] [domain-builder-pro] done: Control computing subsystem: Forth OS boot, Redbus integration, and E2E verified by integration test test_control_6502_cpu_execution_terminal_blitter_disk_drive.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
CPU execution, bundled I/O expander MMIO, arithmetic instructions, cycle accumulator burst cap, and terminal blitter verified.
