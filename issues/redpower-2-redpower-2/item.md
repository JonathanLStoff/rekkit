---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part04-lamp-multipass-render
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
beads_id: rk-dj4p
epic: redpower-2-3-2
blocked_by: ['@redpower-2-redpower']
closed: 2026-11-12
---

# redpower_2: RedPower Lamp: multipass block and inventory rendering with colored halo bloom

Implement custom multipass rendering for full-block lamps matching RenderLamp.java. Pass 0: renders full 1x1x1 cube with sprite index 16+md for unlit states (directional face shading) and 32+md for lit states (full 1.0F emissive brightness). Pass 1: for lit states, renders additive alpha bloom halo box expanded by 0.05 blocks ([-0.05, -0.05, -0.05, 1.05, 1.05, 1.05]) tinted with lightColors[md] at alpha 0.5. Implement inventory block rendering with proper lit/unlit textures. Done: contract-check model verification clean; pass 0 and pass 1 halo rendering verified in tests.

Size: ~70k tokens.
Covers: lamp, lamp_lit, lamp_inverted, lamp_inverted_lit
Read first: spec 3. Machines & Devices, 6.1 Blocks (121 entries)
Java: RenderLamp#renderWorldBlock, RenderLamp#renderInvBlock
Files: mods/redpower_2/src/lighting/render.rs, mods/redpower_2/src/lighting/halo.rs

## Beads import

- Beads id: `rk-dj4p`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-07T13:09:18Z
- Close reason (Beads): Split part04-lamp-multipass-render into part04-lamp-pass0-render and part04-lamp-halo-bloom

## Comments

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-07T12:41:15Z · unknown] [domain-builder] partial: Investigated lamp multipass rendering spec and verified baseline test passing; handed off before implementing render.rs and halo.rs due to budget limit

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-07T12:44:17Z · unknown] [domain-builder] partial: Decompiled RedPowerLighting and extracted exact RenderLamp parameters; ready for lighting module implementation

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-07T12:44:19Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-07T12:57:29Z · unknown] [domain-builder] partial: Completed exact extraction of RenderLamp.java and BlockLamp.java multipass, halo bloom, and inventory rendering parameters; budget reached before writing source files

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-07T12:57:31Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:00Z · @beads-import

[2026-11-07T13:09:16Z · unknown] [spec-writer] done: Split part04-lamp-multipass-render into part04-lamp-pass0-render and part04-lamp-halo-bloom
