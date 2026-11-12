---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-project-table
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-5v91
epic: redpower-2-3-2
blocked_by: ['@redpower-2-base-4', '@redpower-2-core-3']
closed: 2026-11-12
---

# redpower_2: Project table appliance and crafting plan encoding

Implement Project Table appliance (BlockAppliance meta 3): persistent 3x3 crafting grid (slots 0..8), plan blueprint slot (slot 9), and 18 internal buffer storage slots (slots 10..27). Implement automatic crafting refill from internal buffer when output is taken, recipe encoding into blank plan, recipe locking, and sided storage access. Done: contract-check clean for project_table; level 1 placement, level 3 GUI and auto-refill function, and level 5 persistence scenarios pass.

Size: ~140k tokens.
Covers: project_table, blank_plan, plan
Read first: spec Section 3.1 Appliance Machines, Section 4.1 Metallurgy & Crafting Tools, Section 6.1 Blocks (121 entries)
Java: BlockAppliance#getBlockTextureFromSideAndMetadata, TileAdvBench#updateEntity, ContainerAdvBench#canInteractWith, ContainerAdvBench#transferStackInSlot, SlotCraftRefill#onCrafting, InventorySubCraft#getStackInSlot, ItemPlan#writePlan, GuiAdvBench#drawGuiContainerBackgroundLayer
Files: mods/redpower_2/src/machines/project_table.rs, planning/mods/redpower_2.contract/project_table.json

## Beads import

- Beads id: `rk-5v91`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:31:20Z
- Close reason (Beads): Project table appliance and crafting plan encoding implemented with 7 unit tests passing.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:31:18Z · Jonathan Stoff] [domain-builder-pro] done: Project table appliance and crafting plan encoding implemented with 7 unit tests passing.
Files: mods/redpower_2/src/machine/project_table.rs
Plan satisfy from storage, auto-refill, rotation from yaw, and 28-slot layout all verified.
