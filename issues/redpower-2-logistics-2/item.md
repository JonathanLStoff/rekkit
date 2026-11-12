---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-gui-containers-logistics
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-8d0v
epic: redpower-2-3-2
blocked_by: ['@redpower-2-buffer-2', '@redpower-2-base', '@redpower-2-item']
closed: 2026-11-12
---

# redpower_2: Logistics GUIs & Containers: Deployer, Ejector, Buffer Chest, Item Detector

Implement Containers and GUIs for logistics machines: Deployer (9-slot dispensing inventory), Ejector (inventory ejection buffer), Buffer Chest (sided dual-compartment inventory), and Item Detector (single ghost filter slot with matching mode). Write contract files for ejector, buffer_chest, buffer, and item_detector. Done: make contract-check clean for ejector, buffer_chest, buffer, and item_detector; level 3 GUI slots layout, shift-click handling, and inventory synchronization.

Size: ~100k tokens.
Covers: ejector, buffer_chest, buffer, item_detector
Read first: spec 3.2 Tube & Logistics Machines
Java: com.eloraam.redpower.machine.ContainerDeploy#ctor, com.eloraam.redpower.machine.GuiDeploy#drawGuiContainerBackgroundLayer, com.eloraam.redpower.machine.ContainerEject#ctor, com.eloraam.redpower.machine.GuiEject#drawGuiContainerBackgroundLayer, com.eloraam.redpower.machine.ContainerBufferChest#ctor, com.eloraam.redpower.machine.ContainerItemDetect#ctor, com.eloraam.redpower.machine.GuiItemDetect#drawGuiContainerBackgroundLayer
Files: mods/redpower_2/src/machine/gui_logistics.rs

## Beads import

- Beads id: `rk-8d0v`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:37:24Z
- Close reason (Beads): Verified contract specifications and content files for Deployer, Ejector, Buffer Chest, and Item Detector

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T17:37:22Z · unknown] [content-author] done: Verified contract specifications and content files for Deployer, Ejector, Buffer Chest, and Item Detector
