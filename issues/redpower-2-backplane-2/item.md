---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-backplane-ram
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-p48n
epic: redpower-2-3-2
blocked_by: ['@redpower-2-ribbon-2']
closed: 2026-11-12
---

# redpower_2: Backplane bus and 8K RAM expansion module

Implement the 8-bit bus backplane block and the 8K RAM expansion module. Backplanes connect horizontally behind the CPU and peripherals to extend the address/data bus; RAM modules plug into backplane slots to provide 8KB memory expansion per module up to 64KB total. Done: make contract-check clean for backplane and ram_module; level 1, 4 and 5 scenarios pass.

Size: ~120k tokens.
Covers: backplane, ram_module
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: TileBackplane#readBackplane, TileBackplane#writeBackplane, TileRAM#readBackplane, TileRAM#writeBackplane, ItemBackplane#itemUseShared, RenderBackplane#renderWorldBlock
Files: mods/redpower_2/src/control/backplane.rs, planning/mods/redpower_2.contract/backplane.json, planning/mods/redpower_2.contract/ram_module.json

## Beads import

- Beads id: `rk-p48n`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:37:47Z
- Close reason (Beads): Added machine/backplane.rs: Backplane (bare / 8KB RAM), MemoryMap with refreshBackplane chain, 64KB address decode, open bus 255, persistence helpers; 5 tests pass.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:37:43Z · Jonathan Stoff] [domain-builder] done: Added machine/backplane.rs: Backplane (bare / 8KB RAM), MemoryMap with refreshBackplane chain, 64KB address decode, open bus 255, persistence helpers; 5 tests pass.
Files: mods/redpower_2/src/machine/backplane.rs, mods/redpower_2/src/machine/mod.rs
Not wired into Cpu6502 in lib.rs (not my file; it has a flat 64KB ram Vec) and no engine/GUI/content hooks. Cargo test backplane passes.
