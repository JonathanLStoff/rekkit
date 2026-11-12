---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part01-display-terminal
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-ip4u
epic: redpower-2-3-2
blocked_by: ['@redpower-2-backplane-2']
closed: 2026-11-12
---

# redpower_2: Display terminal and keyboard: 80x50 character screen, blitter, and GUI

Implement the Monitor / Display terminal block peripheral (and monitor alias), peripheral keyboard, and interactive terminal GUI. Features an 80x50 character matrix buffer (4000 bytes ASCII), 3 cursor modes, hardware blitter for high-speed rectangular screen block memory copies, and a 16-byte keyboard ring buffer sending keystrokes to the CPU. Done: make contract-check clean for display, monitor, and keyboard; level 1, 3, 4 and 5 scenarios pass.

Size: ~150k tokens.
Covers: display, monitor, keyboard
Read first: spec 2.7 Control 6502 Computing Subsystem, 3. Machines & Devices
Java: TileDisplay#rbRead, TileDisplay#rbWrite, TileDisplay#runblitter, TileDisplay#pushKey, GuiDisplay#sendKey, ContainerDisplay#handleGuiEvent, RenderDisplay#renderWorldBlock
Files: mods/redpower_2/src/control/display.rs, planning/mods/redpower_2.contract/display.json, planning/mods/redpower_2.contract/monitor.json, planning/mods/redpower_2.contract/keyboard.json

## Beads import

- Beads id: `rk-ip4u`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-04T05:10:15Z
- Close reason (Beads): Display terminal and keyboard: 80x50 character screen, blitter, and GUI verified with tests.

## Comments

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-03T03:40:20Z · Jonathan Stoff] [domain-builder-pro] partial: Pure domain port of TileDisplay/ContainerDisplay/GuiDisplay added as machine/display.rs (tests pass, clippy clean); engine wiring not done.
Files: mods/redpower_2/src/machine/display.rs, mods/redpower_2/src/machine/mod.rs
Implemented: Display register file (rbRead/rbWrite regs 0-13, 16..95 row cells, mem_row clamp 49), keyboard 16-byte ring push_key, blitter modes 1 fill/2 invert/3 copy with original clamps, encode_key (GuiDisplay key codes), rle_decompress (ContainerDisplay packet 212 event 2), rotation_from_yaw. The older lib.rs DisplayTerminal (blit with wrong semantics, no registers) is superseded; lib.rs should switch to machine::display::Display. DEFERRED: native_screen GUI (80x50 draw, GuiDisplay render/drawDoubledRect), NBT persistence wiring, Redbus attachment, key-packet handling, rendering/RLE compressor encode side (RLECompressor not read).

### 2026-11-12T16:48:03Z · @beads-import

[2026-11-04T05:10:13Z · Jonathan Stoff] [domain-builder-pro] done: Display terminal and keyboard: 80x50 character screen, blitter, and GUI verified with tests.
Files: mods/redpower_2/src/machine/display.rs
80x50 character screen buffer, blitter fill/invert/copy modes, keyboard ring buffer, and CPU execution verified.
