# Rekkit agent rules

Rekkit ports Java Minecraft mods (Tekkit Classic 1.2.5, Direwolf20 1.4.7, Agrarian Skies 2) to Rust mods for the Horizon engine. Every agent, in any tool (agy, Claude Code, Codex, Cursor, Gemini CLI, Copilot…), follows this file. `CLAUDE.md` and `GEMINI.md` only point here.

## Start of every session

Do these in order, without waiting to be told.

1. **Check the setup.** Run `./setup.sh --check` (Windows: `setup.cmd --check`; any OS: `make doctor`) and read its summary.
   - Exit `0`: ready. Go to step 2.
   - Exit `3`: run `./setup.sh` (it does every `·` line itself), then read the summary again.
   - Exit `1`: a `✗` line failed. If the fix is a command you are allowed to run, run it and re-run `./setup.sh`. If it needs an install the user must approve (`sudo`, Homebrew, winget, rustup, a JDK), tell the user the exact command from the `✗` line.
   - Exit `2`: the `!` lines need a person. **Tell the user, word for word, what each `!` line says to download or do.** Usually that is: install the official Minecraft launcher, sign in, play version 26.3 once, then run `./setup.sh` again. Then **keep working**: everything except the live game test (step 7 of the lifecycle) works without it.
2. **Pick the work.** In `planning/fleet-board.md` take the `active` row, else the first `queued` row whose Deps are all `done`. The order is: `rekkit_core`, then the mods in `planning/priority.md` order, then the other Tekkit mods, then Direwolf20, then Agrarian Skies 2. If the board and `priority.md` disagree, `priority.md` wins: reorder the board.
3. **Do it, to the end.** Run `make fleet-start MOD=<id>` first. Read `.agents/agents/conductor/agent.md` and act as the conductor: it runs the pipeline in `.agents/fleet-plan.md` §4, following `planning/workflow.md`'s 10 steps. Where it says *invoke* a role, start that role as a subagent if your tool can (see *Your tool* below); otherwise do the role yourself, in the same order, by reading `.agents/agents/<role>/agent.md`. Read the skills a role lists (`.agents/skills/<name>/SKILL.md`) before its work.
   **Do not stop until the row is finished.** Finished means release prep is done (steps 9–10 `▶`, waiting for the human to push). Never end your turn to report progress, summarise, or ask whether to continue: keep going to the next step. Wait for subagents you started and merge their work; do not end your turn while they run. The only other reasons to stop: the user tells you to (or ran `make fleet-stop`), or something only a person can fix blocks the row. Then set the row to `blocked` with the reason in Notes, tell the user exactly what they need to do, and stop. In agy and Claude Code the stop hook holds you to this.
4. **Record it.** Update the board row after every step. Without live resources, a passing `make e2e-mod MOD=<id> DRY_RUN=1` marks step 7 `◐`; it does not block steps 8–9.
5. **Before you stop.** Run `make agent-gate` (cargo test, check-mods and the harness dry run for every mod and crate you changed) and fix what it reports. Update the board, and tell the user what is ready for them to push, sign and release.

## Your tool

`.agents/` is the one source for roles, skills and hooks. Edit it there, never in a tool's copy, then run `make agents` to regenerate the copies (`make agents-check` reports drift; `./setup.sh` does both).

| Tool | Reads | Roles (subagents) | Skills | Guards and stop gate |
|---|---|---|---|---|
| agy (Antigravity) | `AGENTS.md` | `.agents/agents/` (`/agents` → conductor) | `.agents/skills/` | `.agents/hooks.json` |
| Claude Code | `CLAUDE.md` → this file | `.claude/agents/` (generated); `/conductor` starts a run | `.claude/skills/` (generated) | `.claude/settings.json` hooks (generated) |
| Codex, Cursor, Copilot, Gemini CLI (`GEMINI.md`), others | `AGENTS.md` | play each role yourself from `.agents/agents/<role>/agent.md`, or use your tool's subagents with that file as the prompt | read `.agents/skills/<name>/SKILL.md` by path | follow *Rules* yourself; `make agent-gate` before you stop |

Model tiers in the roster: `pro` is your tool's strongest model (agy: Gemini 4 Argon, Claude Code: Opus), `flash` its fast one (agy: 3.8 Flash, Claude Code: Sonnet).

Headless, one mod per run, any tool: `make fleet-next AGENT=agy|claude|codex` (or `AGENT_CMD="<cli and flags>"` for another tool); see `.agents/fleet-plan.md` §9.

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
- Never `git push`, `gh release`, `minisign`, change the submodule, `git reset --hard`, `--force` or `rm -rf`. A human does those (the hooks in agy and Claude Code block them; elsewhere it is on you). Releases are signed and published by `.github/workflows/release-mods.yml` after a human pushes.
- Versions move in lockstep across `Cargo.toml`, `content/horizon-mod.toml`, `native/horizon-native.json` and `native/mixins/<id>/horizon-mixin.toml`. A mod whose `content/`, `native/` or `src/` changed since its last release needs a new version (`python3 scripts/release_plan.py --exclude example_mod`).

## What only a person can do

| Thing | Why |
|---|---|
| Own Minecraft: Java Edition and play 26.3 once in the official launcher | Horizon builds its game resources from the player's own copy and never downloads it |
| Approve system installs (`sudo`, Homebrew, winget, rustup, JDK 25) | They change the user's machine |
| `git push`, approving a release in the `mod-release` environment, listing a mod at playhorizon.dev/mods/submit | Outward-facing |
| A CurseForge API key (optional) | Without one, Agrarian Skies 2 uses the public 2.0.6 download |

## Project reference

### Architecture: Horizon dual-artifact mods (no WebAssembly)

Every mod in `mods/<id>/` is three parts:

1. **Content mod (`.hmod`, API `0.2`, `capabilities = ["registries"]`)**: data and assets only (`content/horizon-mod.toml`, `content/data/`, `content/assets/`): blocks (with optional item `components`), items (with optional `crafting_remainder`), custom `mob_effect` types, recipes (including custom machine recipe types), loot tables, tags, models and textures. No code: `cargo-horizon-mod` rejects a `content/` with a `Cargo.toml`.
2. **Native mod (`.hnmod`, `NATIVE_HOOKS_API = 40`)**: a source-level [horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle (`native/horizon-native.json` + `native/mixins/<id>/horizon-mixin.toml` + `.rs` helpers) hooking into `horizon/crates/horizon-client/src/game/native_hooks.rs` (hooks 8–40: block entities, containers, machine ticks, custom machine recipe types, redstone signals, cable/pipe shape updates, player/inventory ticks, `horizon:mod_net` packets, custom GUI screens, server commands).
3. **Pure Rust domain crate (`Cargo.toml` + `src/lib.rs`)**: standalone, fully unit-testable domain logic, energy/EMC math, state machines and `HorizonNativeMod` implementations.

The template is [`mods/example_mod/`](mods/example_mod): `Cargo.toml`, `horizon-registry.toml` (registry page: `name`, `summary`, `description`, `authors`, `license`), `src/lib.rs`, `tests/mod_tests.rs`, `content/horizon-mod.toml`, `native/horizon-native.json` (`schema = 1`, `game = "26.3"`, pinned 40-hex `horizon` and `mixins` revisions, `hooks = 40`), `native/mixins/example_mod/horizon-mixin.toml`.

### The 10-step port lifecycle ([`planning/workflow.md`](planning/workflow.md))

1. **Spec**: `planning/mods/<id>.md` (blocks, items, tile entities, GUI layout, formulas, energy, EMC, recipes, hazards).
2. **Scaffold** the three parts above under `mods/<id>/`.
3. **Test**: `cargo test --manifest-path mods/<id>/Cargo.toml` (`make test-mods` for all).
4. **Extract** legacy assets: the `extract-jar-assets` skill or `make extract-jar JAR=assets/.../<mod>.jar`.
5. **Add assets** under `mods/<id>/content/assets/<id>/` (textures, sounds, lang, models, blockstates, items).
6. **Validate**: `make test-mods` and `make check-mods MOD=<id>`.
7. **Integration** ([`planning/harness.md`](planning/harness.md)): `make integration-test` = `make e2e-mod MOD=<id>` (boots Horizon with the mod, places every block, runs the 7 action stages) + `make e2e-all-mods` (all mods loaded together). `DRY_RUN=1` for pre-flight.
8. **Version**: bump in lockstep (see *Rules*).
9. **Commit** in conventional format (`feat(<id>): …`); a human pushes.
10. **Release**: on push to `main`, `.github/workflows/release-mods.yml` packs, signs and publishes the GitHub Release `<id>-<version>`; Horizon's registry reads it.

### Commands

| Command | What it does |
|---|---|
| `./setup.sh` / `setup.cmd` / `make setup` | Set up a fresh clone end to end; `--check` / `make doctor` reports only |
| `make test-mods` | `cargo test` for every crate under `mods/` |
| `make check-mods [MOD=<id>]` | Validate `.hmod` content and `.hnmod` native manifests and assets against Horizon's rules |
| `make pack-mods [MOD=<id>]` | Build deterministic `.hmod` and `.hnmod` in `mods/<id>/target/horizon-mod/` |
| `make e2e-setup` / `e2e-mod` / `e2e-all-mods` / `integration-test` | The harness: build Horizon with the mods' mixins, then boot it and run the action plans |
| `make agent-gate` | What the stop gate runs, for any agent: tests, checks and dry run of everything you changed |
| `make agents` / `make agents-check` | Regenerate / check each tool's copy of `.agents/` |
| `make fleet-next [AGENT=…] [MOD=<id>]` | One headless conductor run on one board row |
| `make tekkit` / `direwolf20` / `agrarian` / `all` | Download the modpacks into `assets/` (Agrarian Skies 2 falls back to the public 2.0.6 without `CURSEFORGE_API_KEY`) |
| `make extract-jar JAR=<path> [EXTRACT_OUT=<dir>]` / `make extract-assets [DIR=<dir>]` | Extract textures, sprites, audio, models and scripts from mod archives |

Planning: [`planning/workflow.md`](planning/workflow.md), [`planning/priority.md`](planning/priority.md), specs in [`planning/mods/`](planning/mods). Contributor guide: [`CONTRIBUTING.md`](CONTRIBUTING.md).
