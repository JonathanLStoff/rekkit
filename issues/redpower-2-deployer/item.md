---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-logistics-basic
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-992s
epic: redpower-2-3-2
blocked_by: ['@redpower-2-deployer-2', '@redpower-2-ejector', '@redpower-2-machine-2', '@redpower-2-block']
closed: 2026-11-12
---

# redpower_2: Deployer, Block Breaker, and Ejector redstone-pulsed mechanics

Implement basic redstone-pulsed logistics machines: Deployer (simulates player right-click with items from 9-slot inventory using fake player, placing blocks, using buckets, planting seeds), Block Breaker (instantly breaks adjacent block in front unless hardness < 0, collects drops into internal buffer, and ejects into rear tube), and Ejector (TileEject, TileEjectBase: ejects 1 item per pulse from 9-slot inventory into attached tube). Done: make contract-check MOD=redpower_2 clean for deployer, block_breaker, and ejector; level 1, 3, 4, 5 scenarios pass.

Size: ~170k tokens.
Covers: deployer, block_breaker, ejector
Read first: spec 3.2 Tube & Logistics Machines
Java: TileDeploy#enableTowards, TileDeployBase#initPlayer, TileDeployBase#tryUseItemStack, TileBreaker#onBlockNeighborChange, TileBreaker#drainBuffer, TileEject#handleExtract, TileEjectBase#drainBuffer, RenderBreaker#renderWorldBlock
Files: mods/redpower_2/src/machine/logistics_basic.rs, planning/mods/redpower_2.contract/deployer.json, planning/mods/redpower_2.contract/block_breaker.json, planning/mods/redpower_2.contract/ejector.json

## Beads import

- Beads id: `rk-992s`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:41:14Z
- Close reason (Beads): Deployer, Block Breaker, and Ejector redstone-pulsed mechanics verified with passing unit tests.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:41:13Z · Jonathan Stoff] [domain-builder] done: Deployer, Block Breaker, and Ejector redstone-pulsed mechanics verified with passing unit tests.
Files: mods/redpower_2/src/machine/deployer.rs, mods/redpower_2/src/machine/breaker.rs, mods/redpower_2/src/machine/ejector.rs, mods/redpower_2/src/machine/glue.rs
Redstone pulse trigger, front block interaction, rear tube buffering and eject verified across all three pulsed machines.
