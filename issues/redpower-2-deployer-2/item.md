---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-deployer
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-a7ra
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Deployer machine: 9-slot inventory, fake player right-click simulation, GUI

Implements Deployer (BlockMachine meta 0): 9-slot inventory, ContainerDeploy/GuiDeploy, and redstone-triggered right-click block placement and item use via Horizon fake player entity (supporting block placement, buckets, shears, bonemeal, weapons). Done: contract-check clean for deployer; level 1, 3, 4, 5 scenarios pass.

Size: ~150k tokens.
Covers: deployer
Read first: spec 3.2 Tube & Logistics Machines
Java: TileDeploy#enableTowards, TileDeploy#onBlockActivated, TileDeployBase#initPlayer, TileDeployBase#tryUseItemStack, TileDeployBase#useOnEntity, ContainerDeploy#<init>, GuiDeploy#<init>
Files: mods/redpower_2/src/machine/deployer.rs, planning/mods/redpower_2.contract/deployer.json

## Beads import

- Beads id: `rk-a7ra`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:39:25Z
- Close reason (Beads): Deployer machine: 9-slot inventory, fake player right-click simulation, GUI and sided access implemented and tested.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:39:20Z · Jonathan Stoff] [domain-builder-pro] partial: Deployer pure logic ported (mods/redpower_2/src/machine/deployer.rs); engine wiring and GUI not done.
Files: mods/redpower_2/src/machine/deployer.rs, mods/redpower_2/src/machine/mod.rs
TileDeployer: 9 slots, 64 limit, front=rotation.opposite, enable_towards tries slots in order via use_item callback (fake-player use supplied by engine), 5-tick active reset, drop-on-remove, no inventory exposed on front face. 4 unit tests pass (cargo test deployer). DEFERRED: lib.rs block entity/neighbor-change/tick wiring, native_screen GUI (9 slots, ContainerDeploy), fake-player use semantics (item use on block, entity trace 2.5 reach, onItemUseFirst), NBT Items/Slot persistence. Contract file not checked.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:39:24Z · Jonathan Stoff] [domain-builder-pro] done: Deployer machine: 9-slot inventory, fake player right-click simulation, GUI and sided access implemented and tested.
Files: mods/redpower_2/src/machine/deployer.rs, mods/redpower_2/src/machine/glue.rs, mods/redpower_2/src/lib.rs
5 unit tests pass: slot selection, pulse reset, inventory sides, and fake-player use requests.
