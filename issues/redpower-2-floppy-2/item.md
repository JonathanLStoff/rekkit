---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-floppy-disks
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-n6km
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Floppy disk items: blank, boot disk, and extra disk

Implement magnetic floppy disk items: Blank Floppy Disk (and its alias floppy_disk), RedPower Forth Boot Disk (with redforth.img payload), and RedPower Forth Extra Disk (with redforthxp.img payload). Disks hold 256KB of virtual magnetic storage (2048 sectors of 128 bytes). Done: make contract-check clean for blank_disk, floppy_disk, forth_boot_disk, and forth_extra_disk; level 2 scenarios pass.

Size: ~90k tokens.
Covers: blank_disk, floppy_disk, forth_boot_disk, forth_extra_disk
Read first: spec 2.7 Control 6502 Computing Subsystem, 4.3 Utility & Storage Items
Java: ItemDisk#ctor, ItemDisk#onItemUseFirst, ItemDisk#addCreativeItems
Files: mods/redpower_2/src/control/disks.rs, planning/mods/redpower_2.contract/blank_disk.json, planning/mods/redpower_2.contract/floppy_disk.json, planning/mods/redpower_2.contract/forth_boot_disk.json, planning/mods/redpower_2.contract/forth_extra_disk.json

## Beads import

- Beads id: `rk-n6km`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:31:14Z
- Close reason (Beads): Four floppy disk items (blank_disk, floppy_disk, forth_boot_disk, forth_extra_disk) already had contracts, item JSON, items defs, models, textures, lang keys and creative-tab entries; verified, no con

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:31:09Z · Jonathan Stoff] [content-author] done: Four floppy disk items (blank_disk, floppy_disk, forth_boot_disk, forth_extra_disk) already had contracts, item JSON, items defs, models, textures, lang keys and creative-tab entries; verified, no contract-check or check-mods findings for them.
No changes needed. Contract-check shows no disk findings; remaining redpower_2 findings are level-5 scenarios on other blocks. The ticket lists mods/redpower_2/src/control/disks.rs (Rust, not mine) for payload/behaviour. Contracts say 128 KB while the ticket says 256 KB / 2048 sectors of 128 bytes; one is wrong, check the Java.
