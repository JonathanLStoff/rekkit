---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-core-math-traits
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-8opc
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Core Math, 3D Orientation & Common Mod Traits

Implement 3D rotation matrix and quaternion orientation math supporting all 24 block orientations, along with fundamental RedPower 2 traits and network packet abstractions: IConnectable, ICoverable, IMultipart, IRotatable, IPaintable, IFrameSupport, IPipeConnectable, ITubeConnectable, and description/GUI packet handlers. Done: unit tests for orientation matrix/quaternion transformations and trait contracts pass cleanly.

Size: ~80k tokens.
Read first: spec 2. Core Systems & Mechanics, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: MathLib#orientMatrix, MathLib#orientQuat, CoreLib#rotToSide, CoreLib#compareItemStack, Packet211TileDesc#<init>, Packet212GuiEvent#<init>, PacketVLC#<init>
Files: mods/redpower_2/src/core/math.rs, mods/redpower_2/src/core/traits.rs

## Beads import

- Beads id: `rk-8opc`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:32:23Z
- Close reason (Beads): Ported Quat/Matrix3/MathLib 24 orientations, rotToSide, and core traits + VLC packets (211/212) with tests.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:32:21Z · Jonathan Stoff] [domain-builder] done: Ported Quat/Matrix3/MathLib 24 orientations, rotToSide, and core traits + VLC packets (211/212) with tests.
Files: mods/redpower_2/src/core/math.rs, mods/redpower_2/src/core/traits.rs, mods/redpower_2/src/core/mod.rs
Decompiled RedPowerCore-2.0pr5b2 (assets/decompiled, not committed) since it was missing. compare_item_stack takes has_subtypes flag (Java asks the item). TubeConnectable is generic over the item type (TubeItem not yet ported); FluidBuffer getter omitted from PipeConnectable.
