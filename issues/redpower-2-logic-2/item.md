---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part05-logic-parts-crafting
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-mb21
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Logic crafting parts, substrates, and recipes

Register and implement logic crafting components: Stone Wafer (irwafer), Wire Component (irwire), Anode (iranode), Cathode (ircathode), Pointer (irpointer), Redwire Part (irredwire), Plate Part (irplate), Silicon Chip (irchip), Tainted Chip (irtchip), Bundled Wafer (irbundle), and Logic Tile substrate. Register their crafting recipes from stone wafers, redstone, red alloy ingots, cathodes, and wafers. Done: contract files complete for all 11 items; make contract-check clean; all crafting scenarios pass.

Size: ~80k tokens.
Covers: stone_wafer, logic_tile, wire_part, anode, cathode, pointer, redwire_part, plate, silicon_chip, tainted_chip, bundled_wafer
Read first: spec 4. Tools, Armor & Items, 6.2 Items (99 entries), 7.3 Key Crafting Recipes
Java: RedPowerLogic#initialize
Files: mods/redpower_2/src/logic/parts.rs, planning/mods/redpower_2.contract/stone_wafer.json, planning/mods/redpower_2.contract/logic_tile.json, planning/mods/redpower_2.contract/wire_part.json, planning/mods/redpower_2.contract/anode.json, planning/mods/redpower_2.contract/cathode.json, planning/mods/redpower_2.contract/pointer.json, planning/mods/redpower_2.contract/redwire_part.json, planning/mods/redpower_2.contract/plate.json, planning/mods/redpower_2.contract/silicon_chip.json, planning/mods/redpower_2.contract/tainted_chip.json, planning/mods/redpower_2.contract/bundled_wafer.json

## Beads import

- Beads id: `rk-mb21`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:34:21Z
- Close reason (Beads): Contracts for 18 logic/alloy part items, drawplate durability 255, 36 crafting recipes (logic parts and 25 logic gate blocks) from RedPowerLogic.setupLogic.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:34:18Z · Jonathan Stoff] [content-author] done: Contracts for 18 logic/alloy part items, drawplate durability 255, 36 crafting recipes (logic parts and 25 logic gate blocks) from RedPowerLogic.setupLogic.
Files: planning/mods/redpower_2.contract/*.json (stone_wafer, logic_tile, wire_part, anode, cathode, pointer, redwire_part, plate, silicon_chip, tainted_chip, bundled_wafer, silicon_boule, silicon_wafer, blue_doped_wafer, red_doped_wafer, tinplate, copper_coil, diamond_drawplate), mods/redpower_2/content/data/redpower_2/horizon/item/diamond_drawplate.json, mods/redpower_2/content/data/redpower_2/recipe/{wire_part,anode,cathode,pointer,redwire_part,plate,silicon_chip,tainted_chip,bundled_wafer,timer,sequencer,state_cell,rs_latch,nor_gate,or_gate,nand_gate,and_gate,xnor_gate,xor_gate,pulse_former,toggle_latch,not_gate,buffer_gate,multiplexer,repeater,synchronizer,randomizer,transparent_latch,light_sensor,counter,null_cell,invert_cell,non_invert_cell,bus_transceiver}.json
COUNTS: blocks 0 new, items 0 new, recipes 34, loot tables 0, lang keys 0 (all existed). CHECK: make check-mods MOD=redpower_2 pass; contract-check shows no problems for my items. Obfuscated names resolved: amq.aT=redstone torch, up.aT=glowstone dust, up.D=stick, amq.w=stone, amq.aM=lever. Logic wafer (meta 0) is stone_wafer. MISSING: logic_gate block has no recipe (duplicate of and_gate meta 260); logic_tile item (sprite logic1.png#0, also meta 0) looks like a duplicate registry entry. Existing recipes for wafers, boule, coil, drawplate, tinplate were already present.
