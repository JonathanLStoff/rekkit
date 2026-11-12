---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-cpu
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-rq39
epic: redpower-2-3-2
blocked_by: ['@redpower-2-6502-2']
closed: 2026-11-12
---

# redpower_2: 6502 CPU: tile entity, execution slice, registers and backplane memory bus

Implement the 6502 Central Processing Unit (BlockPeripheral meta 1). Runs a cycle-budgeted MOS 6502/65C816 emulation core executing up to 1,000 cycles per tick (burst cap 100,000 cycles) across a 64 KB memory space. Handles backplane bus communication (readBackplane/writeBackplane), CPU halt/run states, reset vector loading, NMI/IRQ signals, and register synchronization. Write contract file for cpu. Done: make contract-check clean for cpu; scenarios test level 1 placement, level 3 execution slice and register access, level 5 persistence across chunk reload.

Size: ~140k tokens.
Covers: cpu, control_computing
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: com.eloraam.redpower.control.TileCPU#g, com.eloraam.redpower.control.TileCPU#readBackplane, com.eloraam.redpower.control.TileCPU#writeBackplane, com.eloraam.redpower.control.TileCPU#readFromPacket, com.eloraam.redpower.control.TileCPU#writeToPacket, com.eloraam.redpower.control.BlockPeripheral#ctor, com.eloraam.redpower.control.RenderCPU#renderWorldBlock
Files: mods/redpower_2/src/control/cpu.rs

## Beads import

- Beads id: `rk-rq39`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:34:36Z
- Close reason (Beads): 6502 CPU: tile entity, execution slice, registers and backplane memory bus verified with unit and integration tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:34:35Z · Jonathan Stoff] [domain-builder-pro] done: 6502 CPU: tile entity, execution slice, registers and backplane memory bus verified with unit and integration tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/src/machine/redbus.rs
Cpu6502 registers, cycle accumulator (test_cpu_cycle_accumulator_and_burst_cap), and extended CPU instructions (test_extended_cpu_instructions) pass tests.
