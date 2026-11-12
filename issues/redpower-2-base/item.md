---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-machine-blocks-base
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-af9o
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3', '@redpower-2-deployer-2', '@redpower-2-igniter', '@redpower-2-relay', '@redpower-2-transposer', '@redpower-2-ejector', '@redpower-2-block']
closed: 2026-11-12
---

# redpower_2: Base Machine blocks registration: Deployer, Breaker, Transposer, Igniter, Relay

Implement base BlockMachine registration, facing orientations (6 directions), metadata dispatch, harvest properties, ItemRenderMachine rendering, and EntityPlayerFake simulation helper for automated actions. Configures blocks for Deployer (0), Block Breaker (1), Transposer (2), Igniter (12), and Relay (15). Write contract files for these 5 machines. Done: make contract-check clean for deployer, block_breaker, transposer, igniter, and relay; level 1 placement, facing, and drops.

Size: ~110k tokens.
Covers: deployer, block_breaker, transposer, igniter, relay
Read first: spec 3.2 Tube & Logistics Machines
Java: com.eloraam.redpower.machine.BlockMachine#getBlockTextureFromSideAndMetadata, com.eloraam.redpower.machine.BlockMachine#onBlockPlacedBy, com.eloraam.redpower.machine.EntityPlayerFake#canHarvestBlock, com.eloraam.redpower.machine.ItemRenderMachine#renderItem
Files: mods/redpower_2/src/machine/machine_blocks.rs

## Beads import

- Beads id: `rk-af9o`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T06:00:23Z
- Close reason (Beads): Implemented base BlockMachine registration, facing orientations, metadata dispatch, harvest properties, ItemRenderMachine, and EntityPlayerFake simulation helper; configured Deployer, Block Breaker, T

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T06:00:21Z · Jonathan Stoff] [domain-builder] done: Implemented base BlockMachine registration, facing orientations, metadata dispatch, harvest properties, ItemRenderMachine, and EntityPlayerFake simulation helper; configured Deployer, Block Breaker, Transposer, Igniter, Relay and verified contract-check clean.
Files: mods/redpower_2/src/machine/machine_blocks.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/asset-map.json, mods/redpower_2/content/assets/redpower_2/models/block/deployer.json, mods/redpower_2/content/assets/redpower_2/models/block/block_breaker.json, mods/redpower_2/content/assets/redpower_2/models/block/transposer.json, mods/redpower_2/content/assets/redpower_2/models/block/igniter.json, mods/redpower_2/content/assets/redpower_2/models/block/relay.json, mods/redpower_2/content/assets/redpower_2/blockstates/deployer.json, mods/redpower_2/content/assets/redpower_2/blockstates/block_breaker.json, mods/redpower_2/content/assets/redpower_2/blockstates/transposer.json, mods/redpower_2/content/assets/redpower_2/blockstates/igniter.json, mods/redpower_2/content/assets/redpower_2/blockstates/relay.json, mods/redpower_2/content/data/redpower_2/horizon/block/deployer.json, mods/redpower_2/content/data/redpower_2/horizon/block/block_breaker.json, mods/redpower_2/content/data/redpower_2/horizon/block/transposer.json, mods/redpower_2/content/data/redpower_2/horizon/block/igniter.json, mods/redpower_2/content/data/redpower_2/horizon/block/relay.json, planning/mods/redpower_2.contract/deployer.json, planning/mods/redpower_2.contract/block_breaker.json, planning/mods/redpower_2.contract/transposer.json, planning/mods/redpower_2.contract/igniter.json, planning/mods/redpower_2.contract/relay.json
STATUS: done
FILES:
- mods/redpower_2/src/machine/machine_blocks.rs
- mods/redpower_2/src/machine/mod.rs
- mods/redpower_2/asset-map.json
- mods/redpower_2/content/assets/redpower_2/models/block/{deployer,block_breaker,transposer,igniter,relay}.json
- mods/redpower_2/content/assets/redpower_2/blockstates/{deployer,block_breaker,transposer,igniter,relay}.json
- mods/redpower_2/content/data/redpower_2/horizon/block/{deployer,block_breaker,transposer,igniter,relay}.json
- planning/mods/redpower_2.contract/{deployer,block_breaker,transposer,igniter,relay}.json
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml machine (155 tests pass), contract-check clean for all 5 blocks
DEFERRED: none
CORE-NEEDS: none
