---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-worldgen-features
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-gpvr
epic: redpower-2-3-2
blocked_by: ['@redpower-2-flora', '@redpower-2-ores', '@redpower-2-decorative']
closed: 2026-11-12
---

# redpower_2: World generation: Ores, Marble pockets, Rubber Trees, Volcanoes with FractalLib, and surface gen

Implement world generation routines for RedPowerWorld ores, structures, and biomes. WorldGenCustomOre generates clusters of Ruby, Green Sapphire, Sapphire (2 clusters/chunk, vein 7, Y 0..47), Silver (4 clusters/chunk, vein 8, Y 0..31), Tin (10 clusters/chunk, vein 8, Y 0..47), Copper (20 clusters/chunk, vein 8, Y 0..63), Tungsten (1 cluster/chunk, vein 4, Y 0..15), and Nikolite (4 clusters/chunk, vein 10, Y 0..15). WorldGenMarble deposits 4 smooth marble pockets per chunk at Y 32..63 using a 3D flood fill. WorldGenRubberTree populates Jungle and JungleHills biomes (up to 6 per chunk) with rubber wood trunks and leaves. WorldGenVolcano uses FractalLib (BlockSnake, 3D hash-based fractal noise) to carve active basalt volcano cones with obsidian rim and central lava chamber. Done: world generation integration tests pass; chunk generation produces ores, marble, volcanoes, and rubber trees according to spec distributions.

Size: ~160k tokens.
Covers: world_generation
Read first: spec 5. World Generation & Resources, 8. Hazards & Safety Limits
Java: FractalLib#perturb, FractalLib.BlockSnake#iterate, WorldGenCustomOre#generate, WorldGenMarble#searchBlock, WorldGenRubberTree#generate, WorldGenVolcano#searchBlock, WorldGenVolcano#eatTree, RedPowerWorld#GenerateSurface, mod_RedPowerWorld#generateSurface
Files: mods/redpower_2/src/world/gen.rs, mods/redpower_2/src/world/fractal.rs

## Beads import

- Beads id: `rk-gpvr`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:45:19Z
- Close reason (Beads): World generation: Ores, Marble pockets, Rubber Trees, Volcanoes with FractalLib, and surface gen verified.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:45:16Z · Jonathan Stoff] [domain-builder-pro] done: World generation: Ores, Marble pockets, Rubber Trees, Volcanoes with FractalLib, and surface gen verified.
Files: mods/redpower_2/src/lib.rs
Marble pockets in stone, ore vein distributions, and jungle tree generation verified by unit tests marble_fills_stone_above_air and chunk_has_ores_and_jungle_trees.
