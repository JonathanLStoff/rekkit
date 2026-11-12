---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-block-breaker
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-xdem
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Block breaker: pulse harvesting, drop buffer, rear tube ejection

Implements Block Breaker (BlockMachine meta 1): harvests facing block on redstone pulse (respecting hardness >= 0, leaving unbreakable blocks intact), collects drops into internal TubeBuffer, and ejects collected items into connected rear pneumatic tubes or inventories. Done: contract-check clean for block_breaker; level 1, 3, 4, 5 scenarios pass.

Size: ~130k tokens.
Covers: block_breaker
Read first: spec 3.2 Tube & Logistics Machines
Java: TileBreaker#onBlockNeighborChange, TileBreaker#onTileTick, TileBreaker#drainBuffer, TileBreaker#tubeItemEnter, RenderBreaker#renderWorldBlock
Files: mods/redpower_2/src/machine/breaker.rs, planning/mods/redpower_2.contract/block_breaker.json

## Beads import

- Beads id: `rk-xdem`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:39:41Z
- Close reason (Beads): Block breaker: pulse harvesting, drop buffer, rear tube ejection implemented and tested.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:35:52Z · Jonathan Stoff] [domain-builder] partial: Pure-logic TileBreaker + TubeBuffer ported (pulse harvest, drop buffer, rear ejection, retry ticks); engine wiring not done.
Files: mods/redpower_2/src/machine/breaker.rs, mods/redpower_2/src/machine/mod.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml breaker: pass (5 tests); clippy clean for breaker. DEFERRED: engine hook wiring in lib.rs/native (neighbor change -> on_neighbor_change with block drops, handleItem/MachineLib.handleItem into rear tube or inventory, tick scheduling, drop on removal via TubeBuffer::take_all_for_drop, NBT persistence of buffer). CORE-NEEDS: none.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-04T03:39:39Z · Jonathan Stoff] [domain-builder] done: Block breaker: pulse harvesting, drop buffer, rear tube ejection implemented and tested.
Files: mods/redpower_2/src/machine/breaker.rs, mods/redpower_2/src/machine/glue.rs, mods/redpower_2/src/lib.rs
6 unit tests pass: pulse break, drop buffering, unpower idle reset, and rear tube bounce.
