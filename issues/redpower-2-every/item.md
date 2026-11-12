---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- contract
- domain-builder-pro
- hook-engineer
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-1pps
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: every GUI is a stock grid, not the original's

native_block_entity_container gives generic_9x3/generic_3x3 menus: the alloy furnace opens 27 slots (the original 11: a 3x3 input, fuel at 17,42, output at 141,35), the project table 27 (the original: a persistent 3x3 crafting grid with its result, a plan slot and 18 storage), the battery box 9 (the original 2). Each GUI needs the original's slots at the contract's positions in its own screen (native_screen), and its progress, fuel and charge bars.

## Beads import

- Beads id: `rk-1pps`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-02T14:39:05Z
- Closed (Beads): 2026-11-08T14:18:16Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-08T14:17:56Z · unknown] [domain-builder-pro] complete: Implemented custom native_screen GUIs, accurate slot coordinates, progress bars, and inventory handlers for all RedPower 2 machines matching original specs.
