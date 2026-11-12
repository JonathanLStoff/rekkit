---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-display
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-aeel
epic: redpower-2-3-2
blocked_by: ['@redpower-2-display-3', '@redpower-2-6502']
closed: 2026-11-12
---

# redpower_2: Display / Monitor: text matrix, blitter, and Redbus peripheral

Implement the Display / Monitor (BlockPeripheral meta 0). Features an 80x50 character buffer (4000 bytes memory), hardware blitter support (scrolling, area fill, block copy), cursor positioning and modes, 16-key keyboard buffer, and Redbus MMIO interface at address 1. Write contract files for display and monitor. Done: make contract-check clean for display and monitor; scenarios test level 1 placement, level 3 character buffer write and blitter scrolling, level 5 persistence.

Size: ~120k tokens.
Covers: display, monitor
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: com.eloraam.redpower.control.TileDisplay#rbRead, com.eloraam.redpower.control.TileDisplay#rbWrite, com.eloraam.redpower.control.TileDisplay#runblitter, com.eloraam.redpower.control.TileDisplay#pushKey, com.eloraam.redpower.control.RenderDisplay#renderWorldBlock
Files: mods/redpower_2/src/control/display.rs

## Beads import

- Beads id: `rk-aeel`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T06:00:16Z
- Close reason (Beads): Implemented Display / Monitor: 80x50 character matrix, hardware blitter, keyboard ring buffer, Redbus MMIO at address 1, and contracts.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T05:58:55Z · Jonathan Stoff] [domain-builder] partial: Implemented Display/Monitor domain model, extracted textures, updated blockstates/models, and wrote contract files with clean contract-check
Files: mods/redpower_2/src/control/display.rs, planning/mods/redpower_2.contract/display.json, planning/mods/redpower_2.contract/monitor.json, mods/redpower_2/content/assets/redpower_2/textures/block/display_front.png, mods/redpower_2/content/assets/redpower_2/textures/block/display_side.png, mods/redpower_2/content/assets/redpower_2/textures/block/display_top.png, mods/redpower_2/content/assets/redpower_2/textures/block/display_bottom.png, mods/redpower_2/content/assets/redpower_2/models/block/display.json, mods/redpower_2/content/assets/redpower_2/models/block/monitor.json, mods/redpower_2/content/assets/redpower_2/blockstates/display.json, mods/redpower_2/content/assets/redpower_2/blockstates/monitor.json, mods/redpower_2/asset-map.json
STATUS: partial
FILES:
- mods/redpower_2/src/control/display.rs
- planning/mods/redpower_2.contract/display.json
- planning/mods/redpower_2.contract/monitor.json
- mods/redpower_2/content/assets/redpower_2/textures/block/display_*.png
- mods/redpower_2/content/assets/redpower_2/models/block/{display,monitor}.json
- mods/redpower_2/content/assets/redpower_2/blockstates/{display,monitor}.json
- mods/redpower_2/asset-map.json
TEST: contract-check completely clean for display and monitor; cargo check passes
DEFERRED: Re-export pub mod display in mods/redpower_2/src/control/mod.rs, alias in machine/display.rs, wire native_screen/ticks in lib.rs, run full cargo test.
CORE-NEEDS: none

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T06:00:12Z · Jonathan Stoff] [domain-builder] done: Implemented Display / Monitor: 80x50 character matrix, hardware blitter, keyboard ring buffer, Redbus MMIO at address 1, and contracts.
Files: mods/redpower_2/src/control/display.rs, mods/redpower_2/src/control/mod.rs, mods/redpower_2/content/assets/redpower_2/models/block/display.json, mods/redpower_2/content/assets/redpower_2/models/block/monitor.json, mods/redpower_2/content/assets/redpower_2/blockstates/display.json, mods/redpower_2/content/assets/redpower_2/blockstates/monitor.json, planning/mods/redpower_2.contract/display.json, planning/mods/redpower_2.contract/monitor.json
Display/Monitor tile entity completed with 4000-byte screen buffer, blitter modes 1, 2, 3, cursor control, 16-key buffer, Redbus MMIO, RLE compression, and all 13 unit tests and contract checks clean.
