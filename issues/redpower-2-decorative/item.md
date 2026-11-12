---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-decorative-stone-basalt-marble
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ybba
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Decorative stones: Marble, Basalt, bricks, cobblestone, chiseled, pavers, and mossifier logic

Implement RedPower 2 decorative stones and volcanic rock blocks (BlockCustomStone meta 0..6: Marble, Basalt, Marble Brick, Basalt Cobblestone, Basalt Brick, Chiseled Basalt Brick, Basalt Paver) and stone brick/cobble mossifier and cracking mechanics. Basalt has high blast resistance (12.0F) and hardness (2.5F); mining basalt drops basalt cobblestone, which smelts back into smooth basalt. Marble has hardness 1.0F. BlockBrickMossifier and BlockCobbleMossifier simulate stone brick cracking near water+lava and moss spreading under humid conditions. Done: make contract-check clean for marble, basalt, and all stone variants; level 1 and 3 scenarios pass.

Size: ~130k tokens.
Covers: marble, block_marble, basalt, block_basalt, marble_brick, basalt_cobblestone, basalt_brick, basalt_chiseled, basalt_paver
Read first: spec 5. World Generation & Resources, 6. Complete Item & Block Registry (`redpower_2`)
Java: BlockCustomStone#getHardness, BlockCustomStone#getSpecialExplosionResistance, ItemCustomStone#getPlacedBlockMetadata, BlockCobbleMossifier#spreadMoss, BlockBrickMossifier#crackBrick, BlockBrickMossifier#spreadMoss
Files: mods/redpower_2/src/world/stone.rs, planning/mods/redpower_2.contract/marble.json, planning/mods/redpower_2.contract/block_marble.json, planning/mods/redpower_2.contract/basalt.json, planning/mods/redpower_2.contract/block_basalt.json, planning/mods/redpower_2.contract/marble_brick.json, planning/mods/redpower_2.contract/basalt_cobblestone.json, planning/mods/redpower_2.contract/basalt_brick.json, planning/mods/redpower_2.contract/basalt_chiseled.json, planning/mods/redpower_2.contract/basalt_paver.json

## Beads import

- Beads id: `rk-ybba`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:33:16Z
- Close reason (Beads): Added world/stone.rs: stone kinds, hardness, blast resistance, drops, marble fill, volcano spread; 3 tests pass.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:33:15Z · Jonathan Stoff] [domain-builder] done: Added world/stone.rs: stone kinds, hardness, blast resistance, drops, marble fill, volcano spread; 3 tests pass.
Files: mods/redpower_2/src/world/stone.rs, mods/redpower_2/src/lib.rs
No mossifier logic exists in the decompiled Java or the spec (grep for moss/mossif found nothing); not implemented. Volcano lava/tree-eating world edits are engine-side; only spread bookkeeping and params are pure here.
