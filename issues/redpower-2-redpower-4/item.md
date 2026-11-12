---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-redpower-lib
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-6lex
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-5']
closed: 2026-11-12
---

# redpower_2: RedPower Redstone Rules & Wire Network Search

Implement RedPowerLib and PowerLib redstone connectivity logic: wire attachment surface rules, corner connection queries across block boundaries, local-to-connection side bitmask transformations, and recursive wire cluster signal strength propagation. Done: unit tests validating wire connection bitmasks, 6-face mapping, and signal level propagation pass.

Size: ~110k tokens.
Read first: spec 2.2 Wiring System
Java: RedPowerLib#canSupportWire, RedPowerLib#isSideNormal, RedPowerLib#getConnections, RedPowerLib#getExtConnections, RedPowerLib#mapConToLocal, RedPowerLib#mapLocalToCon, RedPowerLib#updateCurrent, PowerLib#cutBits
Files: mods/redpower_2/src/core/redpower_lib.rs

## Beads import

- Beads id: `rk-6lex`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:36:50Z
- Close reason (Beads): Ported RedPowerLib redstone rules (mapping, powering, connections, external corner connections) as pure wiring/rules.rs over a RuleWorld trait; 13 tests pass, clippy clean.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:36:47Z · Jonathan Stoff] [domain-builder-pro] done: Ported RedPowerLib redstone rules (mapping, powering, connections, external corner connections) as pure wiring/rules.rs over a RuleWorld trait; 13 tests pass, clippy clean.
Files: mods/redpower_2/src/wiring/rules.rs, mods/redpower_2/src/wiring/mod.rs
The signal search (updateCurrent etc.) already existed in wiring/network.rs and was not touched. can_support_wire keeps the decompiled quirk comparing piston metadata to the block x coordinate (i != md). RuleWorld has no engine implementation yet; an engine adapter must provide it. Connectable.powering_mask is Vec indexed by channel. Not wired into network.rs (WireWorld Tile model still its own).
