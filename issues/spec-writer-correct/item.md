---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:4
- tier:Hard
- wave:1
beads_id: rk-qst.4.6
epic: redpower-2-4
closed: 2026-11-12
---

# spec-writer: correct Sequencer trigger in spec §2.3 (Java TileLogicPointer cycles continuously by timer interval, not per input pulse)

TileLogicPointer.java:157-172 shows sequencerUpdate() cycles continuously by world time and interval rather than input redstone pulses.

## Beads import

- Beads id: `rk-qst.4.6`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:01:11Z
- Closed (Beads): 2026-11-01T21:03:48Z
- Close reason (Beads): Clarified Sequencer cycles continuously by configured interval in §2.3 rather than per input pulse

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T21:03:47Z · Jonathan Stoff] [spec-writer] done: Clarified Sequencer cycles continuously by configured interval in §2.3 rather than per input pulse
Files: planning/mods/redpower_2.md
STATUS: done
SPEC: planning/mods/redpower_2.md
REGISTRY: 113 blocks, 88 items, 136 recipes, 14 GUIs
HOOKS: API 11, 14, 15, 17, 19, 20, 24, 32, 33, 34, 36, 38, 39, 40
CORE: rekkit_core::energy::BlutricityNode, rekkit_core::frequency::{DyeColor, BundledSignal}, rekkit_core::grid::{BlockPos, GridGraph, GridNode, UnionFind}, rekkit_core::side::{Direction, ConnectionMask}, rekkit_core::inventory::{Inventory, ItemStack}, rekkit_core::hooks::{HorizonNativeMod, ModBlockEntity, ModContainer, NativeModInfo, NativeScreen}, rekkit_core::util::{TickTimer, XorShift64}; new: rekkit_core::microblock::MicroblockVoxel, rekkit_core::energy::BlutricityConductor, rekkit_core::grid::GridNode::restrictor_weight
OPEN: 0
