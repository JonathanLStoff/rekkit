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
beads_id: rk-qst.4.8
epic: redpower-2-4
closed: 2026-11-12
---

# builder: fix Counter logic gate to emit output at minimum count (0) as well as maximum count per spec §2.3

Spec §2.3 line 92 states Counter outputs signal at minimum count (0) or maximum count (CountMax); step_tick currently checks only >= max_count.

## Beads import

- Beads id: `rk-qst.4.8`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:01:13Z
- Closed (Beads): 2026-11-01T21:13:37Z
- Close reason (Beads): Fixed Counter logic gate to emit output signal at minimum count (0) as well as maximum count

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T21:13:36Z · Jonathan Stoff] [domain-builder-pro] done: Fixed Counter logic gate to emit output signal at minimum count (0) as well as maximum count
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
STATUS: done
FILES: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml pass (19 unit, 10 integration)
DEFERRED: none
CORE-NEEDS: none
