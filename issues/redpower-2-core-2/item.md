---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-core-foundation
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-56cj
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Core runtime foundation: MathLib, Matrix3, DiskLib, CraftLib, OreStack

Implement core math library (Matrix3 orientation, vector transforms, Quat math), floppy disk byte stream utilities (DiskLib 128-byte sector formatting), recipe matching structures (OreStack, CraftLib), and packet descriptors (Packet211TileDesc, Packet212GuiEvent). Done: unit tests for vector math, disk sector reading/writing, and ore recipe matching pass.

Size: ~110k tokens.
Read first: spec Section 2.7 Control 6502 Computing Subsystem, Section 9 Recreation Implementation Notes for Rekkit (Rust)
Java: RedPowerCore#preInit, RedPowerCore#load, Config#init, CoreLib#readItemStack, MathLib#orientMatrix, Matrix3#multiply, FractalLib#fillFractal, DiskLib#getDisk, OreStack#matches, CraftLib#addOreRecipe, Packet211TileDesc#handlePacket, Packet212GuiEvent#handlePacket
Files: mods/redpower_2/src/core/foundation.rs

## Beads import

- Beads id: `rk-56cj`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-03T03:32:55Z
- Close reason (Beads): Added core/foundation.rs: Quat, Matrix3, MathLib orient tables (24), DiskLib/DiskImage (2048x128 sectors), CraftLib alloy registry, OreStack/OreDictionary, PacketVLC + Packet211TileDesc/Packet212GuiEv

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:32:51Z · Jonathan Stoff] [domain-builder] done: Added core/foundation.rs: Quat, Matrix3, MathLib orient tables (24), DiskLib/DiskImage (2048x128 sectors), CraftLib alloy registry, OreStack/OreDictionary, PacketVLC + Packet211TileDesc/Packet212GuiEvent encode/decode; 8 tests pass, no clippy warnings in the file.
Files: mods/redpower_2/src/core/foundation.rs, mods/redpower_2/src/core/mod.rs
OreStack class is not in any decompiled archive (assets/decompiled has no OreStack); implemented a generic item-or-ore-name ingredient with an OreDictionary. Config#init, CoreLib#readItemStack, FractalLib#fillFractal, RedPowerCore preInit/load were not ported (engine/config-bound or not present in decompiled core). Matrix3::rotate keeps the original M^T v convention.
