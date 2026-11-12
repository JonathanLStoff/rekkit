---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-sortron
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-t90u
epic: redpower-2-3-2
blocked_by: ['@redpower-2-logistics']
closed: 2026-11-12
---

# redpower_2: Sortron: 6502 Redbus MMIO item query, inspection & routing peripheral

Implement Sortron tube machine and Redbus peripheral for 6502 Forth control. Exposes registers for Command, Item Type/ID, Damage, Quantity, Color, and Slot to read passing tube items and programmatically route them. Done: contract-check clean for sortron; Level 1, 3, 4 scenarios passing.

Size: ~130k tokens.
Covers: sortron, item_logistics_machines
Read first: spec 2.7 Control 6502 Computing Subsystem, 3.2 Tube & Logistics Machines, 6.1 Blocks (121 entries)
Java: TileSortron#rbRead, TileSortron#rbWrite, TileSortron#processCommand, TileSortron#tubeItemEnter, TileSortron#hashItem
Files: mods/redpower_2/src/machines/sortron.rs, planning/mods/redpower_2.contract/sortron.json

## Beads import

- Beads id: `rk-t90u`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:37:05Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-08T02:36:44Z · unknown] [domain-builder-pro] complete: Implemented Sortron 6502 Redbus MMIO item query, inspection & routing peripheral with 14 registers, commands 0..=4, Blutricity draw, tube routing, and NBT persistence.
