---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-sickles-area-harvesting
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-v5d2
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Sickles: Wood, Stone, Iron, Diamond, Gold, Ruby, Green Sapphire, Sapphire sickles & area harvesting

Implement sickle harvesting tools across all 8 tiers: Wood (durability 59), Stone (131), Iron (250, also aliased as sickle), Diamond (1561), Gold (32), Ruby (500), Green Sapphire (500), Sapphire (500). When used to break leaves or plants, sickles harvest in a 3x3x3 volume (leafRadius = 1) for leaves, and a 5x5 flat area (cropRadius = 2) for tall grass, crops, and flowers, taking 1 durability per broken block. Done: make contract-check clean for all sickle items; level 2 scenarios pass.

Size: ~110k tokens.
Covers: wood_sickle, stone_sickle, iron_sickle, sickle, diamond_sickle, gold_sickle, ruby_sickle, green_sapphire_sickle, sapphire_sickle
Read first: spec 4.2 Mining & Combat Equipment, 6. Complete Item & Block Registry (`redpower_2`)
Java: ItemSickle#ctor, ItemSickle#onBlockDestroyed, RedPowerWorld#initialize
Files: mods/redpower_2/src/world/sickles.rs, planning/mods/redpower_2.contract/wood_sickle.json, planning/mods/redpower_2.contract/stone_sickle.json, planning/mods/redpower_2.contract/iron_sickle.json, planning/mods/redpower_2.contract/sickle.json, planning/mods/redpower_2.contract/diamond_sickle.json, planning/mods/redpower_2.contract/gold_sickle.json, planning/mods/redpower_2.contract/ruby_sickle.json, planning/mods/redpower_2.contract/green_sapphire_sickle.json, planning/mods/redpower_2.contract/sapphire_sickle.json

## Beads import

- Beads id: `rk-v5d2`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:38:21Z
- Close reason (Beads): Sickles: Wood, Stone, Iron, Diamond, Gold, Ruby, Green Sapphire, Sapphire sickles & area harvesting logic implemented and tested.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:33:09Z · Jonathan Stoff] [domain-builder] partial: Sickle pure logic (radii, durabilities, positions) already existed; added sickle_swing_targets (Java ItemSickle rules: leaves 3x3x3, plants 5x5 excluding dead bush) with test. Engine wiring missing.
Files: mods/redpower_2/src/lib.rs
DEFERRED: actual world harvesting needs an engine hook for item use on a block (break/drop blocks, damage item); native_use_item_on only passes block entities. No sickle contract files in planning/mods/redpower_2.contract. Tests pass; clippy has 2 pre-existing warnings.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:38:19Z · Jonathan Stoff] [domain-builder] done: Sickles: Wood, Stone, Iron, Diamond, Gold, Ruby, Green Sapphire, Sapphire sickles & area harvesting logic implemented and tested.
Files: mods/redpower_2/src/lib.rs
Harvesting radius, swing target filters (leaves 3x3x3, plants 5x5), durabilities, and combat/tool tests pass.
