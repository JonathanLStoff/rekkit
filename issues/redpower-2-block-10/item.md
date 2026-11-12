---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- content-author
- contract
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-rdfy
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: block models are stand-ins for the original's

120 of 121 block models are cube_all; 92 use vanilla textures (project_table: crafting_table_top on all six faces, though project_table_top/front/side/bottom.png were sliced); gates are full cubes with the repeater texture on a repeater outline; no blockstate turns with facing. Templates were picked for their menus (dispenser, furnace, crafting_table) rather than the original's behaviour. Rebuild each model from the contract: its face sprites, its shape as elements, facing variants.

## Beads import

- Beads id: `rk-rdfy`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-02T14:39:05Z
- Closed (Beads): 2026-11-08T14:18:19Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-08T14:18:06Z · unknown] [content-author] complete: Rebuilt all 121 block models, facing states, sub-element geometries, and sliced textures according to contracts and original sprites.
