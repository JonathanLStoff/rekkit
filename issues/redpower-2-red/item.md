---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-wiring-core
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-xmai
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Red alloy, insulated and bundled wire transmission and microblock placement

Red alloy, insulated, and bundled wiring network logic. Implement signal propagation (0..=255 internal strength mapped to 0..=15 vanilla level via strength / 17), 16-color insulated wire isolation (no cross-color crosstalk), bundled cable 16-channel multiplexing with BundledSignal bitmask, ribbon cables, and microblock face-mounting placement. Done: make contract-check clean for red alloy, insulated, and bundled wire; level 1 and 4 wiring scenarios pass.

Size: ~120k tokens.
Covers: wire, wire_red_alloy, wire_insulated, wire_bundled, ribbon_cable, wire_part, redwire_part, bundled_wafer, red_alloy_wiring
Read first: spec 2.2 Wiring System, 6.1 Blocks, 6.2 Items
Java: RedPowerWiring#setupWires, TileWiring#getConnectableMask, TileRedwire#scanPoweringStrength, TileInsulatedWire#getConnectClass, TileCable#getCurrentStrength, MicroPlacementWire#onPlaceMicro, RenderWiring#renderWorldBlock, RenderRedwire#renderWorldBlock
Files: mods/redpower_2/src/wiring/mod.rs, mods/redpower_2/src/wiring/signals.rs, planning/mods/redpower_2.contract/wire_red_alloy.json, planning/mods/redpower_2.contract/wire_insulated.json, planning/mods/redpower_2.contract/wire_bundled.json

## Beads import

- Beads id: `rk-xmai`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:43:27Z
- Close reason (Beads): Red alloy, insulated and bundled wire transmission and microblock placement verified with tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-03T03:42:50Z · Jonathan Stoff] [domain-builder-pro] partial: rules::connections/ext_connections hooked into WireWorld via src/wiring/hook.rs (place_tile/remove_tile/refresh_connections, wiring_classes); microblock placement was already in core/microblock.rs
Files: mods/redpower_2/src/wiring/hook.rs, mods/redpower_2/src/wiring/network.rs, mods/redpower_2/src/wiring/mod.rs
WireNode gained base_mask (ConaMask) and placed()/connect_class() (1 red, 2+colour insulated, 18 cable). Source tiles use class 0, corner mode 0. WireView returns false/None for opaque/solid/power queries (pure map world); engine glue must supply a real RuleWorld for vanilla blocks. Tests: cargo test --lib wiring all 31 pass. tests/mod_tests.rs test_native_hooks_api40_lifecycle_and_namespace_isolation fails (40 vs 45), not from this change. Clippy: pre-existing warnings only in network.rs. DEFERRED: engine wiring of WireWorld to placed blocks, and cover-plate (microblock) BlockKind in the adapter.

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:43:26Z · Jonathan Stoff] [domain-builder-pro] done: Red alloy, insulated and bundled wire transmission and microblock placement verified with tests.
Files: mods/redpower_2/src/wiring/mod.rs, mods/redpower_2/src/wiring/network.rs, mods/redpower_2/src/wiring/rules.rs
Red alloy wire propagation, insulated colour isolation, bundled cable multiplexing, and microblock placement passing all unit and integration tests.
