---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-base-resources
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-cgl3
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-5']
closed: 2026-11-12
---

# redpower_2: Metallurgy & Base Resource Items (Ingots, Gems, Alloys, Lumars, Nuggets)

Implement all RedPower Base metal ingots, gems, alloys, doped wafers, silicon boules, coils, motors, nuggets, and 16-color Lumars. Configures item registration, max stack size (64), sprites from eloraam/base/items1.png, and contract definitions. Done: make contract-check clean for all 24 resource items; their scenarios pass.

Size: ~100k tokens.
Covers: lumar, ruby, green_sapphire, sapphire, silver_ingot, tin_ingot, copper_ingot, nikolite, red_alloy_ingot, blue_alloy_ingot, brass_ingot, silicon_boule, silicon_wafer, blue_doped_wafer, red_doped_wafer, tinplate, fine_copper_wire, fine_iron_wire, copper_coil, bl_motor, iron_nugget, silver_nugget, tin_nugget, copper_nugget
Read first: spec 4. Tools, Armor & Items, 6.2 Items
Java: RedPowerBase#initBaseItems, ItemParts#addItem, ItemParts#getTextureFile
Files: mods/redpower_2/src/base/resources.rs, planning/mods/redpower_2.contract/lumar.json, planning/mods/redpower_2.contract/ruby.json, planning/mods/redpower_2.contract/green_sapphire.json, planning/mods/redpower_2.contract/sapphire.json, planning/mods/redpower_2.contract/silver_ingot.json, planning/mods/redpower_2.contract/tin_ingot.json, planning/mods/redpower_2.contract/copper_ingot.json, planning/mods/redpower_2.contract/nikolite.json, planning/mods/redpower_2.contract/red_alloy_ingot.json, planning/mods/redpower_2.contract/blue_alloy_ingot.json, planning/mods/redpower_2.contract/brass_ingot.json, planning/mods/redpower_2.contract/silicon_boule.json, planning/mods/redpower_2.contract/silicon_wafer.json, planning/mods/redpower_2.contract/blue_doped_wafer.json, planning/mods/redpower_2.contract/red_doped_wafer.json, planning/mods/redpower_2.contract/tinplate.json, planning/mods/redpower_2.contract/fine_copper_wire.json, planning/mods/redpower_2.contract/fine_iron_wire.json, planning/mods/redpower_2.contract/copper_coil.json, planning/mods/redpower_2.contract/bl_motor.json, planning/mods/redpower_2.contract/iron_nugget.json, planning/mods/redpower_2.contract/silver_nugget.json, planning/mods/redpower_2.contract/tin_nugget.json, planning/mods/redpower_2.contract/copper_nugget.json

## Beads import

- Beads id: `rk-cgl3`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:35:36Z
- Close reason (Beads): Wrote 18 missing contract files for the 24 base resource items (other 6 existed); item JSONs/models/lang/loot already present; contract-check clean for these items.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:35:34Z · Jonathan Stoff] [content-author] done: Wrote 18 missing contract files for the 24 base resource items (other 6 existed); item JSONs/models/lang/loot already present; contract-check clean for these items.
Files: planning/mods/redpower_2.contract/lumar.json, planning/mods/redpower_2.contract/ruby.json, planning/mods/redpower_2.contract/sapphire.json, planning/mods/redpower_2.contract/green_sapphire.json, planning/mods/redpower_2.contract/nikolite.json, planning/mods/redpower_2.contract/*_ingot.json, planning/mods/redpower_2.contract/*_nugget.json, planning/mods/redpower_2.contract/fine_*_wire.json, planning/mods/redpower_2.contract/bl_motor.json
COUNTS: 0 new blocks, 18 contracts, 0 recipes. CHECK: check-mods pass. Contracts derived from features.json; lumar/nikolite scenarios are generic use+inventory checks. Lumar item JSON uses a single glowstone_dust template, not 16 colours (not verified).
