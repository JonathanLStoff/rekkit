---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-machine-item-battery
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-khla
epic: redpower-2-3-2
blocked_by: ['@redpower-2-battery-3']
closed: 2026-11-12
---

# redpower_2: BT Battery items: empty and powered portable Blutricity storage

Implement the portable BT Battery items: empty battery and charged/powered battery (ItemBattery). Stores up to 1,200 BT charge units, tracks voltage level, and integrates with Battery Box, Charging Bench, and portable electric tools. Write contract files for battery, battery_powered, and battery_empty. Done: make contract-check clean for battery, battery_powered, and battery_empty; level 2 item use, charge state tracking, and stack limits.

Size: ~60k tokens.
Covers: battery_powered, battery, battery_empty
Read first: spec 2.5 Blutricity Power Grid, 4.3 Utility & Storage Items
Java: com.eloraam.redpower.machine.ItemBattery#getItemNameIS, com.eloraam.redpower.machine.ItemBattery#onItemUseFirst, com.eloraam.redpower.RedPowerMachine#setupItems
Files: mods/redpower_2/src/machine/battery_item.rs

## Beads import

- Beads id: `rk-khla`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:41:20Z
- Close reason (Beads): BT Battery items: empty and powered portable Blutricity storage registered and tested.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:41:19Z · Jonathan Stoff] [content-author] done: BT Battery items: empty and powered portable Blutricity storage registered and tested.
Files: mods/redpower_2/content/data/redpower_2/horizon/item/battery_empty.json, mods/redpower_2/content/data/redpower_2/horizon/item/battery_powered.json, mods/redpower_2/content/data/redpower_2/horizon/item/battery.json, mods/redpower_2/src/lib.rs
Items registered, models and textures verified, test_bt_battery_use passing.
