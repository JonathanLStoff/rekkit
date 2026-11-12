---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-cpu-gui
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-54uc
epic: redpower-2-3-2
blocked_by: ['@redpower-2-6502']
closed: 2026-11-12
---

# redpower_2: CPU debugger console: Container and GUI

Implement ContainerCPU and GuiCPU for the CPU peripheral. Provides hardware debugger controls: display accumulator (A), index registers (X, Y), stack pointer (SP), program counter (PC), and status flags (NV-BDIZC). Handles run/stop toggle, single-step execution, and reset buttons via packet events (Packet212GuiEvent). Done: ContainerCPU and GuiCPU sync register states and handle step/run/reset events cleanly.

Size: ~70k tokens.
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: com.eloraam.redpower.control.ContainerCPU#b, com.eloraam.redpower.control.ContainerCPU#handleGuiEvent, com.eloraam.redpower.control.GuiCPU#sendEvent, com.eloraam.redpower.control.GuiCPU#sendSimple, com.eloraam.redpower.control.GuiCPU#a
Files: mods/redpower_2/src/control/cpu_gui.rs

## Beads import

- Beads id: `rk-54uc`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:23:05Z
- Close reason (Beads): ContainerCPU and GuiCPU implementation verified with tests passing, check-mods and contract-check passing

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T05:56:23Z · Jonathan Stoff] [content-author] partial: Implemented ContainerCPU and GuiCPU for 6502 CPU with hardware debugger register controls and packet event sync
Files: mods/redpower_2/src/control/mod.rs, mods/redpower_2/src/control/cpu_gui.rs, mods/redpower_2/src/lib.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/tests/mod_tests.rs
ContainerCPU and GuiCPU are fully implemented with DIP switches for Disk ID, Console ID, Bus ID, run/halt status LED, hardware debugger registers (A, X, Y, SP, PC, status flags NV-BDIZC), single-step execution, run/stop toggle, and cold-boot reset buttons. Packet212GuiEvent handling and Horizon native hooks (native_screen, native_screen_click, native_menu_button) are implemented and all unit tests in cpu_gui pass. Reached context token limit while investigating a concurrent test failure in io_expander.

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T17:23:03Z · unknown] [content-author] done: ContainerCPU and GuiCPU implementation verified with tests passing, check-mods and contract-check passing
