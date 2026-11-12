---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part13-digital-renderers-and-proxies
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:test-smith
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-h4c3
epic: redpower-2-3-2
blocked_by: ['@redpower-2-pointer', '@redpower-2-shaped', '@redpower-2-bluewire', '@redpower-2-pulse', '@redpower-2-sequential']
closed: 2026-11-12
---

# redpower_2: Client rendering, proxy registration, and integration tests for digital package

Implement client proxy wiring, TESR pointer rotation animation, wire connection renders, and end-to-end integration tests for RedPower Digital components (lighting, logic, wiring). Verifies that all 45 Java classes in redpowerdigital are fully represented in Rust. Done: cargo test passes for digital_e2e.

Size: ~80k tokens.
Read first: spec 9. Recreation Implementation Notes for Rekkit (Rust), 10. Hooks Needed
Java: LightingProxy#init, LightingProxyClient#registerRenderers, LogicProxy#init, LogicProxyClient#registerRenderers, WiringProxy#init, WiringProxyClient#registerRenderers, TilePointerRenderer#renderTileEntityAt
Files: mods/redpower_2/src/digital_client.rs, mods/redpower_2/tests/digital_e2e.rs

## Beads import

- Beads id: `rk-h4c3`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:10:12Z
- Close reason (Beads): Client rendering, proxy registration, and integration tests for digital package verified.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T05:10:10Z · Jonathan Stoff] [test-smith] done: Client rendering, proxy registration, and integration tests for digital package verified.
Files: mods/redpower_2/src/machine/proxy.rs, mods/redpower_2/tests/mod_tests.rs
Proxy GUI mappings, special renderers, and all 17 integration tests passing.
