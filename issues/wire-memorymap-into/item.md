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
beads_id: rk-qst.3.264
epic: redpower-2-3-2
closed: 2026-11-12
---

# Wire MemoryMap into Cpu6502 (lib.rs) so addresses >=0x2000 read via backplanes

Cpu6502 uses flat 64KB RAM; original returns 255 without RAM modules

## Beads import

- Beads id: `rk-qst.3.264`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:37:46Z
- Closed (Beads): 2026-11-08T14:13:52Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-08T14:12:08Z · unknown] [domain-builder] complete: Completed implementation, verification, and tests for rk-qst.3.264
