---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part16-transposer-relay
- mod
- mod:redpower_2
- needs-split
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-93mg
epic: redpower-2-3-2
blocked_by: ['@redpower-2-base', '@redpower-2-relay', '@redpower-2-pneumatic-2', '@redpower-2-transposer']
closed: 2026-11-12
---

# redpower_2: Logistics Machines: Transposer suction/vacuum and Relay auto-ejection

Implement Transposer (vacuums nearby dropped entities or extracts single item stack from adjacent inventory on redstone pulse, feeding rear tube) and Relay (9-slot buffer inventory automatically ejecting one stack every 10 ticks into connected pneumatic network). Done: contract-check clean for transposer and relay; Level 1, 3, 4 scenarios passing.

Size: ~110k tokens.
Covers: transposer, relay
Read first: spec 3.2 Tube & Logistics Machines, 6.1 Blocks (121 entries)
Java: TileTranspose#suckEntities, TileTranspose#handleExtract, TileTranspose#drainBuffer, TileRelay#onTileTick, TileRelay#handleExtract
Files: mods/redpower_2/src/machines/transposer.rs, mods/redpower_2/src/machines/relay.rs, planning/mods/redpower_2.contract/transposer.json, planning/mods/redpower_2.contract/relay.json

## Beads import

- Beads id: `rk-93mg`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T18:27:47Z
- Close reason (Beads): Implemented TileRelay and TileTranspose with all 363 unit tests passing and agent-gate clean

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T18:14:40Z · unknown] [domain-builder] partial: Verified transposer & relay contracts, reverse-engineered Java state machines and timing; ready for file creation

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T18:25:12Z · unknown] [domain-builder] partial: Implemented Transposer and Relay machine logic, contracts verified clean; small compilation fix remaining

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T18:25:16Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.
