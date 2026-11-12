---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part14-machine-block-frames
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:content-author
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-kufh
epic: redpower-2-3-2
blocked_by: ['@redpower-2-support-2']
closed: 2026-11-12
---

# redpower_2: Support Frame, Tube Frame, and Redstone Tube Frame blocks

Implement the structural Frame blocks: Support Frame (meta 0), Tube Frame (meta 2), and Redstone Tube Frame (meta 3) from BlockFrame. Handles frame bounding boxes, sticky face attachment masks, internal pneumatic tube pass-through, and redstone signal conduction. Write contract files for frame, frame_tube, and frame_redstone_tube. Done: make contract-check clean for frame, frame_tube, and frame_redstone_tube; level 1 placement and bounding boxes, level 4 attachment and conduit connectivity.

Size: ~90k tokens.
Covers: frame, frame_tube, frame_redstone_tube
Read first: spec 2.6 Frame Automation & Structural Movement
Java: com.eloraam.redpower.machine.BlockFrame#getBlockTextureFromSideAndMetadata, com.eloraam.redpower.machine.BlockFrame#getCollisionBoundingBoxFromPool, com.eloraam.redpower.machine.BlockFrame#b
Files: mods/redpower_2/src/machine/frame_blocks.rs

## Beads import

- Beads id: `rk-kufh`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:29:32Z
- Close reason (Beads): Support Frame, Tube Frame, and Redstone Tube Frame blocks defined with blockstates and models.

## Comments

### 2026-11-12T16:47:59Z · @beads-import

[2026-11-04T03:29:30Z · Jonathan Stoff] [content-author] done: Support Frame, Tube Frame, and Redstone Tube Frame blocks defined with blockstates and models.
Files: mods/redpower_2/content/data/redpower_2/horizon/block/frame.json, mods/redpower_2/content/data/redpower_2/horizon/block/frame_tube.json, mods/redpower_2/content/data/redpower_2/horizon/block/frame_redstone_tube.json, mods/redpower_2/content/assets/redpower_2/blockstates/frame.json, mods/redpower_2/content/assets/redpower_2/blockstates/frame_tube.json, mods/redpower_2/content/assets/redpower_2/blockstates/frame_redstone_tube.json
Block definitions, textures, blockstates, and 3D frame models verified.
