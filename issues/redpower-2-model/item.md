---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part03-tag-model-loader
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-b0wh
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-7']
closed: 2026-11-12
---

# redpower_2: Model loader, TagFile serialization and texture vertex primitives

Implement RedPower custom TagFile format parser/writer and 3D sub-voxel model reader (RenderModel). Parses stream tokens, nested tags, comments, vertex lists, UV coordinate maps, and model grouping hierarchies. Done: unit tests successfully parse TagFile structures and load sub-voxel mesh definitions from RenderModel data.

Size: ~100k tokens.
Read first: spec 2.1 Microblocks & Sub-Voxel System, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: TagFile#readtag, TagFile#collapsedtag, TagFile#savetag, RenderModel#loadModel, RenderModel#scale, TexVertex#setUV
Files: mods/redpower_2/src/core/model_loader.rs

## Beads import

- Beads id: `rk-b0wh`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T03:34:54Z
- Close reason (Beads): Added core/model_loader.rs: TagFile (parse/save/dotted add/get/remove, comments), StreamTokenizer subset, TexVertex, RenderModel load_model/scale; 5 tests pass.

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:34:52Z · Jonathan Stoff] [domain-builder] done: Added core/model_loader.rs: TagFile (parse/save/dotted add/get/remove, comments), StreamTokenizer subset, TexVertex, RenderModel load_model/scale; 5 tests pass.
Files: mods/redpower_2/src/core/model_loader.rs, mods/redpower_2/src/core/mod.rs
TagFile::Query not included (belongs to part12 TagFile ticket). loadModel takes text, resource lookup left to caller. cargo test model_loader passes.
