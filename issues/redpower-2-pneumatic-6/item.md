---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-tube-flow
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-pxu0
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-5']
closed: 2026-11-12
---

# redpower_2: Pneumatic tube in-flight item simulation and flow dynamics

Implement in-flight pneumatic tube item simulation (TubeItem, TubeFlow). Advances item progress through tube sections (standard 3.0 blocks/sec, magtube 16.0 blocks/tick), handles junction route transitions, color tag preservation, blockage bounce-back into origin buffers, and full NBT persistence. Done: unit tests verify item movement tick advancement, arrival scheduling, bounce handling on closed endpoints, and NBT roundtrip.

Size: ~130k tokens.
Read first: spec 2.4 Pneumatic Tubes & Item Logistics
Java: TubeItem#readFromNBT, TubeItem#writeToNBT, TubeFlow#update, TubeFlow#schedule, TubeFlow#handleItem, TubeFlow#add, TubeFlow#onRemove
Files: mods/redpower_2/src/tubes/flow.rs

## Beads import

- Beads id: `rk-pxu0`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:39:05Z
- Close reason (Beads): Added Tube (TileTube) with TubeFlow in-flight item simulation: progress/power stepping, schedule routing modes, handoff, bounce, colour rules; 12 unit tests pass, clippy clean.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:39:02Z · Jonathan Stoff] [domain-builder-pro] done: Added Tube (TileTube) with TubeFlow in-flight item simulation: progress/power stepping, schedule routing modes, handoff, bounce, colour rules; 12 unit tests pass, clippy clean.
Files: mods/redpower_2/src/machine/tube_flow.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/machine/magtube.rs
TubeItem gained serde-default `priority`. Engine must implement FlowContext (find_route wrapping tube::find_route with mode, pass_to_tube, add_to_inventory, eject) and call Tube::update(getConnections) per tick. DEFERRED: TileRestrictTube wiring, client-side branches, packet sync (engine layer).
