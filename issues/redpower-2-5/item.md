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
- step:5
- tier:Hard
- wave:1
- story
beads_id: rk-qst.5
epic: redpower-2-3
blocked_by: ['@redpower-2-1']
closed: 2026-11-12
---

# redpower_2: 5 Extract assets

## Beads import

- Beads id: `rk-qst.5`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `story`
- Created (Beads): 2026-11-01T17:41:07Z
- Closed (Beads): 2026-11-08T14:57:33Z
- Close reason (Beads): All 150 asset extraction and mapping tickets completed and verified with make contract-check

## Comments

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-01T17:41:16Z · Jonathan Stoff] Tekkit's redpower archive is a server build with no art. Use the Direwolf20 client builds (redpowercore.zip: 7 PNG, redpowerdigital.zip: 9, redpowermechanical.zip: 23). asset-map.json exists with 6 UNVERIFIED sheets; nothing sliced yet.

### 2026-11-12T16:48:13Z · @beads-import

[2026-11-01T19:31:11Z · Jonathan Stoff] Tekkit's own client build is now downloaded (make tekkit): assets/tekkit/client/mods/RedPower{Core,Control,Lighting,Logic,Machine,Wiring,World}-2.0pr5b2.zip, 32 textures in all (pack-info.json art_files). Use the archives of the version the spec targets (it names 2.0pr6 from Direwolf20, with 2.0pr5b2 for 1.2.5); record the source archive in asset-map.json.

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T19:51:22Z · Jonathan Stoff] [asset-smith] partial: raw extraction done; mapping pending spec
Files: assets/extracted/redpower_2/INVENTORY.md
STATUS: partial
TEXTURES: 15 sprite sheets, 23 guis, 2 standalone (DW20 2.0pr6); 12 sprite sheets, 19 guis (Tekkit 2.0pr5b2)
SOUNDS: 0 (RedPower 2 ships no custom sounds; uses vanilla Minecraft sound events)
UNMAPPED: mapping pending final mod spec
GENERATED: none

Archive counts:
Direwolf20 2.0pr6 (63 assets):
- redpowercore.zip: 2 sprite sheets, 4 guis, 1 banner, 0 models, 0 audio, 0 rom, 3 cfg/lang/info
- redpowerdigital.zip: 7 sprite sheets, 2 guis, 0 other, 4 models, 0 audio, 0 rom, 1 info
- redpowermechanical.zip: 5 sprite sheets, 17 guis, 1 vawt texture, 7 models, 0 audio, 3 rom/bin, 1 info
- redpowercompat.zip: 1 sprite sheet, 0 guis, 0 other, 3 models, 0 audio, 0 rom, 1 info

Tekkit 2.0pr5b2 client (41 assets):
- RedPowerControl-2.0pr5b2.zip: 1 sprite sheet, 3 guis, 1 model, 2 rom/bin
- RedPowerCore-2.0pr5b2.zip: 2 sprite sheets, 2 guis, 0 models, 2 cfg/lang
- RedPowerLighting-2.0pr5b2.zip: 1 sprite sheet
- RedPowerLogic-2.0pr5b2.zip: 4 sprite sheets, 2 guis, 2 models
- RedPowerMachine-2.0pr5b2.zip: 1 sprite sheet, 12 guis, 3 models
- RedPowerWiring-2.0pr5b2.zip: 1 sprite sheet
- RedPowerWorld-2.0pr5b2.zip: 2 sprite sheets

### 2026-11-12T16:48:14Z · @beads-import

[2026-11-01T19:54:27Z · Jonathan Stoff] Merge rule (new): take each texture and sound from the version the spec uses for that feature (2.0pr6 when both have it, else 2.0pr5b2 from assets/tekkit/client/mods/), and record the source archive in asset-map.json.
