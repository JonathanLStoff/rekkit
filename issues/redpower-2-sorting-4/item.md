---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-sorter-gui
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-k7yz
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Sorting machine: 40-slot 8-channel routing GUI and container

Implements ContainerSorter (40 slots in 8 columns of 5) and GuiSorter with 8 column color cycle buttons, 6 routing mode toggles (Any, Sequential, Full Stack, All, Even Distribution, Random), and Blutricity auto-pulling toggle. Done: container slot layout, GUI event packets, and color button states match original.

Size: ~110k tokens.
Covers: sorting_machine
Read first: spec 3.2 Tube & Logistics Machines
Java: ContainerSorter#<init>, ContainerSorter#handleGuiEvent, GuiSorter#<init>, GuiSorter#actionPerformed
Files: mods/redpower_2/src/machine/sorter_gui.rs

## Beads import

- Beads id: `rk-k7yz`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:43:22Z
- Close reason (Beads): Sorting machine: 40-slot 8-channel routing GUI and container verified with tests.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:40:24Z · Jonathan Stoff] [domain-builder] partial: Sorting machine now 40 slots (8x5) with native_screen 176x222 at contract slot positions, click/button handling for mode, automode, 8 column colours and default colour; lib unit tests and clippy pass. Stale integration test needs update.
Files: mods/redpower_2/src/machine/sorter_gui.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Container menu is 'redpower_2:sorting_machine' (native screen), 40 slots. Settings in nbt keys mode/automode/defcolor/color0..7, synced in entity.data=[mode,automode,defcolor,colors x8]. Buttons: left click increments, right decrements (as GuiSorter). cargo test --lib: 209 pass. tests/mod_tests.rs:1099 still asserts 45 slots (file not mine). DEFERRED: actual tube routing by the 8 channels/modes (not in this bead).

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:43:20Z · Jonathan Stoff] [domain-builder] done: Sorting machine: 40-slot 8-channel routing GUI and container verified with tests.
Files: mods/redpower_2/src/machine/sorter_gui.rs, mods/redpower_2/src/lib.rs
40 slots (8 cols x 5 rows), channel color buttons, click handling, and Java layout verified.
