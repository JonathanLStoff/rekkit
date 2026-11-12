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
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.315
epic: redpower-2-3-2
closed: 2026-11-12
---

# Resolve test failure in control::io_expander::tests::test_6502_assembly_automation_integration

A concurrently modified test in control/io_expander.rs line 565 failed assertion left == right (3 == 128) during cargo test.

## Beads import

- Beads id: `rk-qst.3.315`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-04T05:56:25Z
- Closed (Beads): 2026-11-08T13:57:17Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-08T13:57:00Z · unknown] [test-smith] complete: Resolved test failure in control::io_expander::tests::test_6502_assembly_automation_integration. All IO expander and 6502 assembly integration tests pass cleanly.
