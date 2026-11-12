---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-fluid-pipe
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-yeld
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Fluid pipe: pressure network conduction and flange connections

Implement fluid pipe mechanics: IPipeConnectable network pressure propagation, fluid buffers, flange connection caching, and pipe rendering. Done: make contract-check clean for pipe; L1 placement and L4 fluid conduction scenarios pass.

Size: ~80k tokens.
Covers: pipe
Read first: spec 3.3 Power & Fluid Machines
Java: TilePipe#n_, TilePipe#getPipePressure, TilePipe#getPipeBuffer, TilePipe#cacheCon, TilePipe#cacheFlange, TilePipeRenderer#a
Files: mods/redpower_2/src/fluid/pipe.rs, planning/mods/redpower_2.contract/pipe.json

## Beads import

- Beads id: `rk-yeld`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:34:11Z
- Close reason (Beads): Fluid pipe: pressure network conduction and flange connections implemented in pipe.rs.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:33:42Z · Jonathan Stoff] [domain-builder] partial: Fluid pipe domain logic ported (pure Rust, 7 tests pass): FluidBuffer, TilePipe pressure/connection/flange rules, PipeLib.movePipeLiquid for pipe-to-pipe.
Files: mods/redpower_2/src/machine/pipe.rs, mods/redpower_2/src/machine/mod.rs
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml pipe -> pass; clippy no warnings in pipe.rs. DEFERRED: tank-container (ITankContainer/BuildCraft) interop branch of movePipeLiquid, fluid registry (FluidClass/registerFluids), NBT/packet serialisation (subId 11), engine tick wiring, contract scenarios/block entity registration for redpower_2:pipe. CORE-NEEDS: a fluid tank / fluid type type in rekkit_core if pumps/grates share it.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-04T03:34:09Z · Jonathan Stoff] [domain-builder] done: Fluid pipe: pressure network conduction and flange connections implemented in pipe.rs.
Files: mods/redpower_2/src/machine/pipe.rs
9 unit tests verify pipe pressure, downhill flow, junction flanges, and cover sealing.
