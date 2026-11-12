---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.313
epic: redpower-2-3-2
closed: 2026-11-12
---

# Blutricity devices in lib.rs still model charge with the simplified conductor

SolarPanelState/ThermopileState/WindTurbineState/BatteryBoxState/TransformerState call add_charge()/charge/capacitance and divide power by voltage instead of the original's applyDirect/applyPower/drawPower on the dynamic conductor, so they do not share the grid's Vcap/Icap timing. BatteryBoxState is also 5x off: Java adds n storage units per n*1000 W draw (TileBatteryBox.g lines 142-152), lib.rs adds draw_w/200. These structs belong to rk-30hm, rk-2mws, rk-8tpx and rk-as8b; whoever takes them should hold a core::conductor::BlueConductor and call apply_power/draw_power/apply_direct with the device tick.

## Beads import

- Beads id: `rk-qst.3.313`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-03T22:04:59Z
- Closed (Beads): 2026-11-08T14:03:03Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-08T14:02:53Z · unknown] [domain-builder] complete: Updated Blutricity devices and BluePowerConductor in lib.rs to implement draw_power, apply_power, apply_direct, and fixed BatteryBoxState charging/discharging scaling (1 unit per 1000 W) matching TileBatteryBox.g in Java.
