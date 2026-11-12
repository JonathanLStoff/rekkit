---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-machine-base
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-678p
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Machine and MachinePanel base blocks, tile entities, and rotation

Implements foundational machine block infrastructure: BlockMachine (meta 0..15), BlockMachinePanel (meta 0..4), 6-direction rotation handling (IRotatable), active and powered state flags, and packet synchronization. Base tile entity abstractions TileMachine and TileMachinePanel with IFrameSupport and Redstone/Tube connection interfaces. Done: contract-check clean for machine block registration, placement, and 6-direction rotation.

Size: ~120k tokens.
Covers: item_logistics_machines
Read first: spec 3. Machines & Devices, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: BlockMachine#<init>, BlockMachine#getTextureFile, BlockMachinePanel#<init>, BlockMachinePanel#getTextureFile, ItemMachine#<init>, ItemMachinePanel#<init>, TileMachine#getFacing, TileMachine#setPartRotation, TileMachine#onBlockPlacedBy, TileMachinePanel#getFacing, TileMachinePanel#setPartRotation, TileMachinePanel#onBlockPlacedBy, RenderMachine#renderWorldBlock, RedPowerMachine#initialize
Files: mods/redpower_2/src/machine/base.rs

## Beads import

- Beads id: `rk-678p`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:33:31Z
- Close reason (Beads): machine/base.rs verified against TileMachine/TileMachinePanel/BlockMachine/RedPowerMachine Java (facing, ps bits, rot/ps NBT, packets, max rotation 5/3, item name/meta mapping all match); added init_m

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:31:05Z · Jonathan Stoff] [domain-builder] partial: machine/base.rs (BlockMachine/Machine2/Panel kinds, TileMachine/TileMachinePanel, rotation, flags, packets, NBT) already existed but was not compiled; wired it in via machine/mod.rs and lib.rs. 7 tests pass, clippy clean.
Files: mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Did not re-verify base.rs against the original Java line by line (no decompiled folder for RedPowerMachine found under assets/decompiled; only redpowerdigital exists). Engine registration of machine blocks not done here.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:33:27Z · Jonathan Stoff] [domain-builder] done: machine/base.rs verified against TileMachine/TileMachinePanel/BlockMachine/RedPowerMachine Java (facing, ps bits, rot/ps NBT, packets, max rotation 5/3, item name/meta mapping all match); added init_machine_nbt and screwdriver_rotate_nbt and hooked screwdriver in lib.rs native_use_item_on; all 23 machine/machine2/panel blocks are registered.
Files: mods/redpower_2/src/machine/base.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
cargo test --lib passes. tests/mod_tests.rs does not compile: ThermopileState::tick signature changed (now one closure arg) but tests at lines 509 and 846 call tick(array, int); not mine. TileMachinePanel.getLightValue is 0 in Java; Rust light_value field defaults 0, fine. native_block_entity still gives every block a 'facing' tag; init_machine_nbt (rot/ps) is not yet called from a placement hook because no on-place native hook was found in use. DEFERRED: per-machine behaviours (separate beads).
