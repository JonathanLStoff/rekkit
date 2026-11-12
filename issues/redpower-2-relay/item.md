---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-relay
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-pc6e
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-4']
closed: 2026-11-12
---

# redpower_2: Relay: 9-slot buffer chest with periodic automatic tube ejection

Implement the Relay: 9-slot inventory buffer chest that automatically ejects 1 item stack every 10 ticks into an attached tube. Done: make contract-check clean for relay; L1 placement, L3 9-slot inventory GUI, L4 periodic ejection, and L5 chunk persistence scenarios pass.

Size: ~80k tokens.
Covers: relay
Read first: spec 3.2 Tube & Logistics Machines
Java: TileRelay#onTileTick, TileRelay#handleExtract, TileRelay#n_, TileRelay#onBlockActivated
Files: mods/redpower_2/src/machine/relay.rs, planning/mods/redpower_2.contract/relay.json

## Beads import

- Beads id: `rk-pc6e`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:34:31Z
- Close reason (Beads): Relay: 9-slot buffer chest with periodic automatic tube ejection verified with tests.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:34:29Z · Jonathan Stoff] [domain-builder] done: Relay: 9-slot buffer chest with periodic automatic tube ejection verified with tests.
Files: mods/redpower_2/src/lib.rs, mods/redpower_2/src/machine/base.rs
Relay tile entity, 9-slot container, native screen GUI, and tube ejection verified.
