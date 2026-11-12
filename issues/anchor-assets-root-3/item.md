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
- step:6
- tier:Hard
- wave:1
beads_id: rk-qst.6.23
epic: redpower-2-6
closed: 2026-11-12
---

# Anchor assets/ in root .gitignore as /assets/

Line 26 of .gitignore has assets/ without leading slash, causing git to ignore any directory named assets anywhere (such as mods/*/content/assets/).

## Beads import

- Beads id: `rk-qst.6.23`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-01T21:29:57Z
- Closed (Beads): 2026-11-01T21:30:28Z
- Close reason (Beads): Assets sliced/generated and wired; check-mods COMPLETE=1 passes
