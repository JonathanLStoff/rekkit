---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-transposer
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-rwsf
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-4']
closed: 2026-11-12
---

# redpower_2: Transposer: ground entity vacuum and adjacent inventory extraction

Implement the Transposer: vacuums loose item entities in front or extracts 1 item stack from adjacent inventory on redstone pulse, injecting extracted items into attached tube. Done: make contract-check clean for transposer; L1 placement and L4 extraction/vacuuming scenarios pass.

Size: ~100k tokens.
Covers: transposer
Read first: spec 3.2 Tube & Logistics Machines
Java: TileTranspose#onBlockNeighborChange, TileTranspose#handleExtract, TileTranspose#doSuck, TileTranspose#suckEntities, TileTranspose#drainBuffer
Files: mods/redpower_2/src/machine/transposer.rs, planning/mods/redpower_2.contract/transposer.json

## Beads import

- Beads id: `rk-rwsf`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:34:39Z
- Close reason (Beads): Transposer: ground entity vacuum and adjacent inventory extraction verified with tests.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:34:38Z · Jonathan Stoff] [domain-builder] done: Transposer: ground entity vacuum and adjacent inventory extraction verified with tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/src/machine/base.rs
Transposer tile entity, extraction mechanics, container and native screen verified.
