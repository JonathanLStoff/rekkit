---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-base-resources
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-j06b
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-2']
closed: 2026-11-12
---

# redpower_2: Base items: resources, alloys, nuggets, and lumar colors

Register all RedPowerBase items: gems (ruby, green sapphire, sapphire), ingots (silver, tin, copper), nikolite dust, nuggets (iron, silver, tin, copper), alloy metals (red alloy, blue alloy, brass, tinplate), silicon boule and wafers (plain, red-doped, blue-doped), fine wires (copper, iron), copper coil, BL motor, canvas, and 16 lumar colors. Done: all item IDs registered, ore dictionary tags set, and stack sizes matching the spec.

Size: ~120k tokens.
Covers: ruby, green_sapphire, sapphire, silver_ingot, tin_ingot, copper_ingot, nikolite, iron_nugget, silver_nugget, tin_nugget, copper_nugget, red_alloy_ingot, blue_alloy_ingot, brass_ingot, silicon_boule, silicon_wafer, blue_doped_wafer, red_doped_wafer, tinplate, fine_copper_wire, fine_iron_wire, copper_coil, bl_motor, canvas, lumar
Read first: spec Section 4.1 Metallurgy & Crafting Tools, Section 6.2 Items (99 entries), Section 7.3 Key Crafting Recipes
Java: RedPowerBase#initBaseItems, RedPowerBase#initAlloys, ItemParts#addItem
Files: mods/redpower_2/src/items/resources.rs

## Beads import

- Beads id: `rk-j06b`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:29:25Z
- Close reason (Beads): Base items registered: gems, ingots, nikolite, nuggets, alloys, silicon boules/wafers, fine wires, coil, motor, canvas, and lumar.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:29:23Z · Jonathan Stoff] [content-author] done: Base items registered: gems, ingots, nikolite, nuggets, alloys, silicon boules/wafers, fine wires, coil, motor, canvas, and lumar.
Files: mods/redpower_2/content/data/redpower_2/horizon/item/ruby.json, mods/redpower_2/content/data/redpower_2/horizon/item/green_sapphire.json, mods/redpower_2/content/data/redpower_2/horizon/item/sapphire.json, mods/redpower_2/content/data/redpower_2/horizon/item/silver_ingot.json, mods/redpower_2/content/data/redpower_2/horizon/item/tin_ingot.json, mods/redpower_2/content/data/redpower_2/horizon/item/copper_ingot.json, mods/redpower_2/content/data/redpower_2/horizon/item/nikolite.json, mods/redpower_2/content/data/redpower_2/horizon/item/iron_nugget.json, mods/redpower_2/content/data/redpower_2/horizon/item/silver_nugget.json, mods/redpower_2/content/data/redpower_2/horizon/item/tin_nugget.json, mods/redpower_2/content/data/redpower_2/horizon/item/copper_nugget.json, mods/redpower_2/content/data/redpower_2/horizon/item/red_alloy_ingot.json, mods/redpower_2/content/data/redpower_2/horizon/item/blue_alloy_ingot.json, mods/redpower_2/content/data/redpower_2/horizon/item/brass_ingot.json, mods/redpower_2/content/data/redpower_2/horizon/item/silicon_boule.json, mods/redpower_2/content/data/redpower_2/horizon/item/silicon_wafer.json, mods/redpower_2/content/data/redpower_2/horizon/item/blue_doped_wafer.json, mods/redpower_2/content/data/redpower_2/horizon/item/red_doped_wafer.json, mods/redpower_2/content/data/redpower_2/horizon/item/tinplate.json, mods/redpower_2/content/data/redpower_2/horizon/item/fine_copper_wire.json, mods/redpower_2/content/data/redpower_2/horizon/item/fine_iron_wire.json, mods/redpower_2/content/data/redpower_2/horizon/item/copper_coil.json, mods/redpower_2/content/data/redpower_2/horizon/item/bl_motor.json, mods/redpower_2/content/data/redpower_2/horizon/item/canvas.json, mods/redpower_2/content/data/redpower_2/horizon/item/lumar.json
All item IDs, tags, stack sizes, and models registered and validated by pack_mod.py check-only.
