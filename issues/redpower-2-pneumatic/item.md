---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-tube-routing-lib
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-9g1a
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-3']
closed: 2026-11-12
---

# redpower_2: Pneumatic Tube Routing & Dijkstra RouteFinder: TubeLib

Implement Dijkstra shortest-path pneumatic tube routing in TubeLib. RouteFinder traverses the pneumatic network to identify the nearest eligible recipient inventory. InRouteFinder evaluates destination inventory capacity via filter rules; OutRouteFinder and RequestRouteFinder search paths for items and pull requests. Enforces exact path weighting: +2 weight per standard tube hop, +1 penalty for turns deviating from entry direction, +1,000,000 weight for Restrictor Tubes, and strict 16-color dye tube filtering (unpainted or matching color only). Done: unit tests pass confirming shortest route selection, turn penalty, restrictor overflow avoidance, and color filtering.

Size: ~160k tokens.
Covers: pneumatic_tubes
Read first: spec 2.4 Pneumatic Tubes & Item Logistics
Java: TubeLib#findRoute, TubeLib#getConnections, TubeLib#isConSide, TubeLib#addToTubeRoute, TubeLib.RouteFinder#find, TubeLib.InRouteFinder#addPoint, TubeLib.OutRouteFinder#addPoint, TubeLib.RequestRouteFinder#addPoint
Files: mods/redpower_2/src/tube/routing.rs

## Beads import

- Beads id: `rk-9g1a`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:33:46Z
- Close reason (Beads): Pneumatic Tube Routing & Dijkstra RouteFinder: TubeLib implemented with shortest-path finding, turn penalties, restrictor tube weighting, and color filtering verified by tests.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:33:44Z · Jonathan Stoff] [domain-builder-pro] done: Pneumatic Tube Routing & Dijkstra RouteFinder: TubeLib implemented with shortest-path finding, turn penalties, restrictor tube weighting, and color filtering verified by tests.
Files: mods/redpower_2/src/machine/tube.rs
RouteFinder, InRouteFinder, OutRouteFinder, restrictor weight (1,000,000), and color filtering pass unit and integration tests.
