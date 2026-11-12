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
beads_id: rk-qst.3.268
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: wire TileItemDetector into lib.rs (BE, GUI, signal, tube hook)

lib.rs is outside domain-builder ownership of item_detector.rs; engine layer must call the pure logic.

## Beads import

- Beads id: `rk-qst.3.268`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:39:00Z
- Closed (Beads): 2026-11-03T03:51:12Z
- Close reason (Beads): item_detector: native_screen 3x3 at (62,17), mode button (117..131,54..68) cycles 0..2 via native_menu_button, stored in nbt mode

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-03T03:51:08Z · Jonathan Stoff] [domain-builder] done: item_detector: native_screen 3x3 at (62,17), mode button (117..131,54..68) cycles 0..2 via native_menu_button, stored in nbt mode
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Layouts from Java Container*/Gui*. cargo test pass (270+17), clippy clean for new code. Only gap: no contract file exists for buffer_chest/filter/battery_box (only item_detector); GUI background textures/charge bars not drawn (native_screen has no draw API used here); battery box has no per-tick wiring of BatteryBoxState/BufferChestState into entity items (container items only), and sided faces for buffer chest use default rotation 3.
