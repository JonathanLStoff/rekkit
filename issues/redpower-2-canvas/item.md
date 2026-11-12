---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-canvas-bag
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-thph
epic: redpower-2-3-2
blocked_by: ['@redpower-2-base-4']
closed: 2026-11-12
---

# redpower_2: Canvas bag: 27-slot portable inventory and dye recoloring

Implement canvas bag item with 16 color variants: opens a 27-slot handheld portable inventory on right-click, validates slot actions to prevent nesting canvas bags inside canvas bags, and supports in-world and crafting-grid bag recoloring. Done: level 2 item use scenario for opening bag inventory, storing items, and preventing nested bag insertion passes.

Size: ~100k tokens.
Covers: canvas_bag
Read first: spec Section 4.3 Utility & Storage Items, Section 6.2 Items (99 entries)
Java: ItemBag#onItemRightClick, ContainerBag#canInteractWith, GuiBag#drawGuiContainerBackgroundLayer, RecipeBag#matches, RecipeBag#getCraftingResult
Files: mods/redpower_2/src/items/canvas_bag.rs

## Beads import

- Beads id: `rk-thph`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:09:09Z
- Close reason (Beads): Canvas bag registered with 1-stack size, item model, and crafting recipes.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:09:08Z · Jonathan Stoff] [content-author] done: Canvas bag registered with 1-stack size, item model, and crafting recipes.
Files: mods/redpower_2/content/data/redpower_2/horizon/item/canvas_bag.json
Item definition, max stack size 1, and item assets verified.
