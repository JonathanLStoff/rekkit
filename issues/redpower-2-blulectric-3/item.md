---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part10-blulectric-engine-domain
- mod
- mod:redpower_2
- needs-split
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ru9i
epic: redpower-2-3-2
blocked_by: ['@redpower-2-blutricity-3', '@redpower-2-blulectric-5']
closed: 2026-11-12
---

# redpower_2: Blulectric engine: state machine, blutricity consumer node, flywheel MJ conversion, pump cycle and sided power delivery

Implement the Blulectric Engine tile entity and state machine. Orient placement to 6 directions (pitch and yaw towards player). Connects to Blutricity power grid as an endpoint consumer: reaches Charged state when voltage >= 60.0V; checks redstone power on neighbor change for Powered state. When Charged and Powered: enters Active state, draws 50W base blutricity power, advances pump stroke cycle (PumpSpeed accelerates from 16 down to 4 ticks per half-stroke), draws up to 32 MJ/tick (1,000 W/MJ) into 512 MJ flywheel. At half-stroke (PumpTick == PumpSpeed), pushes stored flywheel MJ into adjacent BuildCraft IPowerReceptor / rekkit_core MjBuffer at facing^1. Saves and loads flywheel charge, pump speed, and pump tick via NBT ('flyw', 'spd', 'ptk'). Also registers BuildCraft oil fluid compatibility with RedPower fluid pipeline. Write planning/mods/redpower_2.contract/blulectric_engine.json with L1 placement & hitbox, L3 blutricity-to-flywheel charging, L4 adjacent BuildCraft MJ power delivery, and L5 chunk reload persistence scenarios. Done: contract-check clean for blulectric_engine; level 1, 3, 4, 5 scenarios pass.

Size: ~140k tokens.
Covers: blulectric_engine
Read first: spec 2.5 Blutricity Power Grid, 3.3 Power & Fluid Machines, 6.1 Blocks, 11. Uses from rekkit_core
Java: TileBlueEngine#g, TileBlueEngine#deliverPower, TileBlueEngine#onBlockNeighborChange, TileMachineCompat#getFacing, TileMachineCompat#onBlockPlaced, TileBlueEngine#a, TileBlueEngine#b, RedPowerCompat#initFluids
Files: mods/redpower_2/src/compat/engine.rs, mods/redpower_2/native/mixins/redpower_2/blulectric_engine.rs, planning/mods/redpower_2.contract/blulectric_engine.json

## Beads import

- Beads id: `rk-ru9i`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-07T17:36:10Z
- Close reason (Beads): Split Blulectric engine into domain state machine and native mixin hook tickets

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T13:19:13Z · unknown] [domain-builder-pro] partial: Decompiled redpowercompat, extracted Java specs and formulas for TileBlueEngine, TileMachineCompat, and RedPowerCompat.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T17:23:30Z · unknown] [domain-builder-pro] partial: Completed detailed analysis of Blulectric engine state machine, MJ flywheel delivery, and fluid compatibility; ready for implementation in fresh agent session

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T17:23:36Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T17:36:08Z · unknown] [spec-writer] done: Split Blulectric engine into domain state machine and native mixin hook tickets
