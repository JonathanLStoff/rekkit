---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-core-interfaces
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-l19l
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-2']
closed: 2026-11-12
---

# redpower_2: Core framework interfaces: Rotatable, Paintable, Connectable, Multipart

Define trait definitions and base structures for RedPower block/item polymorphism: IRotatable, IPaintable, IConnectable, IWiring, IRedPowerConnectable, IRedPowerWiring, IRedbusConnectable, IMultipart, and IHandleGuiEvent. Implement BlockExtended and ItemExtended base components. Done: trait implementations and unit tests verifying face masking, connection masks, and sub-part indexing pass.

Size: ~95k tokens.
Read first: spec Section 2.1 Microblocks & Sub-Voxel System, Section 2.2 Wiring System, Section 10 Hooks Needed (NATIVE_HOOKS_API = 40)
Java: IBlockHardness#getHardness, IRotatable#getFacing, IPaintable#getColor, IConnectable#getConnectableMask, IWiring#getConnectionMask, IRedPowerConnectable#getConnectClass, IRedPowerWiring#getRedPowerMask, IRedbusConnectable#rbGetAddr, IMultipart#getPartOccupancy, IHandleGuiEvent#handleGuiEvent, IHandlePackets#handlePacket, BlockExtended#getTileEntity, ItemExtended#getUnlocalizedName, ItemParts#addItem, ItemTextured#getIconIndex
Files: mods/redpower_2/src/core/interfaces.rs

## Beads import

- Beads id: `rk-l19l`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:29:34Z
- Close reason (Beads): Core framework interfaces: Connectable, Coverable, Multipart, Rotatable implemented in traits.rs and multipart.rs with unit tests.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:29:33Z · Jonathan Stoff] [domain-builder] done: Core framework interfaces: Connectable, Coverable, Multipart, Rotatable implemented in traits.rs and multipart.rs with unit tests.
Files: mods/redpower_2/src/core/traits.rs, mods/redpower_2/src/core/multipart.rs
Traits and multipart sub-voxel occupancy, harvesting, rotation, and connection masks pass all tests.
