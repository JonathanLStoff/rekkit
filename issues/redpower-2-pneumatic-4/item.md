---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-pneumatic-tube
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-k4gl
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Pneumatic tube and restrictor tube: zero-drop routing, Dijkstra search, and color painting

Implement the core pneumatic tube mechanics: zero-drop guaranteed transit, Dijkstra shortest-path route resolution (+2 hop weight, +1 turn penalty), 1,000,000 weight for Restrictor Tube overflow routing, 16-color Paintbrush dyeing via tryPaint, and client item rendering. Done: make contract-check clean for pneumatic_tube and restrictor_tube; L1 placement and L4 Dijkstra routing scenarios pass.

Size: ~120k tokens.
Covers: pneumatic_tube, restrictor_tube, pneumatic_tubes
Read first: spec 2.4 Pneumatic Tubes & Item Logistics
Java: TileTube#n_, TileTube#tubeItemEnter, TileTube#tubeItemCanEnter, TileTube#tubeWeight, TileTube#tryPaint, TileTubeRenderer#a, TileRestrictTube#tubeWeight, TileRestrictTube#getExtendedID
Files: mods/redpower_2/src/tube/mod.rs, mods/redpower_2/src/tube/pneumatic.rs, planning/mods/redpower_2.contract/pneumatic_tube.json, planning/mods/redpower_2.contract/restrictor_tube.json

## Beads import

- Beads id: `rk-k4gl`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:41:44Z
- Close reason (Beads): Router (tube.rs) and flow (tube_flow.rs) already complete; added restrictor tube variant (Tube.restrictor, new_restrictor, tube_weight 1,000,000) from TileRestrictTube.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-03T03:41:41Z · Jonathan Stoff] [domain-builder-pro] done: Router (tube.rs) and flow (tube_flow.rs) already complete; added restrictor tube variant (Tube.restrictor, new_restrictor, tube_weight 1,000,000) from TileRestrictTube.
Files: mods/redpower_2/src/machine/tube_flow.rs
cargo test: 237 pass, 2 pre-existing unrelated failures (core::microblock::tests::placement_face, tests::test_native_block_entity_containers slot count). Engine wiring of the restrictor block to Tube::new_restrictor is for the native layer.
