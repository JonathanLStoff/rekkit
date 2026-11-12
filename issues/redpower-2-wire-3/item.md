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
beads_id: rk-qst.3.275
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: wire filter into lib.rs with native GUI

lib.rs owns registration (not in domain-builder scope for src/machine); needs native_screen at contract slot positions instead of generic_3x3

## Beads import

- Beads id: `rk-qst.3.275`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:40:38Z
- Closed (Beads): 2026-11-03T03:51:18Z
- Close reason (Beads): filter: native_screen 3x3 at (62,17), colour button (118..129,55..66) cycles 0..16 via native_menu_button, nbt color

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-03T03:51:14Z · Jonathan Stoff] [domain-builder] done: filter: native_screen 3x3 at (62,17), colour button (118..129,55..66) cycles 0..16 via native_menu_button, nbt color
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Layouts from Java Container*/Gui*. cargo test pass (270+17), clippy clean for new code. Only gap: no contract file exists for buffer_chest/filter/battery_box (only item_detector); GUI background textures/charge bars not drawn (native_screen has no draw API used here); battery box has no per-tick wiring of BatteryBoxState/BufferChestState into entity items (container items only), and sided faces for buffer chest use default rotation 3.
