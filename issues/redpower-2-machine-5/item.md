---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-machine-lib
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-t93l
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-5']
closed: 2026-11-12
---

# redpower_2: Machine Inventory Access & FilterMap Logistics

Implement MachineLib automated inventory handling: sided inventory access extraction and insertion, item stack equality and damage comparison, item ejection, and FilterMap pattern matching and counting algorithms for logistics sorting machines. Done: unit tests for sided inventory filtering, item matching, and stack collection pass.

Size: ~90k tokens.
Read first: spec 2.4 Pneumatic Tubes & Item Logistics, 3. Machines & Devices
Java: MachineLib#getInventory, MachineLib#addToInventoryCore, MachineLib#makeFilterMap, MachineLib#genMatchCounts, MachineLib#matchOneStack, MachineLib#matchAnyStack, MachineLib#collectOneStack
Files: mods/redpower_2/src/core/machine_lib.rs

## Beads import

- Beads id: `rk-t93l`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:39:12Z
- Close reason (Beads): Ported MachineLib (sided slot ranges, add_to_inventory, compare_item, FilterMap with match counts/one/any/col/all, collect_one_stack, empty_inventory) as pure Rust with 8 unit tests.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:39:06Z · Jonathan Stoff] [domain-builder] done: Ported MachineLib (sided slot ranges, add_to_inventory, compare_item, FilterMap with match counts/one/any/col/all, collect_one_stack, empty_inventory) as pure Rust with 8 unit tests.
Files: mods/redpower_2/src/core/machine_lib.rs, mods/redpower_2/src/core/mod.rs
Used the newer redpowercore variant. Operates on &[Option<ItemStack>] slices; ItemCatalog trait supplies has_subtypes/max_damage for compare_item. Not ported (engine glue, not pure): getInventory, ejectItem, handleItem, addToRandomInventory. cargo test machine_lib passes (8); clippy clean for this file.
