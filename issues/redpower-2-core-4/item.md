---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-multipart-core
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-1ei6
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-5']
closed: 2026-11-12
---

# redpower_2: Core Multipart, Extended Blocks & Appliance Base

Implement BlockExtended, BlockMultipart, BlockCoverable, and TileAppliance base abstractions. Handles sub-tile metadata dispatch, raytraced multipart bounding boxes, player partial harvesting, and frame movement support (IFrameSupport). Done: unit tests for multi-bounding-box raytracing, sub-tile metadata mapping, and block break behaviors pass.

Size: ~100k tokens.
Read first: spec 2.1 Microblocks & Sub-Voxel System, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: BlockExtended#addTileEntityMapping, BlockMultipart#computeCollidingBoxes, BlockMultipart#removeBlockByPlayer, BlockMultipart#getBlockDropped, BlockCoverable#isBlockSolidOnSide, TileAppliance#onBlockPlacedBy, TileAppliance#getFramePacket
Files: mods/redpower_2/src/core/multipart.rs

## Beads import

- Beads id: `rk-1ei6`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:37:19Z
- Close reason (Beads): Extended core/multipart.rs with BlockExtended tile map, BlockMultipart raytrace/colliding boxes/player part removal/drops, BlockCoverable solid-side and explosion resistance, and TileAppliance (placem

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:37:18Z · Jonathan Stoff] [domain-builder-pro] done: Extended core/multipart.rs with BlockExtended tile map, BlockMultipart raytrace/colliding boxes/player part removal/drops, BlockCoverable solid-side and explosion resistance, and TileAppliance (placement, packets, frame support); 14 multipart tests pass.
Files: mods/redpower_2/src/core/multipart.rs
TileMap<K> (addTileEntityMapping/createTileEntity), MultipartTile trait + CoveredView adapter for TileCovered, ray_box/trace_parts (nearest part, first wins ties), colliding_boxes, remove_block_by_player (RemoveOutcome), extended_drops, get_block_dropped, is_block_solid_on_side, explosion_resistance, TileAppliance (rot=floor(yaw*4/360+.5)&3, light 13 when active, sub id 7 packet, frame packet [7,rot,ps], NBT ps/rot). Frame packet byte order assumed subId first (Java writes headout after body). Clippy clean for multipart.rs. cargo test: 154+17 pass.
