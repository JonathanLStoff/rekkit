---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-assembler
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-gicv
epic: redpower-2-3-2
blocked_by: ['@redpower-2-deployer-2']
closed: 2026-11-12
---

# redpower_2: Assembler: 16-channel bundled cable deployer, 18-slot storage, GUI

Implements Assembler (BlockMachine meta 13): 16 template selector slots (0..15), 18 internal inventory slots (16..33), 16-channel Bundled Cable input mapping, and selective item deployment per active channel. ContainerAssemble and GuiAssemble with mode and skip slot toggles. Done: contract-check clean for assembler; level 1, 3, 4, 5, 6 scenarios pass.

Size: ~160k tokens.
Covers: assembler
Read first: spec 3.2 Tube & Logistics Machines
Java: TileAssemble#updateCurrentStrength, TileAssemble#enableTowardsActive, TileAssemble#getMatchingStack, TileAssemble#enableTowards, ContainerAssemble#<init>, GuiAssemble#<init>
Files: mods/redpower_2/src/machine/assembler.rs, planning/mods/redpower_2.contract/assembler.json

## Beads import

- Beads id: `rk-gicv`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:11:42Z
- Close reason (Beads): Assembler: 16-channel bundled cable deployer, 18-slot storage, GUI verified with tests.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:11:40Z · Jonathan Stoff] [domain-builder-pro] done: Assembler: 16-channel bundled cable deployer, 18-slot storage, GUI verified with tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/src/machine/base.rs
Assembler container, 18-slot storage, and native block entity container tested and passing.
