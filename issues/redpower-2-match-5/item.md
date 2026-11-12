---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- content-author
- contract
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-8ila
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: match the original: and_gate,wire_red_alloy (content-author)

Differs from the original (scenario), for content-author:
- scenario 'and gate requires both inputs active' has no test level ("level": 1-6: 1 placement & hitbox, 2 item use, 3 block function (energy, fuel, storage, GUI slots), 4 interaction with other blocks, 5 persistence (chunk reloaded from disk), 6 automation & sides (hoppers, tubes, sided access))

Recheck: make contract-check MOD=redpower_2

## Beads import

- Beads id: `rk-8ila`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-02T15:41:20Z
- Closed (Beads): 2026-11-03T03:28:21Z
- Close reason (Beads): No difference from the original any more (make contract-check)
