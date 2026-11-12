---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-gui-containers-routing
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-qtvc
epic: redpower-2-3-2
blocked_by: ['@redpower-2-sorting-3', '@redpower-2-retriever', '@redpower-2-base', '@redpower-2-filter-3']
closed: 2026-11-12
---

# redpower_2: Routing GUIs & Containers: Filter, Sorter, Retriever

Implement Containers and GUIs for pneumatic routing machines: Filter (9 filter slots with tube color assignment), Sorter (8 color columns, sorting modes: autosort, random, round-robin), and Retriever (request filter template slots with routing color tag). Write contract files for filter, sorting_machine, and retriever. Done: make contract-check clean for filter, sorting_machine, and retriever; level 3 GUI slots layout, mode selection buttons, and color tag selection.

Size: ~90k tokens.
Covers: filter, sorting_machine, retriever
Read first: spec 3.2 Tube & Logistics Machines
Java: com.eloraam.redpower.machine.ContainerFilter#ctor, com.eloraam.redpower.machine.GuiFilter#drawGuiContainerBackgroundLayer, com.eloraam.redpower.machine.ContainerSorter#ctor, com.eloraam.redpower.machine.GuiSorter#drawGuiContainerBackgroundLayer, com.eloraam.redpower.machine.ContainerRetriever#ctor, com.eloraam.redpower.machine.GuiRetriever#drawGuiContainerBackgroundLayer
Files: mods/redpower_2/src/machine/gui_routing.rs

## Beads import

- Beads id: `rk-qtvc`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T18:02:45Z
- Close reason (Beads): Implemented Routing GUIs & Containers (Filter, Sorter, Retriever) in gui_routing.rs, verified contract checks and unit tests

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-07T18:02:40Z · unknown] [content-author] done: Implemented Routing GUIs & Containers (Filter, Sorter, Retriever) in gui_routing.rs, verified contract checks and unit tests
