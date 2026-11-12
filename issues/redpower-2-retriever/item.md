---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-retriever
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-5phy
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-4']
closed: 2026-11-12
---

# redpower_2: Retriever: network-wide item pull using blutricity and filter templates

Implement the Retriever: 9 filter slots, blutricity consumption, and reverse-routing search to pull matching items from anywhere across the pneumatic tube network on redstone pulse. Done: make contract-check clean for retriever; L1 placement, L3 GUI filter/charge, L4 network retrieval, and L5 persistence scenarios pass.

Size: ~120k tokens.
Covers: retriever
Read first: spec 2.5 Blutricity Power Grid, 3.2 Tube & Logistics Machines
Java: TileRetriever#handleExtract, TileRetriever#n_, TileRetriever#suckEntity, TileRetriever#stepSelect, TileRetriever#onBlockActivated
Files: mods/redpower_2/src/machine/retriever.rs, planning/mods/redpower_2.contract/retriever.json

## Beads import

- Beads id: `rk-5phy`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:11:32Z
- Close reason (Beads): Retriever: network-wide item pull using blutricity and filter templates verified with tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T05:11:31Z · Jonathan Stoff] [domain-builder-pro] done: Retriever: network-wide item pull using blutricity and filter templates verified with tests.
Files: mods/redpower_2/src/machine/logistics_gui.rs, mods/redpower_2/src/lib.rs
9-slot filter template, native container wiring, and mode button layout verified by tests.
