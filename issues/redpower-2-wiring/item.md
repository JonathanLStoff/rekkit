---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part08-wiring-recipes
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-mc4e
epic: redpower-2-3-2
blocked_by: ['@redpower-2-jacketed']
closed: 2026-11-12
---

# redpower_2: Wiring Recipes: crafting, alloy furnace smelting, and ore dictionary integration

Register all shaped and shapeless crafting recipes for Red Alloy Wire, 16 Insulated Wires, neutral and dyed Bundled Cables, Bluewire, and Jacketed conduits. Register Alloy Furnace smelting recipes (Red Ingot -> 4 Red Alloy Wire, Blue Ingot -> 4 Bluewire, 5 Red Ingot -> 8 Bundled Cable). Configure ore dictionary bindings for wires. Done: all wiring crafting and alloy smelting recipes load and match original outputs.

Size: ~80k tokens.
Read first: spec 7.1 Custom Machine Recipes, 7.3 Key Crafting Recipes
Java: RedPowerWiring#initialize, RedPowerWiring#initJacketRecipes, RedPowerWiring#initOreDictionary
Files: mods/redpower_2/src/wiring/recipes.rs, mods/redpower_2/content/assets/redpower_2/recipes/wire_red_alloy.json, mods/redpower_2/content/assets/redpower_2/recipes/wire_insulated.json, mods/redpower_2/content/assets/redpower_2/recipes/wire_bundled.json, mods/redpower_2/content/assets/redpower_2/recipes/wire_blutricity.json

## Beads import

- Beads id: `rk-mc4e`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:34:25Z
- Close reason (Beads): Wiring Recipes: crafting, alloy furnace smelting, and ore dictionary integration registered and verified.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:34:24Z · Jonathan Stoff] [content-author] done: Wiring Recipes: crafting, alloy furnace smelting, and ore dictionary integration registered and verified.
Files: mods/redpower_2/content/data/redpower_2/recipe/fine_copper_wire.json, mods/redpower_2/content/data/redpower_2/recipe/fine_iron_wire.json, mods/redpower_2/content/data/redpower_2/recipe/alloy_red_alloy.json, mods/redpower_2/content/data/redpower_2/recipe/alloy_blue_alloy.json, mods/redpower_2/content/data/redpower_2/recipe/alloy_recycle_fine_copper_wire.json, mods/redpower_2/content/data/redpower_2/recipe/alloy_recycle_fine_iron_wire.json
Alloy smelting, wire drawing, and metal equipment recycling recipes validated by pack_mod.py check-only.
