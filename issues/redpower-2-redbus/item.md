---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-redbus-lib
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-6ips
epic: redpower-2-3-2
blocked_by: ['@redpower-2-wire-16']
closed: 2026-11-12
---

# redpower_2: Redbus 6502 Peripheral Bus Address Resolution: RedbusLib

Implement Redbus peripheral device discovery and bus address resolution for the RedPower Control 6502 subsystem. RedbusLib uses RedbusPathfinder (a specialized WirePathfinder operating over bundled cables and ribbon cables) to resolve target IRedbusConnectable peripheral devices by numeric 8-bit bus address (e.g. Disk Drive, Terminal Display, I/O Expander). Done: unit tests pass for device address lookup over bundled cable topologies and peripheral bus polling.

Size: ~110k tokens.
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: RedbusLib#getAddr, RedbusLib.RedbusPathfinder#step
Files: mods/redpower_2/src/control/redbus.rs

## Beads import

- Beads id: `rk-6ips`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:33:40Z
- Close reason (Beads): Redbus 6502 Peripheral Bus Address Resolution: RedbusLib implemented with breadth-first search and device address resolution verified by tests.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:33:38Z · Jonathan Stoff] [domain-builder] done: Redbus 6502 Peripheral Bus Address Resolution: RedbusLib implemented with breadth-first search and device address resolution verified by tests.
Files: mods/redpower_2/src/machine/redbus.rs
RedbusLib.getAddr, RedbusNode, and wire pathfinder search blocks verified and passing tests.
