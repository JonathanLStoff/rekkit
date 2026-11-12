---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-tube-flow-buffer
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-tx6b
epic: redpower-2-3-2
blocked_by: ['@redpower-2-3d']
closed: 2026-11-12
---

# redpower_2: Pneumatic Tube Transport, Buffers & In-Flight Items: TubeItem, TubeBuffer, TubeFlow

Implement pneumatic tube in-flight item transport mechanics and machine buffer queues. TubeItem encapsulates an in-transit ItemStack with movement progress (0..100), travel direction, 16-color dye tag, speed, and scheduling priority. TubeBuffer provides FIFO queuing with backpressure, bounce-back handling when destination tubes or inventories become blocked, and drop-on-break persistence. TubeFlow simulates tick-by-tick payload motion along tube segments, handles branch collisions, and hands off items to adjacent tubes or recipient inventories with zero item drops. Done: unit tests pass for TubeItem NBT roundtrip, TubeBuffer FIFO/bounce operations, and TubeFlow progression ticks.

Size: ~150k tokens.
Read first: spec 2.4 Pneumatic Tubes & Item Logistics
Java: TubeItem#readFromNBT, TubeItem#writeToNBT, TubeBuffer#add, TubeBuffer#addNewColor, TubeBuffer#addBounce, TubeBuffer#pop, TubeFlow#update, TubeFlow#handleItem, TubeFlow#schedule
Files: mods/redpower_2/src/tube/flow.rs, mods/redpower_2/src/tube/buffer.rs

## Beads import

- Beads id: `rk-tx6b`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:31:31Z
- Close reason (Beads): Pneumatic Tube Transport, Buffers & In-Flight Items: TubeItem, TubeBuffer, TubeFlow implemented with 28 unit tests and routing integration test passing.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:31:29Z · Jonathan Stoff] [domain-builder-pro] done: Pneumatic Tube Transport, Buffers & In-Flight Items: TubeItem, TubeBuffer, TubeFlow implemented with 28 unit tests and routing integration test passing.
Files: mods/redpower_2/src/machine/tube.rs, mods/redpower_2/src/machine/tube_flow.rs
Dijkstra routing weights, colour tagging, tube bounce, FIFO buffers, and in-flight transit all passing tests.
