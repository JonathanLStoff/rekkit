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
- step:1
- tier:Hard
- wave:1
beads_id: rk-qst.1.3
epic: redpower-2-1
closed: 2026-11-12
---

# rekkit_core: add BlutricityConductor dynamic RLC/inductive solver

Iterative conductor relaxation with current conservation and reactive load tracking requires a dedicated solver module.

## Beads import

- Beads id: `rk-qst.1.3`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-01T19:55:45Z
- Closed (Beads): 2026-11-01T19:57:41Z
- Close reason (Beads): Not core: implemented in redpower_2 domain as the blutricity_grid subsystem

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-01T19:57:40Z · Jonathan Stoff] Conductor: the Blutricity conductor solver is redpower_2-specific (power_converters only converts at the node level via BlutricityNode), so it belongs in the redpower_2 domain crate under the 'blutricity_grid' subsystem feature (step 3).
