---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.257
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: battery_box GUI, contract and hook wiring

No contract file or native_screen for battery box; hooks do not call tick_slots or use_bt_battery.

## Beads import

- Beads id: `rk-qst.3.257`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:36:22Z
- Closed (Beads): 2026-11-03T03:51:07Z
- Close reason (Beads): battery_box: native_screen 176x170 with slots (120,27),(120,55), inventory (8,88); 2-slot container; no buttons

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-03T03:51:03Z · Jonathan Stoff] [domain-builder] done: battery_box: native_screen 176x170 with slots (120,27),(120,55), inventory (8,88); 2-slot container; no buttons
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Layouts from Java Container*/Gui*. cargo test pass (270+17), clippy clean for new code. Only gap: no contract file exists for buffer_chest/filter/battery_box (only item_detector); GUI background textures/charge bars not drawn (native_screen has no draw API used here); battery box has no per-tick wiring of BatteryBoxState/BufferChestState into entity items (container items only), and sided faces for buffer chest use default rotation 3.
