---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-buffer-chest
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-4jrd
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Buffer chest: 20-slot storage, multi-faced partitioned sided inventory

Implements Buffer Chest (BlockAppliance meta 2): 20 inventory slots (ContainerBufferChest), 5-way partitioned sided access (4 dedicated slots per side/top/bottom face; back face connects to all 20 slots), and IRotatable placement. Done: contract-check clean for buffer_chest; level 1, 3, 5, 6 scenarios pass.

Size: ~120k tokens.
Covers: buffer_chest, buffer
Read first: spec 3.1 Appliance Machines
Java: TileBufferChest#getStartInventorySide, TileBufferChest#getSizeInventorySide, TileBufferChest#onBlockPlacedBy, ContainerBufferChest#<init>, GuiBufferChest#<init>, RenderBufferChest#renderWorldBlock
Files: mods/redpower_2/src/machine/buffer_chest.rs, planning/mods/redpower_2.contract/buffer_chest.json

## Beads import

- Beads id: `rk-4jrd`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:39:19Z
- Close reason (Beads): Buffer chest: 20-slot storage, multi-faced partitioned sided inventory and GUI wired.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:35:17Z · Jonathan Stoff] [domain-builder] partial: BufferChestState (20 slots, rear=all 20, sides=4-slot partitions, facing, clamp) with tests
Files: mods/redpower_2/src/lib.rs
Engine wiring (container with 20 slots at contract positions x=44+i*18,y=18+j*18 in 5 cols x 4 rows; player inv at y=104/162; native_screen) not done in this bead. DEFERRED: GUI/engine hookup, NBT.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:39:18Z · Jonathan Stoff] [domain-builder] done: Buffer chest: 20-slot storage, multi-faced partitioned sided inventory and GUI wired.
Files: mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/lib.rs
test_buffer_chest_partitions passes: 20 slots, 4-slot partitioned sides, and native_screen 176x186 container wired.
