---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-utility-items
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-lmcj
epic: redpower-2-3-2
blocked_by: ['@redpower-2-combat', '@redpower-2-utility-2']
closed: 2026-11-12
---

# redpower_2: Utility: Seed Bag auto-planting GUI, Wool Card string crafting, and Paint Can/Brush coloring

Implement Seed Bag (9-slot inventory holding up to 576 seeds, spiral 5x5 planting on farmland right-click, vacuuming picked up seeds), Wool Card (crafting remainder card turning 1 wool into 4 string), and 16-color Paint Cans & Paint Brushes for coloring tubes, cables, and canvas bags. Done: contract-check clean for all 6 utility items; Level 2 and Level 3 scenarios passing.

Size: ~130k tokens.
Covers: seed_bag, wool_card, dry_paint_brush, paint_brush, empty_paint_can, paint_can, tools_and_equipment
Read first: spec 4.1 Metallurgy & Crafting Tools, 4.3 Utility & Storage Items, 6.2 Items (99 entries)
Java: ItemSeedBag#getBagInventory, ItemSeedBag#a, ContainerSeedBag, GuiSeedBag, ItemWoolCard, ItemPaintBrush#itemUseShared, ItemPaintCan#a, WorldEvents#onPickupItem
Files: mods/redpower_2/src/items/seed_bag.rs, mods/redpower_2/src/items/wool_card.rs, mods/redpower_2/src/items/paint.rs, planning/mods/redpower_2.contract/seed_bag.json, planning/mods/redpower_2.contract/wool_card.json, planning/mods/redpower_2.contract/dry_paint_brush.json, planning/mods/redpower_2.contract/paint_brush.json, planning/mods/redpower_2.contract/empty_paint_can.json, planning/mods/redpower_2.contract/paint_can.json

## Beads import

- Beads id: `rk-lmcj`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:46Z
- Closed (Beads): 2026-11-08T03:08:12Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T03:07:54Z · unknown] [content-author] complete: Implemented Seed Bag (9 slots, 576 capacity, spiral 5x5 planting), Wool Card (crafting remainder teasing wool into 4 string), and 16-color Paint Cans & Paint Brushes.
