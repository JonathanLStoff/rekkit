---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part06-blutricity-tools
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-nry7
epic: redpower-2-3-2
blocked_by: ['@redpower-2-battery-3']
closed: 2026-11-12
---

# redpower_2: Voltmeter diagnostic tool and Sonic Screwdriver chargeable multi-tool

Implements Voltmeter (displays voltage, current, and wattage in chat when right-clicking any Blutricity conductor or machine) and Sonic Screwdriver (rechargeable screwdriver with entity disorient / creeper defuse capabilities, consuming charge from internal battery). Done: contract-check clean for voltmeter and sonic_screwdriver; level 2 scenarios pass.

Size: ~120k tokens.
Covers: voltmeter, sonic_screwdriver
Read first: spec 4.1 Metallurgy & Crafting Tools, 2.5 Blutricity Power Grid
Java: ItemVoltmeter#itemUseShared, ItemVoltmeter#onItemUseFirst, ItemSonicDriver#onItemUseFirst, ItemSonicDriver#a
Files: mods/redpower_2/src/item/voltmeter.rs, mods/redpower_2/src/item/sonic_screwdriver.rs, planning/mods/redpower_2.contract/voltmeter.json, planning/mods/redpower_2.contract/sonic_screwdriver.json

## Beads import

- Beads id: `rk-nry7`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:44Z
- Closed (Beads): 2026-11-04T03:45:28Z
- Close reason (Beads): Voltmeter diagnostic tool and Sonic Screwdriver chargeable multi-tool registered and verified.

## Comments

### 2026-11-12T16:48:01Z · @beads-import

[2026-11-04T03:45:25Z · Jonathan Stoff] [domain-builder] done: Voltmeter diagnostic tool and Sonic Screwdriver chargeable multi-tool registered and verified.
Files: mods/redpower_2/content/data/redpower_2/horizon/item/voltmeter.json, mods/redpower_2/content/data/redpower_2/horizon/item/sonic_screwdriver.json
Item definitions, models, and blutricity/rotation tool interactions verified.
