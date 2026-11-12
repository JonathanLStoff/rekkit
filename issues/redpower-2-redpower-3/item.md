---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part04-lamp-power-logic-and-tests
- mod
- mod:redpower_2
- needs-split
- needs:rekkit_core
- path:mods/redpower_2
- role:test-smith
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-fd2o
epic: redpower-2-3-2
blocked_by: ['@redpower-2-redpower-2', '@redpower-2-standard']
closed: 2026-11-12
---

# redpower_2: RedPower Lamp: redstone power state toggle and verification scenarios

Implement power state checking and write contract integration scenarios for all 16 lamp colors. When neighbor blocks update, check RedPowerLib.isPowered (handling vanilla redstone, weak power, strong power, and bundled cables). Standard lamp toggles off -> lit on power and lit -> off on signal loss; inverted lamp toggles lit -> off on power and off -> lit on signal loss. Preserve metadata across state swaps. Done: Level 1 placement/hitbox and Level 4 redstone toggle scenarios pass for both standard and inverted lamps.

Size: ~70k tokens.
Covers: lamp, lamp_lit, lamp_inverted, inverted_lamp, lamp_inverted_lit
Read first: spec 3. Machines & Devices, 6.1 Blocks (121 entries)
Java: BlockLamp#checkPowerState, BlockLamp#a
Files: mods/redpower_2/tests/lamp_tests.rs, mods/redpower_2/src/lighting/tests.rs

## Beads import

- Beads id: `rk-fd2o`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-08T03:26:14Z
- Close reason (Beads): RedPower Lamp power state toggling and contract scenarios implemented and tested.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T13:12:10Z · unknown] [test-smith] partial: Token budget reached during analysis of BlockLamp Java and contracts; lamp tests and scenarios remain to be written

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T13:14:27Z · unknown] [test-smith] partial: Verified lamp contracts, e2e baseline, and test setup; reached token budget before creating lamp tests

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T13:14:28Z · unknown] [test-smith] partial: Verified lamp contracts, e2e baseline, and test setup; reached token budget before creating lamp tests

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T13:14:30Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-07T13:14:31Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-08T03:25:58Z · unknown] [test-smith] complete: Implemented power state checking and verification scenarios for all 16 lamp colors. Normal lamps toggle off -> lit on power and lit -> off on signal loss; inverted lamps toggle lit -> off on power and off -> lit on signal loss. Preserves metadata across state swaps; multipass Pass 0 base render and Pass 1 bloom halo verified in tests.
