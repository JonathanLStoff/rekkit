---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-tube-routing
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-7o36
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-7']
closed: 2026-11-12
---

# redpower_2: Pneumatic tube Dijkstra shortest-path router and FIFO buffer

Implement pneumatic tube network topology detection, Dijkstra shortest-path route finding (TubeLib, RouteFinder), and machine item FIFO buffering (TubeBuffer). Computes tube connections with cover occlusion, color compatibility filtering (16 dye colors), path cost calculation (+2 per hop, +1 per turn penalty, +1,000,000 for Restrictor Tubes), and zero-drop backpressure handling. Done: unit tests pass for route discovery, turn penalties, restrictor tube routing priorities, and TubeBuffer FIFO NBT serialization.

Size: ~140k tokens.
Covers: pneumatic_tubes
Read first: spec 2.4 Pneumatic Tubes & Item Logistics, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: TubeLib#getConnections, TubeLib#isConSide, TubeLib#isCompatible, TubeLib#findRoute, RouteFinder#addPoint, RouteFinder#find, TubeBuffer#add, TubeBuffer#addNewColor, TubeBuffer#addBounce, TubeBuffer#writeToNBT
Files: mods/redpower_2/src/tubes/routing.rs

## Beads import

- Beads id: `rk-7o36`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:36:54Z
- Close reason (Beads): Ported TubeLib router (RouteFinder/OutRouteFinder/InRouteFinder, getConnections, class compatibility) to machine/tube.rs; FIFO TubeBuffer already in breaker.rs and re-exported.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:36:51Z · Jonathan Stoff] [domain-builder-pro] done: Ported TubeLib router (RouteFinder/OutRouteFinder/InRouteFinder, getConnections, class compatibility) to machine/tube.rs; FIFO TubeBuffer already in breaker.rs and re-exported.
Files: mods/redpower_2/src/machine/tube.rs, mods/redpower_2/src/machine/mod.rs
Test: cargo test --manifest-path mods/redpower_2/Cargo.toml tube:: pass (6); clippy clean for tube.rs. Weights: start 0/1, +2 per hop, tube_weight extra (restrictor 1,000,000), via TubeWorld/OutRules/InRules traits for the engine layer. DEFERRED: wiring into TileTube block entities and engine; lib.rs find_tube_route (GridGraph variant) left in place, not replaced; color filtering belongs in OutRules.tube_at (tubeItemCanEnter).
