---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-worldgen
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-aont
epic: redpower-2-3-2
blocked_by: ['@redpower-2-world-3', '@redpower-2-world-5']
closed: 2026-11-12
---

# redpower_2: World Generation: Ore clusters, Marble veins, Basalt volcanoes & Rubber trees

Implement world generation features: Custom ore vein distribution (Ruby, Sapphire, Silver, Tin, Copper, Tungsten, Nikolite Y-ranges and cluster sizes), Marble pockets in Y 32..63, Basalt volcano cones with internal magma chambers and lava tubes, and Jungle rubber tree distribution. Includes WorldEvents hook. Done: world generation unit and integration tests passing; Level 4 scenario verifying terrain feature generation.

Size: ~140k tokens.
Covers: world_generation
Read first: spec 5. World Generation & Resources
Java: WorldGenHandler#generate, WorldGenCustomOre#a, WorldGenMarble#a, WorldGenVolcano#a, WorldGenRubberTree#a, WorldEvents#onBonemeal
Files: mods/redpower_2/src/world/gen.rs, mods/redpower_2/src/world/events.rs

## Beads import

- Beads id: `rk-aont`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T03:01:22Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T03:01:09Z · unknown] [domain-builder] complete: Implemented world generation features (8 custom ore clusters, Marble strata, Basalt volcanoes, Indigo patches, Jungle Rubber Trees) and WorldEvents hooks (bonemeal, Athame drops, seed bag pickup).
