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
- step:3
- tier:Hard
- wave:1
beads_id: rk-qst.3.312
epic: redpower-2-3-2
closed: 2026-11-12
---

# spec 2.5: I_flow formula says I_next, the Java uses the pre-update current

planning/mods/redpower_2.md 2.5 writes I_flow = I_next + V*0.5, but BluePowerConductor.iterate (redpowercore 2.0pr6, lines 148-153) does I += V*getCondParallel() on the OLD currents[s] value, then applies that. With a live inductor current the two differ by (V - I*R)*0.07 (60 A vs 66.986 A in the unit test). core/conductor.rs follows the Java; the spec should be corrected so a later porter does not 'fix' the engine to the wrong formula.

## Beads import

- Beads id: `rk-qst.3.312`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `bug`
- Created (Beads): 2026-11-03T22:04:58Z
- Closed (Beads): 2026-11-08T13:56:44Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-08T13:56:34Z · unknown] [spec-writer] complete: Updated planning/mods/redpower_2.md section 2.5 Blutricity dynamic transfer formula to document using pre-update current I for parallel flow matching Java BluePowerConductor.iterate.
