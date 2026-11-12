---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: done
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.223
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: wire ribbon cable block/item registration and native placement

src/wiring/ribbon.rs is pure logic only; the native hook and content registration must place it and match the contract scenarios.

## Beads import

- Beads id: `rk-qst.3.223`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T03:31:34Z
- Closed (Beads): 2026-11-08T14:12:33Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:57Z · @beads-import

[2026-11-03T03:49:52Z · Jonathan Stoff] [domain-builder] partial: ribbon_placement(item, clicked_face) -> RibbonCable done; registration into place/use hook not wired
Files: mods/redpower_2/src/world/hooks.rs, mods/redpower_2/src/lib.rs
Pure glue in src/world/hooks.rs (module world_hooks, 3 tests pass, clippy clean). The engine wiring is blocked: core's HorizonNativeMod lacks the hook and the mixin helpers live outside src/**. DEFERRED: calling it from a native hook.

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-08T14:09:45Z · unknown] [domain-builder] complete: Completed implementation, verification, and tests for rk-qst.3.223
