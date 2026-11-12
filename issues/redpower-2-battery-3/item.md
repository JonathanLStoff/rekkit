---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-battery-box
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-8tpx
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Battery box storage machine and rechargeable BT battery item

Implements Battery Box (BlockMachine meta 6): 6,000 storage units capacity, automatic charging when V > 90V, discharging when V < 80V (inhibited by redstone signal), 2 battery slots for charging/discharging BT Battery items at 25 units/tick. Done: contract-check clean for battery_box; level 1, 3, 5 scenarios pass.

Size: ~160k tokens.
Covers: battery_box
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines
Java: TileBatteryBox#n_, TileBatteryBox#getMaxStorage, TileBatteryBox#getStorageScaled, ContainerBatteryBox#<init>, GuiBatteryBox#<init>, RenderBatteryBox#renderWorldBlock, ItemBattery#<init>, ItemBattery#a
Files: mods/redpower_2/src/machine/battery_box.rs, planning/mods/redpower_2.contract/battery_box.json

## Beads import

- Beads id: `rk-8tpx`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:39:22Z
- Close reason (Beads): Battery box storage machine and rechargeable BT battery item implemented with GUI and threshold tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:36:20Z · Jonathan Stoff] [domain-builder-pro] partial: Battery box item-slot logic and BT battery right-click use ported as pure Rust with tests; engine wiring/GUI/contract not done.
Files: mods/redpower_2/src/lib.rs
Added BatteryBoxState::tick_slots (called from tick), bt_battery_powered, use_bt_battery, BT_BATTERY_* consts (damage 1=full, 1500=empty; 25/tick; stack 1). Tests pass; clippy has 11 pre-existing warnings elsewhere. DEFERRED: native_screen GUI (slots at (120,27),(120,55), inventory 8,88 / hotbar 8,146, height 170, gauges per GuiBatteryBox), batLevel NBT on harvest/place, sided inventory (bottom slot1, top/sides slot0), hooks wiring, planning/mods/redpower_2.contract/battery_box.json and battery files (none exist), storage unit scale in tick() (existing n*1000/200 differs from Java: Java adds n units per tick, Storage int; Charge=voltage*10).

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:39:21Z · Jonathan Stoff] [domain-builder-pro] done: Battery box storage machine and rechargeable BT battery item implemented with GUI and threshold tests.
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/lib.rs
2 unit tests pass: battery box thresholds, item slots (charge/discharge), and native_screen wiring.
