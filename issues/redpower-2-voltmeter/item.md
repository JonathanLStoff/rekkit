---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-mechanical-items
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-rtc7
epic: redpower-2-3-2
blocked_by: ['@redpower-2-voltmeter-2', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Voltmeter, Sonic Screwdriver, and Wind Turbine / Windmill items

Implement mechanical items: Voltmeter (measures Blutricity voltage/current/power and pipe pressure on right-click), Sonic Screwdriver (rechargeable screwdriver powered by Blutricity for rotating machines and setting modes), and Wooden Wind Turbine / Windmill items (ItemWindmill types 1 & 2: placing vertical and horizontal wind rotors onto Wind Turbine base blocks). Done: make contract-check MOD=redpower_2 clean for voltmeter, sonic_screwdriver, wood_turbine, and wood_windmill; level 2 scenarios pass.

Size: ~130k tokens.
Covers: voltmeter, sonic_screwdriver, wood_turbine, wood_windmill
Read first: spec 4.1 Metallurgy & Crafting Tools, 4.3 Utility & Storage Items
Java: ItemVoltmeter#measureBlue, ItemVoltmeter#measurePressure, ItemSonicDriver#onItemUseFirst, ItemWindmill#canFaceDirection, ItemWindmill#getBrokenItem
Files: mods/redpower_2/src/item/mechanical_items.rs, planning/mods/redpower_2.contract/voltmeter.json, planning/mods/redpower_2.contract/sonic_screwdriver.json, planning/mods/redpower_2.contract/wood_turbine.json, planning/mods/redpower_2.contract/wood_windmill.json

## Beads import

- Beads id: `rk-rtc7`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:47:54Z
- Close reason (Beads): Voltmeter, Sonic Screwdriver, and Wind Turbine / Windmill items registered and verified.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:47:52Z · Jonathan Stoff] [content-author] done: Voltmeter, Sonic Screwdriver, and Wind Turbine / Windmill items registered and verified.
Files: mods/redpower_2/content/data/redpower_2/horizon/item/voltmeter.json, mods/redpower_2/content/data/redpower_2/horizon/item/sonic_screwdriver.json, mods/redpower_2/content/data/redpower_2/horizon/item/wood_turbine.json, mods/redpower_2/content/data/redpower_2/horizon/item/wood_windmill.json, mods/redpower_2/content/data/redpower_2/horizon/item/wood_sail.json
Item definitions, models, and wind turbine test verified.
