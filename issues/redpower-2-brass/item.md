---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part10-brass-gear
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-zu0c
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Brass gear: item registration, crafting recipes and asset verification

Register the Brass Gear item (ItemParts itemCompatParts meta 0, sprite /eloraam/compat/compat1.png#1, max stack 64) and crafting recipes: standard recipe (4x brass ingot surrounding BuildCraft stone gear) and alternative recipe (4x brass ingot surrounding iron gear microblock). Ensure item model and texture sliced from eloraam/compat/compat1.png#1 are properly bound. Write planning/mods/redpower_2.contract/brass_gear.json with sprite, max_stack, and crafting scenario. Done: make contract-check clean for brass_gear.

Size: ~50k tokens.
Covers: brass_gear
Read first: spec 4. Tools, Armor & Items, 6.2 Items, 7.3 Key Crafting Recipes
Java: RedPowerCompat#setupBlocks
Files: mods/redpower_2/content/assets/redpower_2/items/brass_gear.json, mods/redpower_2/content/assets/redpower_2/models/item/brass_gear.json, mods/redpower_2/content/data/redpower_2/recipe/brass_gear.json, planning/mods/redpower_2.contract/brass_gear.json

## Beads import

- Beads id: `rk-zu0c`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:33:19Z
- Close reason (Beads): Brass gear: contract, recipe added; item/model/lang/tab verified

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:33:18Z · Jonathan Stoff] [content-author] done: Brass gear: contract, recipe added; item/model/lang/tab verified
Files: planning/mods/redpower_2.contract/brass_gear.json, mods/redpower_2/content/data/redpower_2/recipe/brass_gear.json
Recipe is the original's alt recipe (brass ring around a center); center uses minecraft:iron_ingot as stand-in for microblock 5649 (unverified material). BuildCraft stone-gear recipe omitted (no BC). Texture compat1.png#1 not pixel-compared by me; contract-check shows no brass_gear issue.
