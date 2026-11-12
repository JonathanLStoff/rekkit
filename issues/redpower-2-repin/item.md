---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- horizon-pin
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.215
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: repin to Horizon ed102e05 for creative tabs

Content mods' creative tabs (data/<id>/horizon/creative_tab/) need Horizon 74ff545e6 (2026-10-30) or later; redpower_2 pins 38a97ea, which refuses them. Set native/horizon-native.json 'horizon' and native/mixins/redpower_2/horizon-mixin.toml horizon_revision to ed102e0592eded5585c610c205488f771f52ac40 (what mods/example_mod pins). Keep hooks = 40: hook points are only ever added (Horizon is at API 153). Build with make e2e-mod MOD=redpower_2 (it may need --prepare once for the new game resources); fix any mixin that no longer applies. Then the content-author adds the creative tab(s).

## Beads import

- Beads id: `rk-qst.3.215`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T22:49:21Z
- Closed (Beads): 2026-11-01T22:56:56Z
- Close reason (Beads): Repinned redpower_2 native manifest and mixin configuration to Horizon ed102e0592eded5585c610c205488f771f52ac40.

## Comments

### 2026-11-12T16:48:05Z · @beads-import

[2026-11-01T22:56:55Z · Jonathan Stoff] [hook-engineer] done: Repinned redpower_2 native manifest and mixin configuration to Horizon ed102e0592eded5585c610c205488f771f52ac40.
Files: mods/redpower_2/native/horizon-native.json, mods/redpower_2/native/mixins/redpower_2/horizon-mixin.toml
STATUS: done
HOOKS: API 40 -> horizon_client::game::native_hooks::Game<'_>::native_mods -> lists redpower_2 in options
GAPS: none
CHECK: make check-mods MOD=redpower_2 pass
