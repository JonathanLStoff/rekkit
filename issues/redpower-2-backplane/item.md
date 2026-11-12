---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-backplane-ram
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-613p
epic: redpower-2-3-2
blocked_by: ['@redpower-2-backplane-2', '@redpower-2-6502']
closed: 2026-11-12
---

# redpower_2: Backplane bus interconnect and 8K RAM expansion module

Implement Backplane bus block and 8K RAM module (BlockMultipart meta 0 and 1). Connects directly to the rear of the CPU and chains across adjacent backplane blocks to form a 16-bit address and 8-bit data bus. RAM modules each supply 8 KB of volatile RAM at assigned base addresses (expandable to 64 KB total). Write contract files for backplane and ram_module. Done: make contract-check clean for backplane and ram_module; level 1 placement, level 3 memory expansion and memory read/write over backplane, level 5 persistence.

Size: ~80k tokens.
Covers: backplane, ram_module
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: com.eloraam.redpower.control.TileBackplane#readBackplane, com.eloraam.redpower.control.TileBackplane#writeBackplane, com.eloraam.redpower.control.TileRAM#readBackplane, com.eloraam.redpower.control.TileRAM#writeBackplane, com.eloraam.redpower.control.ItemBackplane#itemUseShared, com.eloraam.redpower.control.RenderBackplane#renderWorldBlock
Files: mods/redpower_2/src/control/backplane.rs

## Beads import

- Beads id: `rk-613p`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:29:46Z
- Close reason (Beads): Implemented Backplane bus interconnect and 8K RAM expansion module domain logic with memory expansion, 6502 execution, and persistence tests

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T05:56:27Z · Jonathan Stoff] [domain-builder] partial: Implemented Backplane and 8K RAM module domain logic with hardware parity, 6502 memory map integration, and contracts
Files: mods/redpower_2/src/machine/backplane.rs, mods/redpower_2/src/control/mod.rs, mods/redpower_2/src/lib.rs, planning/mods/redpower_2.contract/backplane.json, planning/mods/redpower_2.contract/ram_module.json
STATUS: partial
FILES: mods/redpower_2/src/machine/backplane.rs, mods/redpower_2/src/control/mod.rs, mods/redpower_2/src/lib.rs, planning/mods/redpower_2.contract/backplane.json, planning/mods/redpower_2.contract/ram_module.json
TEST: backplane unit tests pass; crate tests blocked by concurrent agent edits in fluid/pump.rs
DEFERRED: none in backplane logic; content author needs to provide block models/textures
CORE-NEEDS: none

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T17:29:45Z · unknown] [domain-builder] done: Implemented Backplane bus interconnect and 8K RAM expansion module domain logic with memory expansion, 6502 execution, and persistence tests
