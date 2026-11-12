---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- core
- for:redpower_2
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.1.2
epic: rekkit-core-3
closed: 2026-11-12
---

# rekkit_core: update Restrictor Tube weight constant to 1,000,000

TileRestrictTube.java specifies tubeWeight = 1,000,000, whereas rekkit_core currently defaults to 500.

## Beads import

- Beads id: `rk-qst.1.2`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T19:55:44Z
- Closed (Beads): 2026-11-01T21:32:40Z
- Close reason (Beads): Implemented locally in redpower_2; core enhancement filed

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-01T19:57:41Z · Jonathan Stoff] Conductor verified: TileRestrictTube.tubeWeight returns 1000000 in both 2.0pr5b2 and 2.0pr6; rekkit_core grid::TubeEdge::RESTRICTOR_WEIGHT is 500. One-constant core fix; moved under redpower_2 step 3 for domain-builder-pro (owner of crates/rekkit_core/src) to fix with the rekkit_core tests.
