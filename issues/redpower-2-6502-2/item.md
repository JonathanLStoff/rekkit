---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-cpu
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-pvqn
epic: redpower-2-3-2
blocked_by: ['@redpower-2-display-3', '@redpower-2-backplane-2', '@redpower-2-floppy-3', '@redpower-2-o']
closed: 2026-11-12
---

# redpower_2: 6502/65C816 CPU: core emulator, backplane bus, registers, and GUI

Implement the 6502 / 65C816 microprocessor CPU block peripheral with 8KB onboard RAM, backplane address decoding for external RAM modules and Redbus peripherals (Display, Disk Drive, IO Expander). Executes 1000 instruction cycles per tick with burst accumulator up to 100,000 cycles. Loads 256-byte bootloader (rpcboot.bin) into 0x0400 on cold boot. Debugger GUI with run, halt, reset, step controls, and register display. Done: make contract-check clean for cpu; level 1, 3, 4 and 5 scenarios pass.

Size: ~180k tokens.
Covers: cpu
Read first: spec 2.7 Control 6502 Computing Subsystem, 3. Machines & Devices
Java: TileCPU#coldBootCPU, TileCPU#warmBootCPU, TileCPU#haltCPU, TileCPU#executeInsn, TileCPU#refreshBackplane, TileCPU#readMem, TileCPU#writeMem, GuiCPU#sendEvent, ContainerCPU#handleGuiEvent, RenderCPU#renderWorldBlock
Files: mods/redpower_2/src/control/cpu.rs, planning/mods/redpower_2.contract/cpu.json

## Beads import

- Beads id: `rk-pvqn`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-04T05:11:45Z
- Close reason (Beads): 6502/65C816 CPU: core emulator, backplane bus, registers, and GUI verified by test_control_6502_cpu_execution_terminal_blitter_disk_drive.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-04T05:11:43Z · Jonathan Stoff] [domain-builder-pro] done: 6502/65C816 CPU: core emulator, backplane bus, registers, and GUI verified by test_control_6502_cpu_execution_terminal_blitter_disk_drive.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/src/machine/redbus.rs, mods/redpower_2/tests/mod_tests.rs
Instruction execution, cycle accumulator, backplane bus MMIO, and terminal blitter verified.
