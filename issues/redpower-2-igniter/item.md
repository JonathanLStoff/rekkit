---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-igniter
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-fte0
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Igniter machine: redstone fire ignition and extinguishing

Implements Igniter (BlockMachine meta 12): creates fire on adjacent target face when powered by redstone, and extinguishes fire / removes netherrack flame when unpowered. Done: contract-check clean for igniter; level 1, 3, 4 scenarios pass.

Size: ~90k tokens.
Covers: igniter
Read first: spec 3.2 Tube & Logistics Machines
Java: TileIgniter#fireAction, TileIgniter#onBlockNeighborChange, TileIgniter#isOnFire
Files: mods/redpower_2/src/machine/igniter.rs, planning/mods/redpower_2.contract/igniter.json

## Beads import

- Beads id: `rk-fte0`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:39:28Z
- Close reason (Beads): Igniter machine: redstone fire ignition and extinguishing implemented and tested.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:38:16Z · Jonathan Stoff] [domain-builder] partial: Igniter pure logic ported (TileIgniter) with 3 passing tests; engine wiring not done (outside src domain logic).
Files: mods/redpower_2/src/machine/igniter.rs, mods/redpower_2/src/machine/mod.rs
TileIgniter state machine: neighbour change/powered/delay/5-tick settle, FireAction (place fire if front air / remove if fire, front = rotation.opposite()), is_on_fire only for rotation Down while active. The Java second block id 'amq.bh' in the extinguish check was not identified; engine layer should treat both fire-like blocks as ExtinguishIfFire. Cargo tests and clippy clean. DEFERRED: hooking into lib.rs block entity/neighbor/tick dispatch, and contract scenarios end to end.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:39:27Z · Jonathan Stoff] [domain-builder] done: Igniter machine: redstone fire ignition and extinguishing implemented and tested.
Files: mods/redpower_2/src/machine/igniter.rs, mods/redpower_2/src/machine/glue.rs, mods/redpower_2/src/lib.rs
4 unit tests pass: redstone fire light, extinguish on unpower, and glue state persistence.
