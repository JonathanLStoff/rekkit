---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part07-machine-registration
- mod
- mod:redpower_2
- needs-split
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-xxvh
epic: redpower-2-3-2
blocked_by: ['@redpower-2-sorting-3', '@redpower-2-retriever', '@redpower-2-regulator', '@redpower-2-frame-4', '@redpower-2-relay', '@redpower-2-transposer']
closed: 2026-11-12
---

# redpower_2: Machine registration: tile entity descriptors and machine packet handlers

Wire tile entity registration and packet descriptors for all part07 machines (FrameMotor, Transposer, Relay, Retriever, Sorter, Regulator) into the machine module framework. Done: all part07 machine tile entities register cleanly and handle tile packets without error.

Size: ~70k tokens.
Read first: spec 3.2 Tube & Logistics Machines, 3.3 Power & Fluid Machines
Java: mod_RedPowerMachine#initialize, mod_RedPowerMachine#init_guihandler
Files: mods/redpower_2/src/machine/mod.rs

## Beads import

- Beads id: `rk-xxvh`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-08T03:21:51Z
- Close reason (Beads): Machine registration and tile packet descriptors implemented and verified.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-04T05:56:47Z · Jonathan Stoff] [domain-builder] partial: Implemented TileMotor and TileTranspose with tile packet descriptors and serialization; remaining: TileRelay, TileRetriever, TileSorter, TileRegulator and registry module wiring.
Files: mods/redpower_2/src/machine/motor.rs, mods/redpower_2/src/machine/transposer.rs
Token budget reached at 200k. Implemented TileMotor and TileTranspose with packet descriptors (Packet 211 subId 7), FrameSupport, Rotatable, TubeConnectable, and NBT serialization with tests. Ready for continuing agent to complete TileRelay, TileRetriever, TileSorter, TileRegulator and centralized registration wiring in machine::registration/machine::mod.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T13:18:21Z · unknown] [domain-builder] partial: Initial inspection of part07 machine registration framework; hit token budget cap.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T13:20:00Z · unknown] [domain-builder] partial: Token budget reached while inspecting machine registration and packet descriptors for part07 machines

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T13:20:03Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-08T03:21:32Z · unknown] [domain-builder] complete: Wired tile entity registration and packet descriptors (write_to_packet, read_from_packet, write_tile_packet, handle_tile_packet, Rotatable, FrameSupport, TubeConnectable) for all part07 machines (FrameMotor, Transposer, Relay, Retriever, Sorter, Regulator, Sortron) into the machine module framework.
