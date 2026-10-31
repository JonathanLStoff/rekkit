# Rekkit agent rules

Rekkit ports Java Minecraft mods (Tekkit Classic 1.2.5, Direwolf20 1.4.7, Agrarian Skies 2) to Rust mods for the Horizon engine. Every agent, in any tool (agy, Claude Code, Codex, Cursor…), follows this file.

## Start of every session

Do these in order, without waiting to be told.

1. **Check the setup.** Run `./setup.sh --check` and read its summary.
   - Exit `0`: ready. Go to step 2.
   - Exit `3`: run `./setup.sh` (it does every `·` line itself), then read the summary again.
   - Exit `1`: a `✗` line failed. If the fix is a command you are allowed to run, run it and re-run `./setup.sh`. If it needs an install the user must approve (`sudo`, Homebrew, rustup, a JDK), tell the user the exact command from the `✗` line.
   - Exit `2`: the `!` lines need a person. **Tell the user, word for word, what each `!` line says to download or do.** Usually that is: install the official Minecraft launcher, sign in, play version 26.3 once, then run `./setup.sh` again. Then **keep working**: everything except the live game test (step 7 of the lifecycle) works without it.
2. **Pick the work.** In `planning/fleet-board.md` take the `active` row, else the first `queued` row whose Deps are all `done`. The order is: `rekkit_core`, then the mods in `planning/priority.md` order, then the other Tekkit mods, then Direwolf20, then Agrarian Skies 2. If the board and `priority.md` disagree, `priority.md` wins: reorder the board.
3. **Do it.** With the agy fleet: the conductor runs `.agents/fleet-plan.md` §4. As a single agent: play every role yourself, in the order of `.agents/fleet-plan.md` §4, following `planning/workflow.md`'s 10 steps and the skills in `.agents/skills/`.
4. **Record it.** Update the board row after every step. Without live resources, a passing `make e2e-mod MOD=<id> DRY_RUN=1` marks step 7 `◐`; it does not block steps 8–9.
5. **Before you stop.** Leave `cargo test` and `make check-mods MOD=<id>` passing, update the board, and tell the user what is ready for them to push, sign and release.

## Rules

- One mod at a time. Never start a second board row while one is `active`.
- Template: `mods/example_mod`. Copy its shape; do not invent new layouts.
- Shared code lives in `crates/rekkit_core` (not under `mods/`). Use it instead of redefining energy, stacks, tanks, grids, side config, multiblocks or frequency keys.
- Never put `Cargo.toml` or `.rs` files under `mods/<id>/content/`.
- Native bundles (`native/mixins/<id>/horizon-mixin.toml`) take only `id`, `version`, `priority` under `[mod]` and need at least one `[[injection]]`; `make check-mods` enforces what `horizon-mixins` enforces.
- Never edit `horizon/` (git submodule). Log engine gaps in `planning/horizon-gaps.md`.
- Every native hook checks its own namespace (`<id>:`) before acting.
- Only edit paths you own (`.agents/fleet-plan.md` §3). Report anything else to the conductor.
- Done = `cargo test` for the crate, `make check-mods MOD=<id>` and `make e2e-mod MOD=<id>` all pass.
- The Makefile picks Python 3.11+ itself; outside make use `$(scripts/fleet/python.sh)`.
- Never `git push`, sign or publish releases. A human does that.
- Versions move in lockstep across `Cargo.toml`, `content/horizon-mod.toml`, `native/horizon-native.json` and `native/mixins/<id>/horizon-mixin.toml`.

## What only a person can do

| Thing | Why |
|---|---|
| Own Minecraft: Java Edition and play 26.3 once in the official launcher | Horizon builds its game resources from the player's own copy and never downloads it |
| Approve system installs (`sudo`, Homebrew, rustup, JDK 25) | They change the user's machine |
| `git push`, `minisign`, GitHub Releases | Outward-facing |
| A CurseForge API key (optional) | Without one, Agrarian Skies 2 uses the public 2.0.6 download |
