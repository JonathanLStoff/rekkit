# Rekkit agent rules

Rekkit ports Java Minecraft mods (Tekkit Classic 1.2.5, Direwolf20 1.4.7, Agrarian Skies 2) to Rust mods for the Horizon engine. Every agent, in any tool (agy, Claude Code, Codex, Cursor, Gemini CLI, Copilot…), follows this file; `CLAUDE.md` and `GEMINI.md` only point here.

- **A role subagent** (the conductor started you on a bead): your prompt and your role file are all you need. Follow the rules below; do not read `.agents/start.md`, the fleet plan or the board.
- **The main session** (a person started you): do `.agents/start.md` now, without waiting to be told. It ends with you conducting one mod to completion.
- Reference (architecture, the 11 steps, every command): `.agents/reference.md`, or `make help`. Read it only when you need it.

## Rules

- One mod at a time. Never start a second epic while one is in progress. `rekkit_core` is the exception that is never started on its own: shared code a mod needs is filed under it (`make core-need`) and built during that mod's run, so the core grows with the mods.
- **Clean-room.** Only the spec-writer, parity-critic, scout and conductor read the original's Java (`assets/decompiled/` from `make decompile`, never committed; `.java`/`.class` under `assets/`). Whoever writes a mod's code works from `planning/mods/<id>.md` alone. The hooks enforce it both ways: a session that writes under `mods/` or `crates/` (or works as a builder role) cannot read the Java, and a session that has read the Java cannot write there. The art and sound are the original's until replaced; see `.agents/release.md`.
- **Several versions: merge, newer wins.** When a mod ships in more than one pack (RedPower 2.0pr5b2 in Tekkit 1.2.5 and 2.0pr6 in Direwolf20 1.4.7), the port is the union of every version: each block, item, recipe and system any version has. Where versions share a feature, the newest version's behaviour, numbers and art win (both have the alarm: the 1.4.7 alarm). A feature only an older version has stays, as that version made it (1.2.5's microblocks stay even if 1.4.7 dropped them). One epic and one mod per original mod, never one per version (`.agents/skills/mod-spec/SKILL.md`).
- **Checkpoints.** A role subagent appends one line to `target/fleet/progress/<bead>.md` after each piece it finishes (`make fleet-task` says so and shows an interrupted predecessor's work). A conductor taking over runs `make fleet-resume MOD=<id>` (`make fleet-start` prints it) and restarts interrupted subagents rather than their steps.
- Template: `mods/example_mod`. Copy its shape; do not invent new layouts.
- Shared code lives in `crates/rekkit_core` (not under `mods/`). Use it instead of redefining energy, stacks, tanks, grids, side config, multiblocks or frequency keys.
- Never put `Cargo.toml` or `.rs` files under `mods/<id>/content/`.
- Native bundles (`native/mixins/<id>/horizon-mixin.toml`) take only `id`, `version`, `priority` under `[mod]` and need at least one `[[injection]]`; `make check-mods` enforces what `horizon-mixins` enforces.
- Never edit `horizon/` (git submodule). An engine gap is a Beads bug labelled `horizon` (subagents: a follow_up in the report).
- Every native hook checks its own namespace (`<id>:`) before acting.
- Only edit paths you own (your role file says which). Report anything else to the conductor.
- Complete = steps 1–9 closed: `make agent-gate MOD=<id>`, `make check-mods MOD=<id> COMPLETE=1`, parity-critic's Check and Recheck, and the live harness (`make e2e-mod MOD=<id>` and `make e2e-all-mods`: every block placed by `/setblock` and by a player in every variant, no WARN/ERROR beyond `planning/harness-allowed-warnings.txt`, which only a person edits).
- New art only for what the original never had (`make fleet-gaps`): SVG or `.pixels` in `mods/<id>/art/`, built by `make art MOD=<id>` at the original's size and palette.
- The Makefile picks Python 3.11+ itself; outside make use `$(scripts/fleet/python.sh)`.
- Never `git push`, `gh release`, `minisign`, change the submodule, `git reset --hard`, `--force` or `rm -rf`. A human does those (the hooks in agy and Claude Code block them; elsewhere it is on you). Releases are signed and published by `.github/workflows/release-mods.yml` after a human pushes.
- Versions move in lockstep across `Cargo.toml`, `content/horizon-mod.toml`, `native/horizon-native.json` and `native/mixins/<id>/horizon-mixin.toml`. A mod whose `content/`, `native/` or `src/` changed since its last release needs a new version (`python3 scripts/release_plan.py --exclude example_mod`).

## Context budget

Everything you read stays in your context and is paid again on every later step, so read narrowly.

- Decompiled Java: `make java-outline DIR=<decompiled> [GREP=<name>]`, then `make java-show DIR=<decompiled> MEMBER=<Class#method>`; never a whole class (the hooks refuse reads of more than 200 lines of it).
- A spec: `make spec-toc MOD=<id>`, then `make spec-show MOD=<id> SECTION="<heading>"` for the sections your work needs.
- Where things are for a mod: `make fleet-context MOD=<id>` (state, spec, original archives in every pack, decompiled folders, files), not `ls`/`find` sweeps.
- Run commands; do not read scripts to learn what they do (`make help`, `<script> --help`). Do not re-read a file you already read unless it changed. Read files by line range when you need part of one.
- Long command output: keep the end (`2>&1 | tail -40`), or grep for what you need.
- Done means stop: write your report (subagents) or go to the next step (conductor); no recap of what you did.
