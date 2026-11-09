# Rekkit agent rules

Rekkit ports Java Minecraft mods (Tekkit Classic 1.2.5, Direwolf20 1.4.7, Agrarian Skies 2) to Rust mods for the Horizon engine. Every agent, in any tool, follows this file; `CLAUDE.md` and `GEMINI.md` only point here.

- **A role subagent** (started on a bead): your prompt, your role file and `make fleet-task BEAD=<id> ROLE=<role>` (your brief) are all you need. Do not read `.agents/start.md`, the fleet plan or the board.
- **The main session** (a person started you): do `.agents/start.md` now, without waiting to be told. Its rules for the main session and the conductor are part of these.

## Rules

- **Spec first, then the Java, a piece at a time.** The spec (`planning/mods/<id>.md`) is an outline; read the sections you need (`make spec-show`), then the original's Java a method at a time (`make java-show DIR=<decompiled> MEMBER=<Class#method>`; never a whole class: the hooks refuse more than 200 lines). Write our own Rust; do not paste Java.
- **Read narrowly; every token you read is paid again on every later turn.** No folder sweeps (recursive listings, `find`, `tree`), no `git status`, `git log -p` or bare `git diff`, no grep over the Java or the spec, no reading `scripts/` or the `Makefile` to learn a command (`make help`, `<script> --help`), nothing read twice. A role subagent's hooks refuse these.
- **Few turns.** A role subagent wraps up at 60 turns or 500k tokens billed (each call's fresh input, cached reads at a tenth): it appends its progress to `target/fleet/progress/<bead>.md` after each piece it finishes, then writes a partial report, and a fresh agent continues.
- Only edit paths you own (your role file says which). Report anything else.
- Shared code lives in `crates/rekkit_core`: use it instead of redefining energy, stacks, tanks, grids, side config, multiblocks or frequency keys. Copy `mods/example_mod`'s shapes; do not invent layouts.
- Never put `Cargo.toml` or `.rs` files under `mods/<id>/content/`. Never edit `horizon/` (a gap is a follow_up `horizon: ...`). Every native hook checks its own namespace (`<id>:`) before acting.
- **The port is the original.** Each block's and item's contract (`planning/mods/<id>.contract/<name>.json`, from the Java) gives its shape, facing, face sprites, GUI slots, sprite, stack size, durability and a scenario per attribute; a vanilla or swapped texture, one texture on every side, a cube where the original is thin, a stock grid GUI or a machine that does nothing is a bug, never done.
- Never `git push`, `gh release`, `minisign`, change the submodule, `git reset --hard`, `--force` or `rm -rf`, and never edit a version by hand (`make version-sync` does it). A human does those.
- Role subagents never run `bd`: put everything for Beads in your report, `target/fleet/inbox/<bead>.json`.
