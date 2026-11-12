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
beads_id: rk-qst.4.3
epic: redpower-2-4
closed: 2026-11-12
---

# builder: expand BLOCK_IDS (113) and ITEM_IDS (88) in src/lib.rs to match spec §6 registry and update test assertions

src/lib.rs truncates BLOCK_IDS to 42 and ITEM_IDS to 29 with inaccurate aliases, while content/data already implements all 113 blocks and 88 items from spec §6.

## Beads import

- Beads id: `rk-qst.4.3`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:01:06Z
- Closed (Beads): 2026-11-01T21:13:29Z
- Close reason (Beads): Expanded BLOCK_IDS to 113 canonical entries and ITEM_IDS to 88 entries per spec §6 and updated test assertions

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T21:13:28Z · Jonathan Stoff] [domain-builder-pro] done: Expanded BLOCK_IDS to 113 canonical entries and ITEM_IDS to 88 entries per spec §6 and updated test assertions
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
STATUS: done
FILES: mods/redpower_2/src/lib.rs, mods/redpower_2/tests/mod_tests.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml pass (19 unit, 10 integration)
DEFERRED: none
CORE-NEEDS: none
