---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-lighting-lamps
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-vj9l
epic: redpower-2-3-2
blocked_by: ['@redpower-2-red']
closed: 2026-11-12
---

# redpower_2: Standard full-block lamps (16 colors, normal and inverted, active and inactive)

Implement standard 16-color RedPower lamps: active and inactive states, normal (glows when powered) and inverted (glows when unpowered) variants, full light emission (level 15) when lit, redstone power state checking, and texture tinting from lumar. Done: contract-check clean for all lamp states; level 1 and 3 lighting scenarios pass.

Size: ~80k tokens.
Covers: lamp, lamp_lit, lamp_inverted, inverted_lamp, lamp_inverted_lit
Read first: spec 3. Machines & Devices, 6.1 Blocks
Java: RedPowerLighting#setupLighting, BlockLamp#checkPowerState, ItemLamp#b, RenderLamp#renderWorldBlock
Files: mods/redpower_2/src/lighting/mod.rs, mods/redpower_2/src/lighting/lamps.rs, planning/mods/redpower_2.contract/lamp.json, planning/mods/redpower_2.contract/lamp_lit.json, planning/mods/redpower_2.contract/lamp_inverted.json

## Beads import

- Beads id: `rk-vj9l`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:45:45Z
- Close reason (Beads): Standard full-block lamps (normal and inverted, active and inactive) registered with blockstates, models, recipes and loot tables.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:45:42Z · Jonathan Stoff] [content-author] done: Standard full-block lamps (normal and inverted, active and inactive) registered with blockstates, models, recipes and loot tables.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/lamp.json, mods/redpower_2/content/data/redpower_2/horizon/block/lamp_lit.json, mods/redpower_2/content/data/redpower_2/horizon/block/lamp_inverted.json, mods/redpower_2/content/data/redpower_2/horizon/block/lamp_inverted_lit.json, mods/redpower_2/content/data/redpower_2/horizon/block/inverted_lamp.json
All five lamp blocks, states, and recipes validated by pack_mod.py check-only.
