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
beads_id: rk-qst.3.248
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: wire buffer_chest container/native_screen to BufferChestState

domain logic exists; no engine container yet

## Beads import

- Beads id: `rk-qst.3.248`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:35:18Z
- Closed (Beads): 2026-11-03T03:51:01Z
- Close reason (Beads): buffer_chest: native_screen 176x186, 20 slots (44+18c, 18+18r), inventory (8,104); container 27-slot 9x3 backing with 20 sided slots

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-03T03:50:57Z · Jonathan Stoff] [domain-builder] done: buffer_chest: native_screen 176x186, 20 slots (44+18c, 18+18r), inventory (8,104); container 27-slot 9x3 backing with 20 sided slots
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Layouts from Java Container*/Gui*. cargo test pass (270+17), clippy clean for new code. Only gap: no contract file exists for buffer_chest/filter/battery_box (only item_detector); GUI background textures/charge bars not drawn (native_screen has no draw API used here); battery box has no per-tick wiring of BatteryBoxState/BufferChestState into entity items (container items only), and sided faces for buffer chest use default rotation 3.
