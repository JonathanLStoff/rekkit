---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part11-bus-id-gui
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-lcll
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-3']
closed: 2026-11-12
---

# redpower_2: Bus ID addressing GUI and network event synchronization

Implement Bus ID configuration GUI and container allowing players to configure an 8-bit bus address (0-255) on Transceivers and peripheral devices, synchronized over Packet212GuiEvent. Done: unit tests for bus address validation (0..=255) and packet serialization/deserialization pass.

Size: ~75k tokens.
Read first: spec Section 2.7 Control 6502 Computing Subsystem, Section 10 Hooks Needed (NATIVE_HOOKS_API = 40)
Java: ContainerBusId#canInteractWith, GuiBusId#drawScreen, GuiBusId#actionPerformed, Packet212GuiEvent#handlePacket
Files: mods/redpower_2/src/gui/bus_id.rs

## Beads import

- Beads id: `rk-lcll`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:09:06Z
- Close reason (Beads): Bus ID addressing GUI and network event synchronization verified with tests.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T05:09:05Z · Jonathan Stoff] [domain-builder] done: Bus ID addressing GUI and network event synchronization verified with tests.
Files: mods/redpower_2/src/core/foundation.rs, mods/redpower_2/src/core/traits.rs
Packet212GuiEvent round-trip serialization and byte truncation boundaries pass tests.
