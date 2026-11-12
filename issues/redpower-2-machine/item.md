---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-machine-panels-blocks
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-1uy1
epic: redpower-2-3-2
blocked_by: ['@redpower-2-magtubes', '@redpower-2-fluid-3', '@redpower-2-solar-4', '@redpower-2-grate']
closed: 2026-11-12
---

# redpower_2: Machine Panel blocks: Solar Panel, Pump, Accelerator, Grate, Transformer

Implement machine panel block states and placement from BlockMachinePanel and ItemMachinePanel: Solar Panel (meta 0, 1/4 slab height), Fluid Pump (meta 1), Magtube Accelerator (meta 2), Fluid Grate (meta 3), and Blutricity Transformer (meta 4, 100x step-up/step-down). Sets up block shapes, orientations, and surface attachment bounds. Write contract files for all 5 machine panels. Done: make contract-check clean for solar_panel, pump, accelerator, grate, and transformer; level 1 placement, face sprites, and hitboxes.

Size: ~100k tokens.
Covers: solar_panel, pump, accelerator, grate, transformer
Read first: spec 3.3 Power & Fluid Machines
Java: com.eloraam.redpower.machine.BlockMachinePanel#getBlockTextureFromSideAndMetadata, com.eloraam.redpower.machine.BlockMachinePanel#getCollisionBoundingBoxFromPool, com.eloraam.redpower.machine.ItemMachinePanel#getItemNameIS
Files: mods/redpower_2/src/machine/machine_panels.rs

## Beads import

- Beads id: `rk-1uy1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:21:31Z
- Close reason (Beads): Machine Panel blocks: Solar Panel, Pump, Accelerator, Grate, Transformer block shapes, placement, and orientation logic implemented and contracts verified

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T17:21:28Z · unknown] [content-author] done: Machine Panel blocks: Solar Panel, Pump, Accelerator, Grate, Transformer block shapes, placement, and orientation logic implemented and contracts verified
