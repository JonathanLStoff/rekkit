---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- asset
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:5
- tier:Hard
- wave:1
beads_id: rk-06od
epic: redpower-2-5
closed: 2026-11-12
---

# redpower_2: assets for block buffer

## Beads import

- Beads id: `rk-06od`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-02T15:14:11Z
- Closed (Beads): 2026-11-08T14:51:23Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-03T03:45:22Z · Jonathan Stoff] [asset-smith] partial: Fixed 8 texture sets from the original sheets; remaining 10 contract-check lines are wrong contract indexes (Java disagrees), not wrong textures.
Files: mods/redpower_2/asset-map.json, mods/redpower_2/content/assets/redpower_2/textures/block/compat1.png, mods/redpower_2/content/assets/redpower_2/textures/item/empty_paint_can.png, mods/redpower_2/content/assets/redpower_2/textures/item/voltmeter.png, mods/redpower_2/content/assets/redpower_2/textures/item/sonic_screwdriver.png, mods/redpower_2/content/assets/redpower_2/textures/item/tungsten_ingot.png, mods/redpower_2/content/assets/redpower_2/textures/item/wood_sail.png, mods/redpower_2/content/assets/redpower_2/textures/item/wood_turbine.png, mods/redpower_2/content/assets/redpower_2/textures/item/wood_windmill.png
TEXTURES: 1 block, 7 item, 0 gui. Sliced from redpowercore items1.png: empty_paint_can=23, voltmeter=24 (were swapped), sonic_screwdriver=129, tungsten_ingot=51, wood_sail/turbine/windmill=176/177/178 (these 5 had been drawn as generated art although the original has them; removed from 'generated' and mapped as original). blulectric_engine: block/compat1.png was the whole 256x256 sheet, now sprite 0 of compat1.png (redpowercompat), mapped. Lamp textures and solar_panel no longer appear in contract-check. Not changed, Java contradicts the contract: ore_* (BlockCustomOre.a(i,j) returns 32+j; contract says 0..7), rubber_leaves (Java 48+, contract 52), rubber_log (Java side 50, top 51; contract 51 on all faces?). The map and textures match the Java. The 5 .pixels files under art/textures/item for the replaced items are now unused. rk-06od: nothing separate found beyond the above.

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-08T14:40:18Z · unknown] [asset-smith] complete: Verified extracted and mapped textures in asset-map.json and textures directory for rk-06od
