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
beads_id: rk-qst.3.258
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: BatteryBoxState.tick charge/discharge should add n storage units per tick like TileBatteryBox

Current code converts n*1000 W by /200, Java does Storage += n (capped), n=min((Charge-900)/10,10) with Charge=int(V*10).

## Beads import

- Beads id: `rk-qst.3.258`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-03T03:36:24Z
- Closed (Beads): 2026-11-08T14:13:39Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-08T14:11:48Z · unknown] [domain-builder] complete: Completed implementation, verification, and tests for rk-qst.3.258
