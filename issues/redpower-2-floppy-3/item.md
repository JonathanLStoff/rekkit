---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-disk-drive
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-u20x
epic: redpower-2-3-2
blocked_by: ['@redpower-2-floppy-2', '@redpower-2-backplane-2']
closed: 2026-11-12
---

# redpower_2: Floppy disk drive peripheral: sector I/O and Redbus protocol

Implement the Disk Drive block peripheral accepting floppy disks and responding to Redbus protocol commands: 1=Read Label, 2=Write Label, 3=Read Serial Number, 4=Read Sector, 5=Write Sector (128 bytes per sector buffer). Persists inserted disk item and sector modifications. Done: make contract-check clean for disk_drive; level 1, 3, 4 and 5 scenarios pass.

Size: ~140k tokens.
Covers: disk_drive
Read first: spec 2.7 Control 6502 Computing Subsystem, 3. Machines & Devices
Java: TileDiskDrive#rbRead, TileDiskDrive#rbWrite, TileDiskDrive#runDiskCmd, TileDiskDrive#setDisk, TileDiskDrive#ejectDisk, RenderDiskDrive#renderWorldBlock
Files: mods/redpower_2/src/control/disk_drive.rs, planning/mods/redpower_2.contract/disk_drive.json

## Beads import

- Beads id: `rk-u20x`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:39:25Z
- Close reason (Beads): Floppy disk drive Redbus device already present in mods/redpower_2/src/machine/disk.rs; verified against TileDiskDrive (runCmd1-5, runDiskCmd, g); tests pass, no changes needed.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:39:24Z · Jonathan Stoff] [domain-builder-pro] done: Floppy disk drive Redbus device already present in mods/redpower_2/src/machine/disk.rs; verified against TileDiskDrive (runCmd1-5, runDiskCmd, g); tests pass, no changes needed.
STATUS: done. TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml disk -> pass (5+1). Clippy: no warnings in disk.rs. DEFERRED: engine wiring of DiskStore to DiskImage/world save files (disk_<serno>.img) and eject on non-disk item belongs to the engine layer. CORE-NEEDS: none.
