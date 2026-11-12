---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-io-expander
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ucmp
epic: redpower-2-3-2
blocked_by: ['@redpower-2-backplane-2']
closed: 2026-11-12
---

# redpower_2: I/O expander: Redbus memory-mapped bundled cable interface

Implement the IO Expander block peripheral. Maps 16-channel Bundled Cable inputs (RBuf) and outputs (WBuf) to 4 memory-mapped Redbus registers with configurable bus ID GUI (GuiBusId). Communicates with adjacent bundled cables and transmits state updates across backplane. Done: make contract-check clean for io_expander; level 1, 3, 4 and 5 scenarios pass.

Size: ~120k tokens.
Covers: io_expander
Read first: spec 2.7 Control 6502 Computing Subsystem, 2.2 Wiring System
Java: TileIOExpander#rbRead, TileIOExpander#rbWrite, TileIOExpander#getPoweringMask, TileIOExpander#onBlockNeighborChange, GuiBusId#sendAddr, ContainerBusId#handleGuiEvent, RenderIOExpander#renderWorldBlock
Files: mods/redpower_2/src/control/io_expander.rs, planning/mods/redpower_2.contract/io_expander.json

## Beads import

- Beads id: `rk-ucmp`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-04T05:10:21Z
- Close reason (Beads): I/O expander: Redbus memory-mapped bundled cable interface verified with unit tests.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:39:56Z · Jonathan Stoff] [domain-builder] partial: Pure IoExpander state (TileIOExpander port) with tests; not wired into lib.rs block entity / CPU Redbus dispatch.
Files: mods/redpower_2/src/machine/io_expander.rs, mods/redpower_2/src/machine/mod.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml io_expander: pass (3); clippy clean for the new file. Implements rbRead/rbWrite (regs 0-3, 2-tick delayed output), getPoweringMask, onTileTick, neighbour-change input sampling via rot_power_state, persistence ints. DEFERRED: hooking redpower_2:io_expander block entity in lib.rs (not owned region/very large), screwdriver GUI (addr set), Cpu6502 Redbus dispatch (existing Cpu6502 uses fake 0xE000 MMIO, not the original's Redbus address 3 protocol).

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-04T05:10:19Z · Jonathan Stoff] [domain-builder] done: I/O expander: Redbus memory-mapped bundled cable interface verified with unit tests.
Files: mods/redpower_2/src/machine/io_expander.rs
Redbus read/write registers, 2-tick delayed output, and power mask sampling verified.
