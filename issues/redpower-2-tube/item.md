---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part15-frame-tubes
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-coht
epic: redpower-2-3-2
blocked_by: ['@redpower-2-support-2', '@redpower-2-frame-2']
closed: 2026-11-12
---

# redpower_2: Tube Frame and Redstone Tube Frame structural carriers

Implement Tube Frame (TileFrameTube) and Redstone Tube Frame (TileFrameRedstoneTube), combining support frame structure and adherence with embedded pneumatic routing and redstone signal transmission across moving assemblies. Done: make contract-check MOD=redpower_2 clean for frame_tube and frame_redstone_tube; level 1, 3, 4, 5 scenarios pass.

Size: ~120k tokens.
Covers: frame_tube, frame_redstone_tube
Read first: spec 2.4 Pneumatic Tubes & Item Logistics, 2.6 Frame Automation & Structural Movement
Java: TileFrameTube#rebuildSticky, TileFrameTube#tryAddCover, TileFrameRedstoneTube#rebuildSticky, TileFrameRedstoneTube#tryAddCover, RenderFrameTube#renderWorldBlock, RenderFrameRedstoneTube#renderWorldBlock
Files: mods/redpower_2/src/machine/frame_tubes.rs, planning/mods/redpower_2.contract/frame_tube.json, planning/mods/redpower_2.contract/frame_redstone_tube.json

## Beads import

- Beads id: `rk-coht`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:33:52Z
- Close reason (Beads): Tube Frame and Redstone Tube Frame structural carriers registered and wired into frame and tube systems.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:33:50Z · Jonathan Stoff] [domain-builder] done: Tube Frame and Redstone Tube Frame structural carriers registered and wired into frame and tube systems.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/frame_tube.json, mods/redpower_2/content/data/redpower_2/horizon/block/frame_redstone_tube.json
Frame tube and redstone tube frame block definitions and 3D models validated.
