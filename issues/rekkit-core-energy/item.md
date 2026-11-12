---
created: 2026-11-12
updated: 2026-11-12
type: feature
status: open
priority: normal
labels:
- core
- for:redpower_2
- mod
- mod:rekkit_core
- path:crates/rekkit_core
- tier:Hard
- wave:core
beads_id: rk-p2g.12
epic: rekkit-core-3
---

# rekkit_core: energy::BlutricityConductor (dynamic RLC) missing

redpower_2 spec section 11 lists 'new: rekkit_core::energy::BlutricityConductor (full dynamic inductive/RLC circuit simulation matching BluePowerConductor)' as a core gap, and rekkit_core::energy only has the simple BlutricityNode (V/Q/C, Ohm's-law equalisation). The engine currently lives in mods/redpower_2/src/core/conductor.rs; it should move to rekkit_core once the core row is worked so any other Blutricity mod can share it (crates/rekkit_core/src is outside this bead's row).

## Beads import

- Beads id: `rk-p2g.12`
- Beads status: `open`
- Beads priority: `2`
- Beads type: `feature`
- Created (Beads): 2026-11-03T22:05:02Z
