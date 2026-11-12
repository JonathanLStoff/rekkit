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
beads_id: rk-qst.4.4
epic: redpower-2-4
closed: 2026-11-12
---

# builder: fix Red Alloy vanilla mapping to spec formula strength / 17 and update test_red_alloy_wire_distance_and_vanilla_signal

Spec §2.2 line 52 specifies strength / 17; src/lib.rs has internal_to_vanilla_redstone using an incorrect interpolated curve tested by mod_tests.rs.

## Beads import

- Beads id: `rk-qst.4.4`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:01:08Z
- Closed (Beads): 2026-11-01T21:13:33Z
- Close reason (Beads): Unified Red Alloy signal mapping to spec formula strength / 17 everywhere and updated test assertions

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T21:13:32Z · Jonathan Stoff] [domain-builder-pro] done: Unified Red Alloy signal mapping to spec formula strength / 17 everywhere and updated test assertions
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
STATUS: done
FILES: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml pass (19 unit, 10 integration)
DEFERRED: none
CORE-NEEDS: none
