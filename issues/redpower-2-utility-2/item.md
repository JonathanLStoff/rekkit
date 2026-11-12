---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-utility-woolcard-paint
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-60ue
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Utility crafting tools: Wool Card, Dry Paint Brush, Paint Brush, Empty Paint Can, and Paint Can

Implement Wool Card, Paint Cans, and Paint Brushes. Wool Card (ItemWoolCard, durability 63) crafts with any color wool block to tease it back into 4 string, losing 1 durability in the crafting grid. Empty Paint Can (crafting from 3 tinplate) combines with dye and seeds to produce 16-color Paint Cans with 15 uses. Dry Paint Brush crafts with a full paint can to yield a colored Paint Brush (15 durability). Colored Paint Brush colors pneumatic tubes, jacketed cables, and canvas bags on right-click, consuming 1 durability. Also covers tools_and_equipment feature. Done: make contract-check clean for wool_card, dry_paint_brush, paint_brush, empty_paint_can, paint_can; level 2 scenarios pass.

Size: ~120k tokens.
Covers: wool_card, dry_paint_brush, paint_brush, empty_paint_can, paint_can, tools_and_equipment
Read first: spec 4.1 Metallurgy & Crafting Tools, 4.3 Utility & Storage Items, 6. Complete Item & Block Registry (`redpower_2`)
Java: ItemWoolCard#ctor, ItemPaintBrush#itemUseShared, ItemPaintCan#onItemRightClick, RedPowerWorld#initialize
Files: mods/redpower_2/src/world/utility_tools.rs, planning/mods/redpower_2.contract/wool_card.json, planning/mods/redpower_2.contract/dry_paint_brush.json, planning/mods/redpower_2.contract/paint_brush.json, planning/mods/redpower_2.contract/empty_paint_can.json, planning/mods/redpower_2.contract/paint_can.json

## Beads import

- Beads id: `rk-60ue`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:31:04Z
- Close reason (Beads): Wrote 5 contracts and fixed stack size/durability for wool card, brushes and cans; check-mods passes.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:31:01Z · Jonathan Stoff] [content-author] done: Wrote 5 contracts and fixed stack size/durability for wool card, brushes and cans; check-mods passes.
Files: planning/mods/redpower_2.contract/wool_card.json, planning/mods/redpower_2.contract/dry_paint_brush.json, planning/mods/redpower_2.contract/paint_brush.json, planning/mods/redpower_2.contract/empty_paint_can.json, planning/mods/redpower_2.contract/paint_can.json, mods/redpower_2/content/data/redpower_2/horizon/item/wool_card.json, mods/redpower_2/content/data/redpower_2/horizon/item/paint_brush.json, mods/redpower_2/content/data/redpower_2/horizon/item/paint_can.json, mods/redpower_2/content/data/redpower_2/horizon/item/dry_paint_brush.json, mods/redpower_2/content/data/redpower_2/horizon/item/empty_paint_can.json
Models, lang, recipes and creative tab entries already existed. Set max_damage 63/15/15 and stacks 1/1/1/16/64 per contract. 5 items, 0 blocks. Item behaviours (card tease, brush paint, can coat) are native work.
