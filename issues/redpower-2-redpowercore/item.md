---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-core-lifecycle
- mod
- mod:redpower_2
- needs-split
- needs:rekkit_core
- path:mods/redpower_2
- role:hook-engineer
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-5xcv
epic: redpower-2-3-2
blocked_by: ['@redpower-2-red-3', '@redpower-2-pneumatic-6', '@redpower-2-sub-4']
closed: 2026-11-12
---

# redpower_2: RedPowerCore module lifecycle, network handling and native hook wiring

Implement RedPowerCore lifecycle registration, subsystem initialization hooks, network packet dispatch, and GUI event routing. Binds native hooks for block entity tick loops, block entity descriptions, and crafting callbacks. Done: mixin compilation clean and native hooks initialize core subsystems on mod load.

Size: ~90k tokens.
Read first: spec 1. Overview & Version Target, 10. Hooks Needed
Java: mod_RedPowerCore#initialize, mod_RedPowerCore#init_guihandler, mod_RedPowerCore#renderWorldBlock, mod_RedPowerCore#takenFromCrafting, mod_RedPowerCore#takenFromFurnace
Files: mods/redpower_2/native/mixins/redpower_2/core_init.rs

## Beads import

- Beads id: `rk-5xcv`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-07T13:09:41Z
- Close reason (Beads): Split part03-core-lifecycle into part03-core-lifecycle-init and part03-core-net-gui

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T12:41:23Z · unknown] [hook-engineer] partial: Turn budget reached; inspected native_hooks.rs entrypoints and hook patterns for core_init.rs

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T12:43:55Z · unknown] [hook-engineer] partial: Investigated native hook wiring, RedPowerCore lifecycle, and mixin structure; next agent writes core_init.rs

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T12:43:59Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T12:57:24Z · unknown] [hook-engineer] partial: Turn budget reached; mapped out RedPowerCore hook points and prepared core_init.rs mixin requirements

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T12:57:27Z · unknown] Two agents could not finish this ticket (or one finished nothing): split it into smaller tickets (a spec-writer in split mode), then close this one.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-07T13:09:39Z · unknown] [spec-writer] done: Split part03-core-lifecycle into part03-core-lifecycle-init and part03-core-net-gui
