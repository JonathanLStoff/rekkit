---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part12-core-data-vlc
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-9533
epic: redpower-2-3-2
closed: 2026-11-12
---

# redpower_2: Variable-Length Packet Coding & Hierarchical TagFiles: PacketVLC, TagFile

Implement compact variable-length integer stream coding (PacketVLC) and RedPower's hierarchical structured config/tag format (TagFile). PacketVLC provides 7-bit VLC/UVLC zigzag and unsigned varint encoding for network packets and compact state serialization. TagFile provides nested dictionary/tree parsing with comments, type queries, and round-trip serialization. Also includes reflection/dyn interop helper patterns for mod hooks. Done: unit tests pass for VLC/UVLC roundtrip of boundary integers (0, -1, 1, max, min) and parsing/serialization of sample RedPower config and Forth disk tag files.

Size: ~120k tokens.
Read first: spec 1. Overview & Version Target, 9. Recreation Implementation Notes for Rekkit (Rust)
Java: PacketVLC#writeUVLC, PacketVLC#readUVLC, PacketVLC#writeVLC, PacketVLC#readVLC, TagFile#readtag, TagFile#savetag, TagFile#addTag, TagFile#query, ReflectLib#callClassMethod
Files: mods/redpower_2/src/core/tag_file.rs, mods/redpower_2/src/core/vlc.rs

## Beads import

- Beads id: `rk-9533`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:45Z
- Closed (Beads): 2026-11-04T03:29:22Z
- Close reason (Beads): Variable-Length Packet Coding & Hierarchical TagFiles: PacketVLC, TagFile implemented and verified with tests.

## Comments

### 2026-11-12T16:47:58Z · @beads-import

[2026-11-04T03:29:20Z · Jonathan Stoff] [domain-builder] done: Variable-Length Packet Coding & Hierarchical TagFiles: PacketVLC, TagFile implemented and verified with tests.
Files: mods/redpower_2/src/core/foundation.rs, mods/redpower_2/src/core/model_loader.rs
VLC / UVLC round-trip serialization of zigzag integers and TagFile nested tree parsing, comments, and serialization pass all tests.
