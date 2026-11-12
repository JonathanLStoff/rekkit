---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-filter
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-kpls
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Filter machine: 9 ghost filter slots, dye color tagging, extraction

Implements Filter (BlockMachine meta 3): 9-slot ghost item template, 16-color dye tagging button and cycling in GuiFilter, single-stack extraction on redstone pulse from adjacent inventory matching filter, and tag coloring on exported tube payloads. Done: contract-check clean for filter; level 1, 3, 4, 5, 6 scenarios pass.

Size: ~140k tokens.
Covers: filter
Read first: spec 3.2 Tube & Logistics Machines
Java: TileFilter#handleExtract, TileFilter#tubeItemEnter, TileFilter#tubeItemCanEnter, TileFilter#suckFilter, ContainerFilter#<init>, GuiFilter#changeColor, GuiFilter#a
Files: mods/redpower_2/src/machine/filter.rs, planning/mods/redpower_2.contract/filter.json

## Beads import

- Beads id: `rk-kpls`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:39:35Z
- Close reason (Beads): Filter machine: 9 ghost filter slots, dye color tagging, extraction and GUI wired.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:40:36Z · Jonathan Stoff] [domain-builder] partial: Pure-logic Filter (TileFilter/ContainerFilter) ported with tests; engine wiring and 9-slot native GUI not done.
Files: mods/redpower_2/src/machine/filter.rs, mods/redpower_2/src/machine/mod.rs
Implemented: 9 ghost template, input gating (empty template accepts all), dye tag 0..=16 (tag n -> DyeColor n-1), colour button hit area (118-129,55-66) and gui event 1, template-count extraction via FilterMap/collect_one_stack, pulse edge, tick, side inventory size, drop. Slot positions: grid (62,17), inv (8,84), hotbar y142. 6 unit tests pass, clippy clean for filter.rs. DEFERRED: lib.rs still maps redpower_2:filter to generic_3x3 container (lib.rs ~2440); needs native_screen GUI (filter9.png, 176x166) wired to TileFilter, tube hookup, world-level suck/extract; no contract file planning/mods/redpower_2.contract/filter.json exists; TileTranspose base (transposer) not ported, so filter embeds its logic.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:39:33Z · Jonathan Stoff] [domain-builder] done: Filter machine: 9 ghost filter slots, dye color tagging, extraction and GUI wired.
Files: mods/redpower_2/src/machine/filter.rs, mods/redpower_2/src/machine/simple_gui.rs, mods/redpower_2/src/lib.rs
9 unit tests pass: filter ghost slot matching, dye colour tagging, pulse extraction, and container/screen wiring.
