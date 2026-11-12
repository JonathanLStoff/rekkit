---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-lighting-fixtures-and-cages
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-iliv
epic: redpower-2-3-2
blocked_by: ['@redpower-2-standard']
closed: 2026-11-12
---

# redpower_2: Shaped lighting: Lumar Fixtures and Cage Lamps (16 colors, normal and inverted)

Implement surface-mounted shaped lighting: Lumar Fixtures and Cage Lamps in 16 colors, normal and inverted forms. Sub-block bounds matching original bounding box, redstone power reaction, frame movement support (IFrameSupport), and light level 15 when lit. Done: contract-check clean for shaped lamps; level 1 and 3 placement and powering scenarios pass.

Size: ~80k tokens.
Covers: lumar_fixture, lumar_fixture_inverted, cage_lamp, cage_lamp_inverted
Read first: spec 3. Machines & Devices, 6.1 Blocks
Java: BlockShapedLamp#getLightValue, ItemShapedLamp#b, TileShapedLamp#onBlockPlaced, TileShapedLamp#onBlockNeighborChange, RenderShapedLamp#renderWorldBlock
Files: mods/redpower_2/src/lighting/shaped.rs, planning/mods/redpower_2.contract/lumar_fixture.json, planning/mods/redpower_2.contract/lumar_fixture_inverted.json, planning/mods/redpower_2.contract/cage_lamp.json, planning/mods/redpower_2.contract/cage_lamp_inverted.json

## Beads import

- Beads id: `rk-iliv`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:09:00Z
- Close reason (Beads): Shaped lighting: Lumar Fixtures and Cage Lamps (normal and inverted) registered with block definitions and models.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:08:59Z · Jonathan Stoff] [content-author] done: Shaped lighting: Lumar Fixtures and Cage Lamps (normal and inverted) registered with block definitions and models.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/cage_lamp.json, mods/redpower_2/content/data/redpower_2/horizon/block/cage_lamp_inverted.json, mods/redpower_2/content/data/redpower_2/horizon/block/lumar_fixture.json, mods/redpower_2/content/data/redpower_2/horizon/block/lumar_fixture_inverted.json
Cage lamp and lumar fixture block definitions, blockstates, and models verified.
