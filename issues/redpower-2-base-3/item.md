---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-base-tools
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-3i6t
epic: redpower-2-3-2
blocked_by: ['@redpower-2-base-4', '@redpower-2-microblock-2']
closed: 2026-11-12
---

# redpower_2: Base tools: iron and diamond handsaws, screwdriver, diamond drawplate, and indigo dye

Implement crafting and utility tools from RedPowerBase: iron handsaw (durability 320, sharpness 0), diamond handsaw (durability 1280, sharpness 2) with crafting remainder behavior; screwdriver with right-click block/gate rotation and mode toggle; diamond drawplate (durability 255) for pulling fine copper and iron wire; indigo dye for blue dyeing and flax interaction. Done: level 2 item use scenarios for handsaw cutting, screwdriver rotation, and drawplate wire pulling pass.

Size: ~130k tokens.
Covers: iron_handsaw, handsaw, diamond_handsaw, screwdriver, diamond_drawplate, dye_indigo
Read first: spec Section 4.1 Metallurgy & Crafting Tools, Section 6.2 Items (99 entries)
Java: ItemHandsaw#getSharpness, ItemHandsaw#hasContainerItem, ItemScrewdriver#onItemUse, ItemDrawplate#hasContainerItem, ItemDyeIndigo#onItemUse, RedPowerBase#initIndigo, RedPowerBase#initBlocks
Files: mods/redpower_2/src/items/tools.rs

## Beads import

- Beads id: `rk-3i6t`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:37:54Z
- Close reason (Beads): Base tools registered: iron/diamond handsaws, screwdriver, diamond drawplate, and indigo dye with recipes.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:37:52Z · Jonathan Stoff] [content-author] done: Base tools registered: iron/diamond handsaws, screwdriver, diamond drawplate, and indigo dye with recipes.
Files: mods/redpower_2/content/data/redpower_2/horizon/item/iron_handsaw.json, mods/redpower_2/content/data/redpower_2/horizon/item/diamond_handsaw.json, mods/redpower_2/content/data/redpower_2/horizon/item/screwdriver.json, mods/redpower_2/content/data/redpower_2/horizon/item/diamond_drawplate.json, mods/redpower_2/content/data/redpower_2/horizon/item/dye_indigo.json, mods/redpower_2/content/data/redpower_2/recipe/iron_handsaw.json, mods/redpower_2/content/data/redpower_2/recipe/diamond_handsaw.json, mods/redpower_2/content/data/redpower_2/recipe/screwdriver.json
All tool items, recipes, durabilities, and model references verified.
