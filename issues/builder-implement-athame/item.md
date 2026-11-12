---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:4
- tier:Hard
- wave:1
beads_id: rk-qst.4.1
epic: redpower-2-4
closed: 2026-11-12
---

# builder: implement Athame 25 damage vs Endermen, Disjunction enchantment, and Vorpal 100 bonus damage & decapitation hazard

Spec §4.2, §8, and §10 specify Athame dealing 25 damage vs Endermen/Ender Dragon, Disjunction +6*level damage, and Vorpal 2*level^2 % chance for 100 extra critical damage with decapitation on <= -20 HP overkill (API 39).

## Beads import

- Beads id: `rk-qst.4.1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:01:04Z
- Closed (Beads): 2026-11-01T21:13:23Z
- Close reason (Beads): Implemented Athame 25 damage vs Ender mobs, Disjunction (+6*lvl), Vorpal (2*lvl^2% for +100 damage), and decapitation (overkill <= -20 HP)

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T21:13:22Z · Jonathan Stoff] [domain-builder-pro] done: Implemented Athame 25 damage vs Ender mobs, Disjunction (+6*lvl), Vorpal (2*lvl^2% for +100 damage), and decapitation (overkill <= -20 HP)
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
STATUS: done
FILES: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml pass (19 unit, 10 integration)
DEFERRED: none
CORE-NEEDS: none
