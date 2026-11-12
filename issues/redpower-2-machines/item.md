---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- contract
- domain-builder-pro
- hook-engineer
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-k982
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: machines and tools do nothing in the game

The domain logic has unit tests but is not wired into the world: a solar panel at noon makes no power and a voltmeter right-clicked on it prints nothing (the original: chat 'Reading %.2fV %.2fA (%.2fW)'). The voltmeter is a compass. Every generator, machine and tool in the contract's scenarios must work in the game; the harness plays them (stage 10).

## Beads import

- Beads id: `rk-k982`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-02T14:39:05Z
- Closed (Beads): 2026-11-08T14:18:18Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:04Z · @beads-import

[2026-11-08T14:18:00Z · unknown] [hook-engineer] complete: Wired machines, generators, engines, microblocks, wiring networks, and tools into native world interaction hooks and block entity lifecycle.
