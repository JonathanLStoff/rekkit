---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-io-expander
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-i9w8
epic: redpower-2-3-2
blocked_by: ['@redpower-2-6502', '@redpower-2-o']
closed: 2026-11-12
---

# redpower_2: IO Expander: Redbus memory-mapped I/O to bundled cable

Implement the IO Expander flat peripheral (BlockMultipart meta 0 in peripheralFlat). Connects to ribbon cable / Redbus at address 3, mapping read/write registers to 16 distinct bundled cable channels. Provides atomic 16-bit input latching and output driving for automation control from 6502 assembly. Write contract file for io_expander. Done: make contract-check clean for io_expander; level 1 placement, level 3 register read/write and bundled channel signal driving, level 4 wire interaction.

Size: ~70k tokens.
Covers: io_expander
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: com.eloraam.redpower.control.TileIOExpander#rbRead, com.eloraam.redpower.control.TileIOExpander#rbWrite, com.eloraam.redpower.control.TileIOExpander#getPoweringMask, com.eloraam.redpower.control.TileIOExpander#onTileTick, com.eloraam.redpower.control.RenderIOExpander#renderWorldBlock
Files: mods/redpower_2/src/control/io_expander.rs

## Beads import

- Beads id: `rk-i9w8`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:45:48Z
- Close reason (Beads): Implemented IO Expander: Redbus memory-mapped I/O registers 0..=3 to 16-channel bundled cable with DIP switch addressing and GUI.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T05:56:37Z · Jonathan Stoff] [domain-builder] partial: Implemented IO Expander flat peripheral with Redbus registers, 16-channel bundled cable I/O, and contract file.
Files: planning/mods/redpower_2.contract/io_expander.json, mods/redpower_2/src/control/io_expander.rs, mods/redpower_2/src/control/mod.rs, mods/redpower_2/src/machine/io_expander.rs, mods/redpower_2/src/lib.rs
STATUS: partial
FILES:
- planning/mods/redpower_2.contract/io_expander.json
- mods/redpower_2/src/control/io_expander.rs
- mods/redpower_2/src/control/mod.rs
- mods/redpower_2/src/machine/io_expander.rs
- mods/redpower_2/src/lib.rs
TEST: cargo test (319 passed, 1 failed in test_6502_assembly_automation_integration due to opcode mismatch with step_tick)
DEFERRED: none
CORE-NEEDS: none

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T17:44:02Z · unknown] [domain-builder] partial: Implemented IO Expander Redbus MMIO, bundled cable driving, DIP switch GUI & hooks; tests passing, formatting pending gate

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T17:45:47Z · unknown] [domain-builder] done: Implemented IO Expander: Redbus memory-mapped I/O registers 0..=3 to 16-channel bundled cable with DIP switch addressing and GUI.
