---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-retriever-regulator
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-1irz
epic: redpower-2-3-2
blocked_by: ['@redpower-2-retriever', '@redpower-2-regulator', '@redpower-2-sorting-2', '@redpower-2-routing', '@redpower-2-management']
closed: 2026-11-12
---

# redpower_2: Logistics Machines: Retriever network-pulling and Regulator inventory stocking

Implement Retriever (searches connected pneumatic network on pulse to pull items matching its 9 filter slots, using Blutricity) and Regulator (compares adjacent inventory contents against 9-slot stocking template, routing missing items in and excess items out). Includes GUIs and container sync. Done: contract-check clean for retriever and regulator; Level 1, 3, 4, 5 scenarios passing.

Size: ~150k tokens.
Covers: retriever, regulator
Read first: spec 3.2 Tube & Logistics Machines, 6.1 Blocks (121 entries)
Java: TileRetriever#handleExtract, TileRetriever#stepSelect, TileRegulator#scanInput, TileRegulator#scanOutput, TileRegulator#handleTransfer
Files: mods/redpower_2/src/machines/retriever.rs, mods/redpower_2/src/machines/regulator.rs, planning/mods/redpower_2.contract/retriever.json, planning/mods/redpower_2.contract/regulator.json

## Beads import

- Beads id: `rk-1irz`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T02:33:25Z
- Close reason (Beads): Implemented Retriever network pulling and Regulator inventory stocking mechanics in mods/redpower_2/src/machines/{retriever,regulator}.rs with exact slot layouts, power consumption, and chunk reload p

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T02:33:23Z · unknown] [domain-builder-pro] done: Implemented Retriever network pulling and Regulator inventory stocking mechanics in mods/redpower_2/src/machines/{retriever,regulator}.rs with exact slot layouts, power consumption, and chunk reload persistence.
