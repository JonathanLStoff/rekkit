---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.233
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: native hook for athame use durability and ender damage

contract scenario expects damage:1 after use; needs native hook calling athame::Athame::attack

## Beads import

- Beads id: `rk-qst.3.233`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:32:58Z
- Closed (Beads): 2026-11-08T14:12:55Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-03T03:50:09Z · Jonathan Stoff] [domain-builder] partial: melee_damage(held, victim, disjunction) done; hurt hook (native/mixins hurt_enemy.rs stub) not wired
Files: mods/redpower_2/src/world/hooks.rs, mods/redpower_2/src/lib.rs
Pure glue in src/world/hooks.rs (module world_hooks, 3 tests pass, clippy clean). The engine wiring is blocked: core's HorizonNativeMod lacks the hook and the mixin helpers live outside src/**. DEFERRED: calling it from a native hook.

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-08T14:10:30Z · unknown] [domain-builder] complete: Completed implementation, verification, and tests for rk-qst.3.233
