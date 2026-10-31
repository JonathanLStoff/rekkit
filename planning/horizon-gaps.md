# Horizon engine gaps

Features Rekkit mods need that Horizon does not provide yet. Appended by the hook-engineer (or any agent that finds one); triaged by a human (upstream a Horizon PR, work around it, or defer). Format is in `.agents/skills/horizon-hooks/SKILL.md`.

## example_mod: vanilla loot-pool `conditions` drop nothing
- Needed for: every block loot table (`content/data/<id>/loot_table/blocks/*.json`)
- Missing: at Horizon `38a97ea6`, a pool with vanilla's `"conditions": [{ "condition": "minecraft:survives_explosion" }]` yields no drops when the block is broken in survival. The same pool without `conditions`, or in Horizon's own template form (`"condition": { "type": "minecraft:survives_explosion" }`, from `cargo horizon-mod new`), drops the item. Found by the live E2E harness (stage 7), 2026-10-31.
- Proposed fix: Horizon's loot loader accepts vanilla's `conditions` array on pools and entries.
- Workaround: write block loot tables in Horizon's template form (see `mods/example_mod/content/data/example_mod/loot_table/blocks/ender_chest.json`).
- Status: open (not yet reported upstream)
