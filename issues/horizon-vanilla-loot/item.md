---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: open
priority: normal
labels:
- horizon
- mod:example_mod
beads_id: rk-yls
---

# horizon: vanilla loot-pool `conditions` drop nothing

At Horizon 38a97ea6, a pool with vanilla's "conditions": [{"condition": "minecraft:survives_explosion"}] yields no drops when the block is broken in survival; Horizon's own form ("condition": {"type": ...}) works. Found by the live harness (stage 7), 2026-10-31. Proposed fix: Horizon's loot loader accepts vanilla's conditions array. Workaround: block loot tables in Horizon's form (mods/example_mod/.../ender_chest.json). Not yet reported upstream.

## Beads import

- Beads id: `rk-yls`
- Beads status: `open`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T17:41:17Z
