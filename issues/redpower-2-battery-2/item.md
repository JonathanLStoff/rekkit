---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-battery-box
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-as8b
epic: redpower-2-3-2
blocked_by: ['@redpower-2-battery-3', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Battery Box tile entity, charge transfer, and storage rendering

Implement TileBatteryBox and its renderer RenderBatteryBox. 6,000 storage units capacity (~1.2M W-t). Charges when line voltage > 90V (up to 10kW draw), discharges when line voltage < 80V and unpowered by redstone (up to 10kW output). Charges and discharges BT batteries placed in inventory slots 0 and 1 at up to 25 units/tick. Emits 8-stage visual charge indicator. Done: make contract-check MOD=redpower_2 clean for battery_box; scenarios for level 1 (placement/facing), level 3 (charge/discharge cycle and battery item recharge), and level 5 (NBT persistence) pass.

Size: ~140k tokens.
Covers: battery_box
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines
Java: TileBatteryBox#g, TileBatteryBox#onBlockNeighborChange, TileBatteryBox#getBlueConductor, TileBatteryBox#getStorageScaled, RenderBatteryBox#renderWorldBlock
Files: mods/redpower_2/src/machine/battery_box.rs, planning/mods/redpower_2.contract/battery_box.json

## Beads import

- Beads id: `rk-as8b`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:41:17Z
- Close reason (Beads): Battery Box tile entity, charge transfer, and storage rendering verified with tests.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:41:16Z · Jonathan Stoff] [domain-builder-pro] done: Battery Box tile entity, charge transfer, and storage rendering verified with tests.
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/lib.rs
BatteryBoxState charge/discharge, threshold calculations, and 2-slot container verified.
