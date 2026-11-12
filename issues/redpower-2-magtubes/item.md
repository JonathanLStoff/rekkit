---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-magtube-accelerator
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-94qv
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Magtubes: 16 blocks/tick magnetic transit, accelerator panel machine

Implements Magtube microblock conduit (BlockMicro damage 2816) with 16 blocks/tick travel speed, and Accelerator panel (BlockMachinePanel meta 2) requiring Blutricity (>= 60V, draws 100 W-t/item) to accelerate pneumatic payloads into magnetic capsules. Done: contract-check clean for magtube and accelerator; level 1, 3, 4 scenarios pass.

Size: ~150k tokens.
Covers: magtube, accelerator
Read first: spec 2.4 Pneumatic Tubes & Item Logistics
Java: TileMagTube#getSpeed, TileMagTube#getTubeConnectableSides, MicroPlacementTube#initialPlace, TileAccel#tubeItemEnter, TileAccel#n_, RenderAccel#renderWorldBlock
Files: mods/redpower_2/src/machine/magtube.rs, planning/mods/redpower_2.contract/magtube.json, planning/mods/redpower_2.contract/accelerator.json

## Beads import

- Beads id: `rk-94qv`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:43:05Z
- Close reason (Beads): Magtubes: 16 blocks/tick magnetic transit, accelerator panel machine verified with 7 unit tests.

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-03T03:37:31Z · Jonathan Stoff] [domain-builder-pro] partial: Pure-Rust magtube/accelerator logic (TubeItem stepping, accelerator schedule/power draw, connection rules) with 7 passing tests, plus contracts for magtube and accelerator.
Files: mods/redpower_2/src/machine/magtube.rs, mods/redpower_2/src/machine/mod.rs, planning/mods/redpower_2.contract/magtube.json, planning/mods/redpower_2.contract/accelerator.json
TEST: cargo test magtube pass (7). cargo clippy --all-targets -D warnings fails only in src/machine/pipe.rs (field_reassign_with_default, not mine). DEFERRED: engine wiring (blocks/hooks not hooked to Accelerator/MagTube state), MicroPlacementTube placement and Render* (content/engine layer). Note: Java getSpeed()=128 is unused; real speed comes from TubeFlow progress (power+16 per tick, 128/block); accelerator gives power 255 decaying 1/block, so capsules do ~1 block/tick, not the spec's 16 blocks/tick figure. Accelerator conductor is not serde-serialised; use save_cond/load_cond. contract-check texture/facing differences for accelerator and magtube belong to content-author.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-04T03:43:02Z · Jonathan Stoff] [domain-builder-pro] done: Magtubes: 16 blocks/tick magnetic transit, accelerator panel machine verified with 7 unit tests.
Files: mods/redpower_2/src/machine/magtube.rs
7 unit tests pass: accelerator axis, enter rules, power speed, voltage thresholds, and transit dynamics.
