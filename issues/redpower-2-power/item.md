---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-gui-containers-power
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-j314
epic: redpower-2-3-2
blocked_by: ['@redpower-2-battery-3', '@redpower-2-base', '@redpower-2-blulectric-2', '@redpower-2-blulectric-4']
closed: 2026-11-12
---

# redpower_2: Power & Furnaces GUIs & Containers: Battery Box, Charging Bench, Wind Turbine, Furnaces

Implement Containers and GUIs for power storage, wind generation, and electric furnaces: Battery Box (battery slot and charge bar), Charging Bench (16 tool/battery charging slots), Wind Turbine (rotor slot and efficiency gauge), Blulectric Furnace (smelting slots and power meter), and Blulectric Alloy Furnace (3x3 alloy input grid and power meter). Write contract files for all 5 blocks. Done: make contract-check clean for battery_box, charging_bench, wind_turbine, blulectric_furnace, and blulectric_alloy_furnace; level 3 GUI slots layout, energy meter sync, and progress gauges.

Size: ~110k tokens.
Covers: battery_box, charging_bench, wind_turbine, blulectric_furnace, blulectric_alloy_furnace, wood_sail
Read first: spec 2.5 Blutricity Power Grid, 3.1 Appliance Machines, 3.3 Power & Fluid Machines
Java: com.eloraam.redpower.machine.ContainerBatteryBox#ctor, com.eloraam.redpower.machine.GuiBatteryBox#drawGuiContainerBackgroundLayer, com.eloraam.redpower.machine.ContainerChargingBench#ctor, com.eloraam.redpower.machine.ContainerWindTurbine#ctor, com.eloraam.redpower.machine.ContainerBlueFurnace#ctor, com.eloraam.redpower.machine.ContainerBlueAlloyFurnace#ctor
Files: mods/redpower_2/src/machine/gui_power.rs

## Beads import

- Beads id: `rk-j314`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:40:51Z
- Close reason (Beads): Verified and generated contracts and content files for power/furnace blocks (battery_box, charging_bench, wind_turbine, blulectric_furnace, blulectric_alloy_furnace, wood_sail)

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T17:40:48Z · unknown] [content-author] done: Verified and generated contracts and content files for power/furnace blocks (battery_box, charging_bench, wind_turbine, blulectric_furnace, blulectric_alloy_furnace, wood_sail)
