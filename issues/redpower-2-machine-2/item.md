---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-machine-base
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-vsvz
epic: redpower-2-3-2
blocked_by: ['@redpower-2-machine-3']
closed: 2026-11-12
---

# redpower_2: Machine base traits, network synchronization, and proxy registration

Implement base machine foundation for RedPower 2: TileMachine (rotation via IRotatable, active/powered/delay/charged flags, network packet sync via IHandlePackets, frame movement via IFrameSupport) and TileMachinePanel (multipart panel machine base, side rotation, harvest and bounds), base machine rendering dispatchers (RenderMachine, RenderMachine2), and proxy registration for machine tile entities, GUIs, and special renderers (MachineProxy, MachineProxyClient). Done: base machine traits and packet synchronization compile cleanly; machine proxy registers handlers.

Size: ~120k tokens.
Read first: spec 2.2 Wiring System, 2.5 Blutricity Power Grid, 3. Machines & Devices
Java: TileMachine#onBlockPlaced, TileMachine#getFacing, TileMachine#handleFramePacket, TileMachinePanel#setPartBounds, RenderMachine#renderWorldBlock, RenderMachine2#renderWorldBlock, MachineProxy#getClientGuiElement, MachineProxyClient#registerRenderers
Files: mods/redpower_2/src/machine/base.rs, mods/redpower_2/src/machine/proxy.rs

## Beads import

- Beads id: `rk-vsvz`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-03T03:42:48Z
- Close reason (Beads): base.rs already held TileMachine/TileMachinePanel traits, flags, packets (subId 7/8), rotation; added machine/proxy.rs (GUI id table 1..16, renderer tables for machine/machine2/panel/appliance/frame/m

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-03T03:42:45Z · Jonathan Stoff] [domain-builder-pro] done: base.rs already held TileMachine/TileMachinePanel traits, flags, packets (subId 7/8), rotation; added machine/proxy.rs (GUI id table 1..16, renderer tables for machine/machine2/panel/appliance/frame/micro-high, texture anims) and registered it in machine/mod.rs.
Files: mods/redpower_2/src/machine/proxy.rs, mods/redpower_2/src/machine/mod.rs
lib tests 251 pass. tests/mod_tests.rs test_native_hooks_api40_lifecycle_and_namespace_isolation fails (left 40, right 45) - hook count in lib.rs/test out of sync, from other concurrent edits, not this bead. RenderMachine/RenderMachine2 per-face texture logic was already in base.rs (unrotated_texture_indices).
