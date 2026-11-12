---
created: 2026-11-12
updated: 2026-11-12
type: bug
status: fixed
priority: normal
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- step:2
- tier:Hard
- wave:1
beads_id: rk-qst.2.1
epic: redpower-2-2-2
closed: 2026-11-12
---

# redpower_2: horizon-registry.toml line 13 is not valid TOML

'$V = I \cdot R$' inside a basic string: \c is not a TOML escape, so make check-mods fails. Use a literal string or drop the LaTeX.

## Beads import

- Beads id: `rk-qst.2.1`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T17:41:16Z
- Closed (Beads): 2026-11-01T19:43:23Z
- Close reason (Beads): Fix invalid LaTeX escape in mods/redpower_2/horizon-registry.toml to plain text (V = I * R)

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-01T19:43:21Z · Jonathan Stoff] [scaffolder] done: Fix invalid LaTeX escape in mods/redpower_2/horizon-registry.toml to plain text (V = I * R)
Files: mods/redpower_2/horizon-registry.toml
STATUS: done
FILES:
- mods/redpower_2/horizon-registry.toml
CHECK:
- make check-mods MOD=redpower_2 (pass)
- cargo test --manifest-path mods/redpower_2/Cargo.toml (pass)
