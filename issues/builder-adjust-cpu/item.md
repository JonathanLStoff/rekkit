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
beads_id: rk-qst.4.9
epic: redpower-2-4
closed: 2026-11-12
---

# builder: adjust CPU cycles per tick limit and cycle accumulator to 1,000 / 100,000 slice per spec §2.7 and Java TileCPU

Spec §2.7 line 153 and TileCPU.java:198-208 define a +1,000 cycle/tick slice with a 100,000 cycle burst cap; src/lib.rs currently hardcodes MAX_CYCLES_PER_TICK = 4096.

## Beads import

- Beads id: `rk-qst.4.9`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:01:14Z
- Closed (Beads): 2026-11-01T21:13:39Z
- Close reason (Beads): Adjusted MOS 6502 cycle limit and accumulator to +1,000 cycles/tick slice with 100,000 burst cap

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T21:13:38Z · Jonathan Stoff] [domain-builder-pro] done: Adjusted MOS 6502 cycle limit and accumulator to +1,000 cycles/tick slice with 100,000 burst cap
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
STATUS: done
FILES: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml pass (19 unit, 10 integration)
DEFERRED: none
CORE-NEEDS: none
