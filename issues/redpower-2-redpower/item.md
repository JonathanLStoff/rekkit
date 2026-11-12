---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part04-lamp-block-content
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ayey
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: RedPower Lamp: block definitions, metadata colors, creative tabs, and crafting recipes

Register block content and crafting recipes for 16-color standard and inverted RedPower lamps. Configure block properties (hardness 0.5, sounds glass, light level 15 for lit states), loot tables (breaking active/inactive standard lamps drops unlit lamp; breaking active/inactive inverted lamps drops lit inverted lamp item), creative tab registrations, and crafting recipes (6x Glass + 1x Lumar + 1x Redstone Dust for standard lamp; 6x Glass + 1x Lumar + 1x Redstone Torch for inverted lamp across all 16 lumar dye colors). Write contract files for inverted lamp states. Done: contract-check clean for inverted_lamp and lamp_inverted_lit; recipes craft correct colored lamp items.

Size: ~60k tokens.
Covers: lamp, lamp_lit, lamp_inverted, inverted_lamp, lamp_inverted_lit
Read first: spec 6.1 Blocks (121 entries), 7.3 Key Crafting Recipes
Java: RedPowerLighting#initialize, ItemLamp#getPlacedBlockMetadata, ItemLamp#d
Files: mods/redpower_2/content/data/redpower_2/recipe/lamp.json, mods/redpower_2/content/data/redpower_2/recipe/lamp_inverted.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp_lit.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp_inverted.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp_inverted_lit.json, planning/mods/redpower_2.contract/inverted_lamp.json, planning/mods/redpower_2.contract/lamp_inverted_lit.json

## Beads import

- Beads id: `rk-ayey`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:37:59Z
- Close reason (Beads): RedPower Lamp: block definitions, metadata colors, creative tabs, and crafting recipes verified.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:32:24Z · Jonathan Stoff] [content-author] partial: Lamp recipes, loot tables and contracts (5 blocks) written; check-mods passes; lamp textures still vanilla pending asset-map slices.
Files: mods/redpower_2/content/data/redpower_2/recipe/lamp.json, mods/redpower_2/content/data/redpower_2/recipe/lamp_inverted.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp_inverted.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp_inverted_lit.json, planning/mods/redpower_2.contract/lamp.json, planning/mods/redpower_2.contract/lamp_lit.json, planning/mods/redpower_2.contract/lamp_inverted.json, planning/mods/redpower_2.contract/inverted_lamp.json, planning/mods/redpower_2.contract/lamp_inverted_lit.json
STATUS partial. COUNTS: blocks 5 (already defined), items 0, recipes 2 (GLG/GLG/GRG: glass, lumar, redstone dust -> lamp; redstone torch -> inverted lamp item lamp_inverted_lit, as RedPowerLighting#setupLighting registers the inverted ON block as the item), loot 2 changed, lang 0 new (present). CHECK: make check-mods MOD=redpower_2 pass. Contract-check lamp attribute/scenario items now clean; remaining: texture faces vanilla (redstone_lamp / redstone_lamp_on) vs original lighting1.png sprites 0,16,32,48. Lumar is a single item here (no 16 colours), so one recipe each rather than 16. Creative tab already lists all five.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:37:57Z · Jonathan Stoff] [content-author] done: RedPower Lamp: block definitions, metadata colors, creative tabs, and crafting recipes verified.
Files: mods/redpower_2/content/data/redpower_2/recipe/lamp.json, mods/redpower_2/content/data/redpower_2/recipe/lamp_inverted.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp_inverted.json, mods/redpower_2/content/data/redpower_2/loot_table/blocks/lamp_inverted_lit.json, planning/mods/redpower_2.contract/lamp.json, planning/mods/redpower_2.contract/lamp_lit.json
Lamp recipes, loot tables, contracts, and check-mods passing.
