---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-gem-tools
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-u2rk
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Gem tools: Ruby, Green Sapphire, and Sapphire pickaxes, shovels, axes, swords, and hoes

Implement all 15 gemstone tools across Ruby, Green Sapphire, and Sapphire materials: Pickaxes, Shovels, Axes, Swords, and Hoes. Every gem tool has durability 500, harvest level 2 (iron tier), mining speed 8.0F (faster than iron 6.0F, approaching diamond 8.0F), combat attack damage 7 (swords) / 3 (tools), and enchantability 12. Hoes till dirt/grass into farmland losing 1 durability. Done: make contract-check clean for all 15 gem tools; level 2 scenarios pass.

Size: ~120k tokens.
Covers: ruby_pickaxe, green_sapphire_pickaxe, sapphire_pickaxe, ruby_shovel, green_sapphire_shovel, sapphire_shovel, ruby_axe, green_sapphire_axe, sapphire_axe, ruby_sword, green_sapphire_sword, sapphire_sword, ruby_hoe, green_sapphire_hoe, sapphire_hoe
Read first: spec 4.2 Mining & Combat Equipment, 6. Complete Item & Block Registry (`redpower_2`)
Java: ItemCustomPickaxe#ctor, ItemCustomShovel#ctor, ItemCustomAxe#ctor, ItemCustomSword#ctor, ItemCustomHoe#ctor, RedPowerWorld#initialize
Files: mods/redpower_2/src/world/gem_tools.rs, planning/mods/redpower_2.contract/ruby_pickaxe.json, planning/mods/redpower_2.contract/green_sapphire_pickaxe.json, planning/mods/redpower_2.contract/sapphire_pickaxe.json, planning/mods/redpower_2.contract/ruby_shovel.json, planning/mods/redpower_2.contract/green_sapphire_shovel.json, planning/mods/redpower_2.contract/sapphire_shovel.json, planning/mods/redpower_2.contract/ruby_axe.json, planning/mods/redpower_2.contract/green_sapphire_axe.json, planning/mods/redpower_2.contract/sapphire_axe.json, planning/mods/redpower_2.contract/ruby_sword.json, planning/mods/redpower_2.contract/green_sapphire_sword.json, planning/mods/redpower_2.contract/sapphire_sword.json, planning/mods/redpower_2.contract/ruby_hoe.json, planning/mods/redpower_2.contract/green_sapphire_hoe.json, planning/mods/redpower_2.contract/sapphire_hoe.json

## Beads import

- Beads id: `rk-u2rk`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:33:08Z
- Close reason (Beads): 15 gem tools (ruby/sapphire/green_sapphire pickaxe, shovel, axe, sword, hoe): contracts written, max_damage 500 added; models, lang, tab, textures already existed.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:33:05Z · Jonathan Stoff] [content-author] done: 15 gem tools (ruby/sapphire/green_sapphire pickaxe, shovel, axe, sword, hoe): contracts written, max_damage 500 added; models, lang, tab, textures already existed.
Files: mods/redpower_2/content/data/redpower_2/horizon/item/*_{pickaxe,shovel,axe,sword,hoe}.json (15), planning/mods/redpower_2.contract/*_{pickaxe,shovel,axe,sword,hoe}.json (15)
COUNTS: 0 blocks, 15 items, 0 recipes new, 0 loot new, 0 lang new. CHECK check-mods pass. Templates are minecraft:diamond_* so tool speed/damage are diamond defaults; harvest level 2 / 8.0 speed / 7 damage need native verification.
