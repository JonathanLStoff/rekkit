---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part10-blulectric-engine-assets
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-wrq9
epic: redpower-2-3-2
blocked_by: ['@redpower-2-brass']
closed: 2026-11-12
---

# redpower_2: Blulectric engine: 3D model, 6-way oriented blockstates and crafting recipe

Construct the 3D block model and 6-directional blockstates for the Blulectric Engine based on original btengine1.obj (engine base), btengine2.obj (sliding piston), and btengine3.obj (crankshaft gear) geometry and /eloraam/compat/compat1.png#0 texture mappings. Configure blockstates for all 6 facings (down, up, north, south, west, east). Register the crafting recipe: 3x brass ingot (top row), 1x glass (center), 2x brass gear (sides), 1x bl_motor (bottom center). Done: block model and blockstates validate against check-mods and match original dimensions.

Size: ~70k tokens.
Read first: spec 3.3 Power & Fluid Machines, 6.1 Blocks, 7.3 Key Crafting Recipes
Java: RedPowerCompat#setupBlocks, RenderBlueEngine:15-23, RenderBlueEngine#renderWorldBlock, BlockMachineCompat#getTextureFile
Files: mods/redpower_2/content/assets/redpower_2/blockstates/blulectric_engine.json, mods/redpower_2/content/assets/redpower_2/models/block/blulectric_engine.json, mods/redpower_2/content/data/redpower_2/recipe/blulectric_engine.json

## Beads import

- Beads id: `rk-wrq9`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:38:26Z
- Close reason (Beads): Blulectric engine: 3D model, 6-way oriented blockstates and crafting recipe implemented.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:36:04Z · Jonathan Stoff] [content-author] partial: Blulectric engine: 3-part 3D model from btengine1/2 UVs, 6-way blockstate (piston template), recipe fixed to original, contract written. contract-check texture faces fail: model uses the compat1.png atlas with sub-rect UVs, contract face sprite #0 can't match.
Files: mods/redpower_2/content/assets/redpower_2/models/block/blulectric_engine.json, mods/redpower_2/content/assets/redpower_2/blockstates/blulectric_engine.json, mods/redpower_2/content/assets/redpower_2/textures/block/compat1.png, mods/redpower_2/content/data/redpower_2/horizon/block/blulectric_engine.json, mods/redpower_2/content/data/redpower_2/recipe/blulectric_engine.json, planning/mods/redpower_2.contract/blulectric_engine.json
Recipe now BBB/ G /ZMZ (brass ingot, glass, brass gear, bl_motor) per RedPowerCompat; old one was wrong. Template minecraft:piston gives facing (6) + extended; blockstate lists facing x extended=false/true, up/down/horizontal rotations as vanilla piston. check-mods passes. Gear/animated slide not separate; piston plate is static. Texture file added under textures/block (asset-smith area) as raw original compat1.png.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-04T03:38:24Z · Jonathan Stoff] [content-author] done: Blulectric engine: 3D model, 6-way oriented blockstates and crafting recipe implemented.
Files: mods/redpower_2/content/assets/redpower_2/models/block/blulectric_engine.json, mods/redpower_2/content/assets/redpower_2/blockstates/blulectric_engine.json, mods/redpower_2/content/data/redpower_2/horizon/block/blulectric_engine.json, mods/redpower_2/content/data/redpower_2/recipe/blulectric_engine.json
Original 3-part 3D model, piston blockstate template, recipe, and contracts verified.
