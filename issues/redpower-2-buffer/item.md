---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-buffer-chest
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-puh9
epic: redpower-2-3-2
blocked_by: ['@redpower-2-buffer-2', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Buffer Chest partitioned sided inventory

Implement TileBufferChest and RenderBufferChest for buffer_chest and buffer. 20-slot buffer decoupling high-speed extraction from processing. Rear face connects to all 20 inventory slots; each side face connects to a dedicated 4-slot partition (left, right, top, bottom). Rotatable with screwdriver. Done: make contract-check MOD=redpower_2 clean for buffer_chest and buffer; level 1, 3, 5 and 6 (partitioned sided access) scenarios pass.

Size: ~100k tokens.
Covers: buffer_chest, buffer
Read first: spec 3.1 Appliance Machines
Java: TileBufferChest#onBlockActivated, TileBufferChest#getStartInventorySide, TileBufferChest#getSizeInventorySide, RenderBufferChest#renderWorldBlock
Files: mods/redpower_2/src/machine/buffer_chest.rs, planning/mods/redpower_2.contract/buffer_chest.json, planning/mods/redpower_2.contract/buffer.json

## Beads import

- Beads id: `rk-puh9`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:41:23Z
- Close reason (Beads): Buffer Chest partitioned sided inventory verified with tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:41:21Z · Jonathan Stoff] [content-author] done: Buffer Chest partitioned sided inventory verified with tests.
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/lib.rs
BufferChestState 20-slot inventory, 4-slot partitioned sides, and native_screen GUI verified by test_buffer_chest_partitions.
