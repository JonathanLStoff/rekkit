---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-gui-containers-management
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-yuzu
epic: redpower-2-3-2
blocked_by: ['@redpower-2-regulator', '@redpower-2-base', '@redpower-2-assembler']
closed: 2026-11-12
---

# redpower_2: Management GUIs & Containers: Regulator, Manager, Assembler

Implement Containers and GUIs for inventory regulation and auto-assembly: Regulator (dual-side target and input buffer slots), Manager (stock maintenance grid with color channel config), and Assembler (3x3 crafting matrix, plan slot, internal component storage). Write contract files for regulator, manager, and assembler. Done: make contract-check clean for regulator, manager, and assembler; level 3 GUI slots layout, plan item interaction, and regulation threshold syncing.

Size: ~90k tokens.
Covers: regulator, manager, assembler
Read first: spec 3.2 Tube & Logistics Machines
Java: com.eloraam.redpower.machine.ContainerRegulator#ctor, com.eloraam.redpower.machine.GuiRegulator#drawGuiContainerBackgroundLayer, com.eloraam.redpower.machine.ContainerManager#ctor, com.eloraam.redpower.machine.GuiManager#drawGuiContainerBackgroundLayer, com.eloraam.redpower.machine.ContainerAssemble#ctor, com.eloraam.redpower.machine.GuiAssemble#drawGuiContainerBackgroundLayer
Files: mods/redpower_2/src/machine/gui_management.rs

## Beads import

- Beads id: `rk-yuzu`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:26:22Z
- Close reason (Beads): Implemented Regulator, Manager, and Assembler Container and GUI layouts with exact slot configurations and sync settings in mods/redpower_2/src/machine/gui_management.rs.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-08T02:26:20Z · unknown] [content-author] done: Implemented Regulator, Manager, and Assembler Container and GUI layouts with exact slot configurations and sync settings in mods/redpower_2/src/machine/gui_management.rs.
