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
beads_id: rk-qst.3.317
epic: redpower-2-3-2
closed: 2026-11-12
---

# Align 6502 opcodes in test_6502_assembly_automation_integration

test_6502_assembly_automation_integration used 0xAD (LDA abs) and 0xC9 (CMP imm) which should either be added to step_tick or test should use supported instructions (LDA #imm / STA abs).

## Beads import

- Beads id: `rk-qst.3.317`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-04T05:56:41Z
- Closed (Beads): 2026-11-08T13:57:19Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-08T13:57:04Z · unknown] [test-smith] complete: Aligned 6502 opcodes in test_6502_assembly_automation_integration to use supported instructions, ensuring full integration test pass.
