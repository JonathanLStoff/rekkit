---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-gem-tools-sickles
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-wipg
epic: redpower-2-3-2
blocked_by: ['@redpower-2-base-3', '@redpower-2-world', '@redpower-2-gem', '@redpower-2-sickles']
closed: 2026-11-12
---

# redpower_2: Equipment: Ruby/Sapphire gem tools, handsaws and 8 sickle material variants

Implement gem tools (Pickaxes, Shovels, Axes, Swords, Hoes across Ruby, Green Sapphire, and Sapphire materials: Durability 500, Harvest level 2, Mining speed 8.0, Damage 3), gem handsaws (Sharpness 1, Durability 640), and Sickles across 8 material tiers (Wood, Stone, Iron, Diamond, Gold, Ruby, Green Sapphire, Sapphire), with area-of-effect harvesting of 3x3x3 leaves and 5x5 tall grass/crops. Done: contract-check clean for all 27 tool items; Level 2 item use scenarios passing.

Size: ~150k tokens.
Covers: ruby_pickaxe, green_sapphire_pickaxe, sapphire_pickaxe, ruby_shovel, green_sapphire_shovel, sapphire_shovel, ruby_axe, green_sapphire_axe, sapphire_axe, ruby_sword, green_sapphire_sword, sapphire_sword, ruby_hoe, green_sapphire_hoe, sapphire_hoe, ruby_handsaw, green_sapphire_handsaw, sapphire_handsaw, wood_sickle, stone_sickle, iron_sickle, sickle, diamond_sickle, gold_sickle, ruby_sickle, green_sapphire_sickle, sapphire_sickle
Read first: spec 4.1 Metallurgy & Crafting Tools, 4.2 Mining & Combat Equipment, 6.2 Items (99 entries)
Java: ItemCustomPickaxe, ItemCustomShovel, ItemCustomAxe, ItemCustomSword, ItemCustomHoe, ItemSickle#a
Files: mods/redpower_2/src/items/tools.rs, mods/redpower_2/src/items/sickle.rs, planning/mods/redpower_2.contract/ruby_pickaxe.json, planning/mods/redpower_2.contract/green_sapphire_pickaxe.json, planning/mods/redpower_2.contract/sapphire_pickaxe.json, planning/mods/redpower_2.contract/ruby_shovel.json, planning/mods/redpower_2.contract/green_sapphire_shovel.json, planning/mods/redpower_2.contract/sapphire_shovel.json, planning/mods/redpower_2.contract/ruby_axe.json, planning/mods/redpower_2.contract/green_sapphire_axe.json, planning/mods/redpower_2.contract/sapphire_axe.json, planning/mods/redpower_2.contract/ruby_sword.json, planning/mods/redpower_2.contract/green_sapphire_sword.json, planning/mods/redpower_2.contract/sapphire_sword.json, planning/mods/redpower_2.contract/ruby_hoe.json, planning/mods/redpower_2.contract/green_sapphire_hoe.json, planning/mods/redpower_2.contract/sapphire_hoe.json, planning/mods/redpower_2.contract/ruby_handsaw.json, planning/mods/redpower_2.contract/green_sapphire_handsaw.json, planning/mods/redpower_2.contract/sapphire_handsaw.json, planning/mods/redpower_2.contract/wood_sickle.json, planning/mods/redpower_2.contract/stone_sickle.json, planning/mods/redpower_2.contract/iron_sickle.json, planning/mods/redpower_2.contract/sickle.json, planning/mods/redpower_2.contract/diamond_sickle.json, planning/mods/redpower_2.contract/gold_sickle.json, planning/mods/redpower_2.contract/ruby_sickle.json, planning/mods/redpower_2.contract/green_sapphire_sickle.json, planning/mods/redpower_2.contract/sapphire_sickle.json

## Beads import

- Beads id: `rk-wipg`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T03:03:04Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-08T03:02:52Z · unknown] [content-author] complete: Implemented and verified all 27 gem tools (Ruby, Green Sapphire, Sapphire pickaxes/shovels/axes/swords/hoes), handsaws (durability 640), and 8 sickle material variants (3x3x3 leaves, 5x5 crops AoE harvesting).
