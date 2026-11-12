---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-disk-drive
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-3e6w
epic: redpower-2-3-2
blocked_by: ['@redpower-2-floppy-2', '@redpower-2-6502', '@redpower-2-floppy-3']
closed: 2026-11-12
---

# redpower_2: Floppy Disk Drive and Floppy Disks: sector I/O and boot disks

Implement the Disk Drive peripheral (BlockPeripheral meta 2) and Floppy Disk items (blank, Forth boot, Forth extra). Floppy disks hold 256 KB formatted into 2048 sectors of 128 bytes each. The Disk Drive maps to Redbus address 2 with sector select, buffer read/write, and status registers (disk present, motor running). Write contract files for disk_drive and disk items. Done: make contract-check clean for disk_drive and disks; level 1 placement, level 2 disk insertion and sector read/write, level 5 persistence.

Size: ~110k tokens.
Covers: disk_drive, blank_disk, floppy_disk, forth_boot_disk, forth_extra_disk
Read first: spec 2.7 Control 6502 Computing Subsystem, 4.3 Utility & Storage Items
Java: com.eloraam.redpower.control.TileDiskDrive#rbRead, com.eloraam.redpower.control.TileDiskDrive#rbWrite, com.eloraam.redpower.control.ItemDisk#onItemUseFirst, com.eloraam.redpower.control.ItemDisk#d, com.eloraam.redpower.control.RenderDiskDrive#renderWorldBlock
Files: mods/redpower_2/src/control/disk.rs

## Beads import

- Beads id: `rk-3e6w`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T05:57:11Z
- Close reason (Beads): Implemented Disk Drive peripheral (Redbus address 2, sector select, databuf, status register) and Floppy Disks (blank, Forth boot, Forth extra) with 256KB capacity and contract verification

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T05:57:10Z · Jonathan Stoff] [domain-builder] done: Implemented Disk Drive peripheral (Redbus address 2, sector select, databuf, status register) and Floppy Disks (blank, Forth boot, Forth extra) with 256KB capacity and contract verification
Files: mods/redpower_2/src/machine/disk.rs, mods/redpower_2/src/control/mod.rs, mods/redpower_2/src/machine/redforth.img, mods/redpower_2/src/machine/redforthxp.img, mods/redpower_2/content/assets/redpower_2/models/block/disk_drive.json, mods/redpower_2/content/assets/redpower_2/blockstates/disk_drive.json, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_front.png, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_top.png, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_bottom.png, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_side.png, mods/redpower_2/asset-map.json, planning/mods/redpower_2.contract/disk_drive.json
STATUS: done
FILES: mods/redpower_2/src/machine/disk.rs, mods/redpower_2/src/control/mod.rs, mods/redpower_2/src/machine/redforth.img, mods/redpower_2/src/machine/redforthxp.img, mods/redpower_2/content/assets/redpower_2/models/block/disk_drive.json, mods/redpower_2/content/assets/redpower_2/blockstates/disk_drive.json, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_front.png, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_top.png, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_bottom.png, mods/redpower_2/content/assets/redpower_2/textures/block/disk_drive_side.png, mods/redpower_2/asset-map.json, planning/mods/redpower_2.contract/disk_drive.json
TEST: cargo test --manifest-path mods/redpower_2/Cargo.toml disk: pass (12 passed; 0 failed); python3 scripts/contract_check.py --mod redpower_2: 0 problems on disk_drive and disk items
DEFERRED: none
CORE-NEEDS: none
