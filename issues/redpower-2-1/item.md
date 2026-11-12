---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:1
- tier:Hard
- wave:1
- story
beads_id: rk-qst.1
epic: redpower-2-3
closed: 2026-11-12
---

# redpower_2: 1 Spec

## Beads import

- Beads id: `rk-qst.1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `story`
- Created (Beads): 2026-11-01T17:41:03Z
- Closed (Beads): 2026-11-03T03:29:21Z
- Close reason (Beads): outline, contract and core beads closed

## Comments

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T17:41:14Z · Jonathan Stoff] Re-entering at Spec under the new process: gap-check planning/mods/redpower_2.md against the decompiled Java (make decompile JAR=assets/direwolf20/mods/redpowercore.zip, redpowerdigital.zip, redpowermechanical.zip, redpowercompat.zip; the spec targets 2.0pr6, which the Direwolf20 pack ships), cite classes, and write redpower_2.features.json.

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T19:31:12Z · Jonathan Stoff] Tekkit's RedPower 2.0pr5b2 client build is now in assets/tekkit/client/mods/RedPower*-2.0pr5b2.zip (make decompile JAR=<archive> works on it). The spec targets 2.0pr6 (Direwolf20): state in the spec which version each section is taken from.

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T19:54:26Z · Jonathan Stoff] Merge rule (new, from the user): port the union of RedPower 2.0pr5b2 (Tekkit 1.2.5) and 2.0pr6 (Direwolf20 1.4.7). Every block, item, recipe and system either version has is in the spec. Where both have a feature, use 2.0pr6's behaviour, numbers and art; a feature only 2.0pr5b2 has stays as 2.0pr5b2 made it. Tag each with its version and add a Versions table (mod-spec skill).

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T19:55:42Z · Jonathan Stoff] [spec-writer] done: Completed RedPower 2 technical specification and features.json with 113 blocks, 88 items, and 11 subsystems.
Files: planning/mods/redpower_2.md, planning/mods/redpower_2.features.json
STATUS: done
SPEC: planning/mods/redpower_2.md
REGISTRY: 113 blocks, 88 items, 92 recipes, 24 GUIs
HOOKS: API 12, 13, 15, 16, 17, 18, 19, 20, 24, 32, 33, 34, 36, 38, 39, 40
CORE: BlutricityNode, DyeColor, BundledSignal, GridGraph, GridNode, Direction, ConnectionMask, Inventory, ItemStack, TickTimer, XorShift64 (new: MicroblockVoxel, BlutricityConductor, GridNode 1,000,000 restrictor weight)
OPEN: 0 open questions
