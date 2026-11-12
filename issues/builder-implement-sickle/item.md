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
beads_id: rk-qst.4.2
epic: redpower-2-4
closed: 2026-11-12
---

# builder: implement Sickle leaf (3x3x3) and crop (5x5) harvesting radius calculations

Spec §4.2 lines 290-292 specifies Sickles harvest leaves in a 3x3x3 volume (leafRadius = 1) and crops/grass in a 5x5 flat area (cropRadius = 2).

## Beads import

- Beads id: `rk-qst.4.2`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:01:05Z
- Closed (Beads): 2026-11-01T21:13:27Z
- Close reason (Beads): Implemented Sickle leaf (3x3x3) and crop (5x5) harvesting radius calculations and variant durabilities

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T21:13:26Z · Jonathan Stoff] [domain-builder-pro] done: Implemented Sickle leaf (3x3x3) and crop (5x5) harvesting radius calculations and variant durabilities
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
STATUS: done
FILES: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml pass (19 unit, 10 integration)
DEFERRED: none
CORE-NEEDS: none
