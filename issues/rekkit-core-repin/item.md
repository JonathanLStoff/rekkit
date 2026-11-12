---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- core
- for:redpower_2
- mod
- mod:rekkit_core
- path:crates/rekkit_core
- tier:Hard
- wave:core
beads_id: rk-p2g.1
epic: rekkit-core-3
closed: 2026-11-12
---

# rekkit_core: repin HORIZON_REVISION constant to ed102e05

creative tabs require ed102e05

## Beads import

- Beads id: `rk-p2g.1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T23:02:08Z
- Closed (Beads): 2026-11-01T23:04:15Z
- Close reason (Beads): Repinned HORIZON_REVISION constant to ed102e0592eded5585c610c205488f771f52ac40 in rekkit_core and example_mod

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-01T23:04:14Z · Jonathan Stoff] [domain-builder-pro] done: Repinned HORIZON_REVISION constant to ed102e0592eded5585c610c205488f771f52ac40 in rekkit_core and example_mod
Files: crates/rekkit_core/src/lib.rs, mods/example_mod/src/lib.rs
STATUS: done
FILES:
- crates/rekkit_core/src/lib.rs
- mods/example_mod/src/lib.rs
TEST: cargo test --manifest-path crates/rekkit_core/Cargo.toml (11 passed), cargo test --manifest-path mods/redpower_2/Cargo.toml (29 passed), cargo test --manifest-path mods/example_mod/Cargo.toml (6 passed)
DEFERRED: none
CORE-NEEDS: none
