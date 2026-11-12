---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-crafting-tools
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-2jv4
epic: redpower-2-3-2
blocked_by: ['@redpower-2-metallurgy']
closed: 2026-11-12
---

# redpower_2: Crafting Tools: Handsaws, Screwdriver, Diamond Drawplate & Indigo Dye

Implement core RedPower crafting tools: Iron Handsaw (durability 320, sharpness 0), Diamond Handsaw (durability 1280, sharpness 2) with damage on craft in microblock cutting; Screwdriver (lossless remainder, right-click rotates blocks, shift+right-click cycles operational modes); Diamond Drawplate (durability 255, draws copper/iron ingots into fine wire); and Indigo Dye (dyes wool and sheep blue, dye crafting combinations). Done: make contract-check clean for all 6 tool items; their level 2 scenarios pass.

Size: ~110k tokens.
Covers: iron_handsaw, handsaw, diamond_handsaw, screwdriver, diamond_drawplate, dye_indigo
Read first: spec 4.1 Metallurgy & Crafting Tools
Java: ItemHandsaw#getSharpness, ItemScrewdriver#onItemUseFirst, ItemDrawplate#<init>, ItemDyeIndigo#a, RedPowerBase#initIndigo
Files: mods/redpower_2/src/base/tools.rs, planning/mods/redpower_2.contract/iron_handsaw.json, planning/mods/redpower_2.contract/handsaw.json, planning/mods/redpower_2.contract/diamond_handsaw.json, planning/mods/redpower_2.contract/screwdriver.json, planning/mods/redpower_2.contract/diamond_drawplate.json, planning/mods/redpower_2.contract/dye_indigo.json

## Beads import

- Beads id: `rk-2jv4`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:38:15Z
- Close reason (Beads): Wrote contracts for 6 handsaws, screwdriver, dye_indigo; set max_damage on handsaws (320/1280/640); contract-check and check-mods clean for these items.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:38:13Z · Jonathan Stoff] [content-author] done: Wrote contracts for 6 handsaws, screwdriver, dye_indigo; set max_damage on handsaws (320/1280/640); contract-check and check-mods clean for these items.
Files: planning/mods/redpower_2.contract/iron_handsaw.json, planning/mods/redpower_2.contract/handsaw.json, planning/mods/redpower_2.contract/diamond_handsaw.json, planning/mods/redpower_2.contract/ruby_handsaw.json, planning/mods/redpower_2.contract/green_sapphire_handsaw.json, planning/mods/redpower_2.contract/sapphire_handsaw.json, planning/mods/redpower_2.contract/screwdriver.json, planning/mods/redpower_2.contract/dye_indigo.json, mods/redpower_2/content/data/redpower_2/horizon/item/*_handsaw.json
Drawplate contract already existed and passes. Out of scope and still open: sonic_screwdriver (scenario, durability 400, sprite 129), indigo_flower (cross shape, sprite, scenarios).
