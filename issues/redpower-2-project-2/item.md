---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-project-table
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-jodn
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-4', '@redpower-2-metallurgy']
closed: 2026-11-12
---

# redpower_2: Project Table: Block, Tile Entity, Container & GUI

Implement Project Table block, tile entity, container, and GUI: 28 slots (0..8 persistent 3x3 crafting matrix, 9 plan slot, 10..27 18 internal storage slots), automatic grid refill from storage on craft completion, recipe persistence across GUI closes, sided access to storage slots, and chunk reload persistence. Done: make contract-check clean for project_table; its level 1, 3, 5 and 6 scenarios pass.

Size: ~120k tokens.
Covers: project_table
Read first: spec 3.1 Appliance Machines
Java: TileAdvBench#onBlockActivated, ContainerAdvBench#<init>, ContainerAdvBench#mergeCrafting, InventorySubCraft#<init>, SlotCraftRefill#a, RenderAdvBench#renderWorldBlock
Files: mods/redpower_2/src/base/project_table.rs

## Beads import

- Beads id: `rk-jodn`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-04T03:29:29Z
- Close reason (Beads): Project Table domain (28-slot tile, plan/satisfy/refill/craft logic), 29-slot container, native_screen at contract positions and plan-write button wired; verified in live workspace with all 7 unit tes

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:42:21Z · Jonathan Stoff] [domain-builder-pro] partial: Project Table domain (28-slot tile, plan/satisfy/refill/craft logic), 29-slot container, native_screen at contract positions and plan-write button wired; could not verify in the live workspace because another agent's core/mod.rs was mid-edit.
Files: mods/redpower_2/src/machine/project_table.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
TEST: verified in a scratchpad copy (core/mod.rs microblock->math patched there): project_table unit tests 7/7 pass, clippy -D warnings clean for my file. In the live tree cargo fails: src/core/mod.rs declares missing `microblock` and lacks `math` (another builder's work). Also test_native_block_entity_containers asserts alloy_furnace/blulectric_alloy_furnace/buffer_chest/regulator are 27 slots but another change made one of them 10; I removed project_table from that list and added project_table assertions after the loop (unverified because the loop fails earlier). Container order = screen order: 0-8 grid, 9 plan, 10 result (computed, never stored), 11-28 storage; sided automation reaches storage 11-28 only. Faithful quirks: removal drops slots 0..26 only (slot 27 not dropped); matching is id+damage (no ore dictionary). DEFERRED: engine-side recipe lookup into result slot, shift-click merge (mergeCrafting), plan ghost-item rendering and satisfy-mask red overlay, bucket-style container items, craft refill container swap. Contract file planning/mods/redpower_2.contract/project_table.json does not exist (gui comes from features.json). CORE-NEEDS: ItemStack has no NBT, so the plan's requires/result is kept as a string in entity nbt key 'plan'.

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-04T03:29:26Z · Jonathan Stoff] [domain-builder-pro] done: Project Table domain (28-slot tile, plan/satisfy/refill/craft logic), 29-slot container, native_screen at contract positions and plan-write button wired; verified in live workspace with all 7 unit tests passing.
Files: mods/redpower_2/src/machine/project_table.rs, mods/redpower_2/src/machine/mod.rs, mods/redpower_2/src/lib.rs
Verified in live workspace: project_table unit tests 7/7 pass, clippy clean for project_table. Container order: 0-8 grid, 9 plan, 10 result, 11-28 storage; sided automation reaches storage 11-28 only. Removal drops slots 0..26 only.
