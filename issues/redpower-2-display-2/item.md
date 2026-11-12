---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-control-display-gui
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-hqv9
epic: redpower-2-3-2
blocked_by: ['@redpower-2-display', '@redpower-2-display-3']
closed: 2026-11-12
---

# redpower_2: Display terminal GUI, RLE screen compression, and Keyboard

Implement ContainerDisplay, GuiDisplay, and RLE screen compression (RLECompressor) for the terminal screen. Renders 80x50 monochrome character glyphs, double-sized rendering rects, handles blinking hardware cursor, and forwards user keystrokes into the keyboard buffer. Includes Keyboard block contract. Done: make contract-check clean for keyboard; GUI renders terminal text from tile display buffer and transmits keystrokes reliably.

Size: ~80k tokens.
Covers: keyboard
Read first: spec 2.7 Control 6502 Computing Subsystem
Java: com.eloraam.redpower.control.ContainerDisplay#getDisplayRLE, com.eloraam.redpower.control.ContainerDisplay#decompress, com.eloraam.redpower.control.GuiDisplay#sendKey, com.eloraam.redpower.control.GuiDisplay#a, com.eloraam.redpower.control.GuiDisplay#drawDoubledRect
Files: mods/redpower_2/src/control/display_gui.rs

## Beads import

- Beads id: `rk-hqv9`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-07T17:39:24Z
- Close reason (Beads): Implemented ContainerDisplay, GuiDisplay with doubled rect rendering, hardware cursor blinking, RLE synchronization, and Keyboard block integration.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-07T17:39:22Z · unknown] [content-author] done: Implemented ContainerDisplay, GuiDisplay with doubled rect rendering, hardware cursor blinking, RLE synchronization, and Keyboard block integration.
