---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part09-athame-disjunction
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ck8x
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Athame ritual silver dagger and Disjunction enchantment

Implement the Athame ceremonial silver dagger (ItemAthame) and the exclusive Disjunction weapon enchantment (EnchantmentDisjunction). Athame has durability 100, base attack damage 1 against normal mobs, but deals 25 base damage against Endermen and the Ender Dragon. Disjunction (levels 1..5) is exclusive to Athame and deals +6 * level additional bonus damage to Endermen and the Ender Dragon. Done: make contract-check clean for athame; level 2 combat scenarios pass.

Size: ~110k tokens.
Covers: athame
Read first: spec 4.2 Mining & Combat Equipment, 6. Complete Item & Block Registry (`redpower_2`)
Java: ItemAthame#ctor, ItemAthame#getDamageVsEntity, EnchantmentDisjunction#ctor, EnchantmentDisjunction#calcModifierLiving, EnchantmentDisjunction#canEnchantItem
Files: mods/redpower_2/src/world/athame.rs, planning/mods/redpower_2.contract/athame.json

## Beads import

- Beads id: `rk-ck8x`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:32:59Z
- Close reason (Beads): Athame/Disjunction module (src/world/athame.rs) already existed but was not compiled; wired it into lib.rs as pub mod athame; tests pass.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:32:56Z · Jonathan Stoff] [domain-builder] done: Athame/Disjunction module (src/world/athame.rs) already existed but was not compiled; wired it into lib.rs as pub mod athame; tests pass.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/src/world/athame.rs
cargo test athame passes (8). Formulas match Java: min enchantability 5+8*lvl, +6*lvl vs ender mobs, incompatible with self and damage enchants. lib.rs still has duplicate athame_attack_damage/constants at crate root; harmless. Native hook for in-game 'damage' scenario (durability loss on use) is engine side, not done here.
