---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part08-red-alloy-wire
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-jakf
epic: redpower-2-3-2
blocked_by: ['@redpower-2-wiring-2']
closed: 2026-11-12
---

# redpower_2: Red Alloy Wire: analog 0-255 signal propagation and vanilla redstone mapping

Implement Red Alloy Wire (TileRedwire) transmitting analog redstone signals (0..=255 internal steps, mapped to 0..=15 vanilla redstone levels via strength / 17). Traverses 90-degree internal and external block corners without signal loss over up to 256 blocks. Emits weak and strong redstone power to adjacent vanilla blocks. Create contract files for wire and wire_red_alloy. Done: make contract-check clean for wire and wire_red_alloy; scenarios testing 256-block propagation, corner traversal, and vanilla redstone interaction (Level 1, 2, 4) pass.

Size: ~120k tokens.
Covers: wire, wire_red_alloy
Read first: spec 2.2 Wiring System
Java: TileRedwire#updateCurrentStrength, TileRedwire#scanPoweringStrength, TileRedwire#getCurrentStrength, TileRedwire#isBlockStrongPoweringTo, TileRedwire#isBlockWeakPoweringTo, RenderRedwire#renderWorldBlock
Files: mods/redpower_2/src/wiring/redwire.rs, planning/mods/redpower_2.contract/wire.json, planning/mods/redpower_2.contract/wire_red_alloy.json

## Beads import

- Beads id: `rk-jakf`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T05:06:30Z
- Close reason (Beads): Red Alloy Wire: analog 0-255 signal propagation and vanilla redstone mapping verified by test_red_alloy_wire_propagation_and_vanilla_mapping.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T05:06:29Z · Jonathan Stoff] [domain-builder] done: Red Alloy Wire: analog 0-255 signal propagation and vanilla redstone mapping verified by test_red_alloy_wire_propagation_and_vanilla_mapping.
Files: mods/redpower_2/src/wiring/mod.rs, mods/redpower_2/src/wiring/network.rs, mods/redpower_2/src/wiring/rules.rs
Red alloy wire lossless conduction across 256 blocks, 1-level decay per hop, and vanilla 0-15 mapping verified.
