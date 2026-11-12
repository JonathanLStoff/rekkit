---
created: 2026-11-12
updated: 2026-11-12
type: task
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
- story
beads_id: rk-qst.3
epic: redpower-2-3
blocked_by: ['@redpower-2-2-2']
closed: 2026-11-12
---

# redpower_2: 3 Code

## Beads import

- Beads id: `rk-qst.3`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `story`
- Created (Beads): 2026-11-01T17:41:05Z
- Closed (Beads): 2026-11-08T14:38:17Z
- Close reason (Beads): Closed

## Comments

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T17:41:16Z · Jonathan Stoff] Existing domain code (src/lib.rs), tests and 29 item JSONs are kept; after the spec is corrected, step 4 Check reviews them against it and the Java and files what must change. The 42 blocks were not authored yet.

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T22:39:34Z · Jonathan Stoff] Reopened with its 201 item and block features: none of them is in the creative menu. A content mod lists its items in data/<id>/horizon/creative_tab/*.json, which needs Horizon 74ff545e6 (2026-10-30) or later; redpower_2 pins 38a97ea, which has no mod creative tabs. To close each feature: repin the mod to a Horizon with creative tabs, list the item (or the block's item) in a creative tab with a translated title (itemGroup.redpower_2.<tab>), make check-mods MOD=redpower_2 COMPLETE=1 passes, and the harness finds it in the creative menu (step 8 'creative <id>' features).

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-03T03:48:15Z · Jonathan Stoff] Conductor checkpoint 2026-11-02: ~55 tickets run; pure-domain modules landed in src/{core,machine,wiring,world}. Most reports are partial: engine wiring in lib.rs (block entities, GUIs, hooks) deferred and filed as follow-up beads. Contract diffs 1155 -> 1033. Step 8 (live harness) needs Minecraft 26.3 installed by a person (setup.sh --check exit shows it missing); JDK 25 also missing.
