---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-logistics-advanced
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-hk87
epic: redpower-2-3-2
blocked_by: ['@redpower-2-assembler', '@redpower-2-machine-2']
closed: 2026-11-12
---

# redpower_2: Advanced logistics: Bundled-cable Assembler and Blutricity Manager

Implement Assembler (34 slots: 16 template selectors connected to 16 bundled cable channels, 18 internal inventory slots; deploys matching item on corresponding channel signal) and Manager (blutricity-powered inventory regulator: continuously balances and syncs connected inventory to match 24-slot template, requesting missing items across tube network and ejecting excess). Done: make contract-check MOD=redpower_2 clean for assembler and manager; level 1, 3, 4, 5 scenarios pass.

Size: ~180k tokens.
Covers: assembler, manager
Read first: spec 3.2 Tube & Logistics Machines
Java: TileAssemble#enableTowardsActive, TileAssemble#updateCurrentStrength, TileManager#scanInventory, TileManager#doRequest, TileManager#handleTubeItem, TileManager#requestTubeItem
Files: mods/redpower_2/src/machine/logistics_advanced.rs, planning/mods/redpower_2.contract/assembler.json, planning/mods/redpower_2.contract/manager.json

## Beads import

- Beads id: `rk-hk87`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:34:22Z
- Close reason (Beads): Advanced logistics: Bundled-cable Assembler and Blutricity Manager verified with tests.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T05:34:21Z · Jonathan Stoff] [domain-builder-pro] done: Advanced logistics: Bundled-cable Assembler and Blutricity Manager verified with tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/src/machine/base.rs
Assembler and Manager tile entities, native block entity containers, and proxy mappings tested.
