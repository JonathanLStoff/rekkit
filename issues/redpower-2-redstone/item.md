---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-redstone-tube
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-p6vs
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pneumatic-4']
closed: 2026-11-12
---

# redpower_2: Redstone tube: dual pneumatic transport and redstone signal conduction

Implement Redstone Tube combining pneumatic item transport with IRedPowerWiring redstone conduction across connected tube networks. Done: make contract-check clean for redstone_tube; L1 placement and L4 dual conduction scenarios pass.

Size: ~90k tokens.
Covers: redstone_tube
Read first: spec 2.2 Wiring System, 2.4 Pneumatic Tubes & Item Logistics
Java: TileRedstoneTube#getPoweringMask, TileRedstoneTube#updateCurrentStrength, TileRedstoneTube#scanPoweringStrength, TileRedstoneTube#onBlockNeighborChange, TileRedstoneTube#isBlockWeakPoweringTo
Files: mods/redpower_2/src/tube/redstone.rs, planning/mods/redpower_2.contract/redstone_tube.json

## Beads import

- Beads id: `rk-p6vs`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:34:28Z
- Close reason (Beads): Redstone tube: dual pneumatic transport and redstone signal conduction verified.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:34:26Z · Jonathan Stoff] [domain-builder] done: Redstone tube: dual pneumatic transport and redstone signal conduction verified.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/redstone_tube.json, mods/redpower_2/src/machine/proxy.rs
Redstone tube block definition, renderer dispatch, and dual conduction traits verified.
