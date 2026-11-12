---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-athame-enchantments
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-0h70
epic: redpower-2-3-2
blocked_by: ['@redpower-2-athame', '@redpower-2-equipment']
closed: 2026-11-12
---

# redpower_2: Combat: Athame Ender-slaying dagger, Disjunction and Vorpal decapitation enchantments

Implement Athame silver ritual dagger (durability 100, 1 base damage to normal mobs, 25 massive damage to Endermen and Ender Dragon), Disjunction enchantment (+6 dmg/level vs Ender mobs), and Vorpal weapon enchantment (2*lvl^2 % chance of +100 critical damage, overkill <= -20 HP decapitates victim heads). Done: contract-check clean for athame; Level 2 combat and damage scenarios passing.

Size: ~110k tokens.
Covers: athame
Read first: spec 4.2 Mining & Combat Equipment, 6.2 Items (99 entries)
Java: ItemAthame#a, EnchantmentDisjunction#a, EnchantmentVorpal#a, WorldEvents#onDeath
Files: mods/redpower_2/src/items/athame.rs, mods/redpower_2/src/items/enchantments.rs, planning/mods/redpower_2.contract/athame.json

## Beads import

- Beads id: `rk-0h70`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-08T03:05:29Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T03:05:13Z · unknown] [domain-builder] complete: Implemented Athame silver ritual dagger (1 damage normal / 25 damage vs Ender mobs), Disjunction (+6 dmg/level vs Ender mobs), and Vorpal enchantment (critical bonus damage and decapitation on <= -20 HP overkill).
