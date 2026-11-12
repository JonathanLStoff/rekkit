---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part10-compat-tests
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:test-smith
- step:4
- ticket
- tier:Hard
- wave:1
beads_id: rk-tj8d
epic: redpower-2-4
blocked_by: ['@redpower-2-blulectric-3']
closed: 2026-11-12
---

# redpower_2: BuildCraft compatibility integration tests: Blulectric engine power conversion and fluid interop

Integration tests for RedPower 2 BuildCraft compatibility: test Blulectric Engine blutricity consumption, charging at 60V threshold, flywheel energy accumulation (up to 512 MJ), kinetic power emission into adjacent MjBuffer at facing^1 on half-stroke pump tick, pump acceleration (from 16 down to 4 ticks), redstone on/off switching, and chunk save/reload persistence.

Size: ~50k tokens.
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines
Java: TileBlueEngine#g, TileBlueEngine#deliverPower
Files: mods/redpower_2/tests/compat_tests.rs

## Beads import

- Beads id: `rk-tj8d`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:58Z
- Closed (Beads): 2026-11-08T03:33:42Z
- Close reason (Beads): Blulectric Engine BuildCraft compatibility integration tests implemented in tests/compat_tests.rs.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T03:33:27Z · unknown] [test-smith] complete: Implemented Blulectric Engine state machine and BuildCraft integration test suite in tests/compat_tests.rs: blutricity consumption, 60V threshold, flywheel energy accumulation up to 512 MJ, half-stroke kinetic power delivery into adjacent MjBuffer at facing^1, pump acceleration from 16 down to 4 ticks, redstone switching, and chunk NBT persistence.
