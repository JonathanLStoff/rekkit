---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-alloy-metallurgy-core
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-16eb
epic: redpower-2-3-2
blocked_by: ['@redpower-2-metallurgy']
closed: 2026-11-12
---

# redpower_2: Alloy Metallurgy Recipe System & Remainder Handling

Implement CraftLib alloy recipe registry ('redpower_2:alloy_furnace'): 3x3 unordered ingredient count matching, partial consumption, damage-on-craft tool tracking, and recipes for Red Alloy Ingot, Blue Alloy Ingot, Brass Ingot, Silicon Boule, Red/Blue Doped Wafers, and metal recycling. Done: unit tests for alloy recipe matching, ingredient subtraction, and craft remainder handling pass.

Size: ~90k tokens.
Covers: alloy_metallurgy
Read first: spec 3.1 Appliance Machines, 7.1 Custom Machine Recipes
Java: CraftLib#addAlloyResult, CraftLib#getAlloyResult, CraftLib#addDamageOnCraft, RedPowerBase#initAlloys
Files: mods/redpower_2/src/base/alloys.rs

## Beads import

- Beads id: `rk-16eb`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:38:12Z
- Close reason (Beads): CraftLib: container remainder now only for concrete-item ingredients (ore ingredients empty the slot, as in redpowercore CraftLib.getAlloyResult); added register_default_alloy_recipes (spec 7.1 alloys

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:38:09Z · Jonathan Stoff] [domain-builder] done: CraftLib: container remainder now only for concrete-item ingredients (ore ingredients empty the slot, as in redpowercore CraftLib.getAlloyResult); added register_default_alloy_recipes (spec 7.1 alloys, silicon/doped wafers, iron/gold recycling, wire recycling); tests added.
Files: mods/redpower_2/src/core/foundation.rs
foundation tests 10 pass; clippy has 3 pre-existing warnings outside this change. register_default_alloy_recipes is not yet called by any machine/lib.rs (alloy furnace owner must call it). Item ids use redpower_2:<name>_ingot style per spec 7.1 (existing unit tests use older ingot_x names, unaffected). Charcoal/coal registered as two recipes; no ore dictionary needed.
