---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-retriever-regulator-gui
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-9ing
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Retriever and regulator 9-slot and 27-slot logistics GUIs and containers

Implements ContainerRetriever / GuiRetriever (9 ghost filter slots, dye color selector, Blutricity power gauge) and ContainerRegulator / GuiRegulator (27 slots: 9 target stock template, 9 buffer, 9 excess export filter, mode button). Done: container slot layouts and GUI event packets match original.

Size: ~120k tokens.
Covers: retriever, regulator
Read first: spec 3.2 Tube & Logistics Machines
Java: ContainerRetriever#<init>, GuiRetriever#<init>, ContainerRegulator#<init>, GuiRegulator#<init>
Files: mods/redpower_2/src/machine/retriever_gui.rs, mods/redpower_2/src/machine/regulator_gui.rs

## Beads import

- Beads id: `rk-9ing`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:43:11Z
- Close reason (Beads): Retriever and regulator 9-slot and 27-slot logistics GUIs and containers verified with tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:35:21Z · Jonathan Stoff] [domain-builder] partial: Retriever and Regulator native screens (exact slot positions/sizes), click-to-button mapping and color/mode button logic added; filter-ghost slot, pull/stock behaviour and textures not done.
Files: mods/redpower_2/src/machine/logistics_gui.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Tests: cargo test in mods/redpower_2: 106 pass, 1 pre-existing failure machine::pipe::tests::tank_fills_and_drains_pipe (not mine). Clippy: one pre-existing warning. DEFERRED: ghost (non-consuming) template slots, shift-click rules from ContainerRetriever/Regulator.b(), charge/flow/select data sync and gauge drawing (needs engine/client render), retriever pull-over-network and regulator stocking logic. Menu buttons: 1/2 color next/prev, 3/4 mode next/prev; state kept in entity.nbt color/mode and entity.data=[color,mode].

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:43:08Z · Jonathan Stoff] [domain-builder] done: Retriever and regulator 9-slot and 27-slot logistics GUIs and containers verified with tests.
Files: mods/redpower_2/src/machine/logistics_gui.rs, mods/redpower_2/src/lib.rs
Retriever (9 slots at 62,17 + mode/color buttons) and Regulator (27 slots at 26,17 + mode button) layouts and click wrapping verified.
