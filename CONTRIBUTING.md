# Contributing to Rekkit

Rekkit ports classic Java Minecraft mods (Tekkit Classic, Direwolf20, Agrarian Skies 2) to Rust mods for the [Horizon](https://github.com/horizon-rust/horizon) engine. Help is welcome from people and from AI agents. This page is everything you need; no other instructions are required.

## 1. Set up (once, about 20–40 minutes, mostly unattended)

```sh
git clone https://github.com/JonathanLStoff/rekkit.git
cd rekkit
./setup.sh          # or: make setup    (Windows: setup.cmd, or make setup)
```

`./setup.sh` does every step a machine can do and is safe to run again:

| Step | What it does |
|---|---|
| Tools | Checks git, a C linker (MSVC Build Tools on Windows), Rust 1.90+, Python 3.11+, JDK 25 (and make, which is optional) and prints the install command for anything missing |
| Submodule | Initialises `horizon/` (the engine) |
| Mods | What `make test-mods` and `make check-mods` run |
| Beads and port tools | Installs nothing itself: asks you to `brew install beads` if missing, then sets up its database (`bd bootstrap`) and fetches the pinned Vineflower decompiler |
| Modpacks | Downloads Tekkit Classic, Direwolf20 and Agrarian Skies 2 into `assets/` (the original mod JARs you port from). No CurseForge key needed |
| Horizon | Builds the game with every mod's native hooks (`target/e2e/`) |
| Minecraft 26.3 | Builds Horizon's game resources from **your own** Minecraft install and fetches the matching server jar |
| Smoke test | Boots the game offscreen with `example_mod` and runs its 41 test actions |

It ends with a summary and an exit status: `0` ready, `2` ready except lines marked `!` that need you, `1` something failed (the `✗` line says how to fix it). `./setup.sh --check` (or `make doctor`) reports without changing anything.

**The one thing only you can do:** own Minecraft: Java Edition, install the official launcher, sign in, choose version **26.3**, press Play once and quit. Horizon never downloads Minecraft; it builds its files from your copy. Then run `./setup.sh` again. Until you do, everything except the live game test still works.

Supported: macOS (Apple Silicon or Intel), Linux and Windows 10/11. On Windows run `setup.cmd` (or `make setup` with GNU make, e.g. `winget install ezwinports.make`); it also asks you to turn on long paths once. Linux also needs `sudo apt install pkg-config libasound2-dev libudev-dev` and a desktop session (or Xvfb) for the live game test.

## 2. Pick something to work on

The queue is [`planning/fleet-board.md`](planning/fleet-board.md):

1. `rekkit_core`, the shared Rust library every mod uses,
2. then the mods in [`planning/priority.md`](planning/priority.md) order (RedPower 2, BuildCraft, IndustrialCraft 2, …),
3. then the other Tekkit mods, then Direwolf20, then Agrarian Skies 2.

Take the `active` row, else the first `queued` row whose Deps are `done`. Before starting, check open pull requests and issues for that mod, and comment on (or open) an issue to claim it, so two people don't port the same mod.

## 3. Do the work

Every mod follows the 10 steps in [`planning/workflow.md`](planning/workflow.md). The short version, with the rules, is [`.agents/skills/port-mod/SKILL.md`](.agents/skills/port-mod/SKILL.md). Copy the layout of [`mods/example_mod`](mods/example_mod), which passes every check including the live game test.

| What | Command |
|---|---|
| Unit tests (one mod) | `cargo test --manifest-path mods/<id>/Cargo.toml` |
| Manifest and content checks | `make check-mods MOD=<id>` |
| Test plan only (fast) | `make e2e-mod MOD=<id> DRY_RUN=1` |
| Live game test, one mod | `make e2e-mod MOD=<id>` |
| Live game test, mods together | `make e2e-all-mods MOD=a,b` (or all mods) |
| Unpack an original mod JAR | `make extract-jar JAR=assets/<pack>/mods/<jar> EXTRACT_OUT=assets/extracted/<id>` |

Live test results, game logs, crash reports and screenshots land in `target/e2e-report/<session>/`. Something the engine can't do yet is a Beads bug labelled `horizon` (`bd list --label horizon`), never a change to `horizon/`.

## 4. Send it

- `make test-mods`, `make check-mods MOD=<id>` and `make e2e-mod MOD=<id>` pass.
- The mod's row in `planning/fleet-board.md` is updated.
- Versions match across `Cargo.toml`, `content/horizon-mod.toml`, `native/horizon-native.json` and `native/mixins/<id>/horizon-mixin.toml`.
- Conventional commit: `feat(<id>): …`, `fix(<id>): …`.
- Open a pull request. Never commit Minecraft files (`assets/`, `target/`, `.local/` are ignored for that reason).

## Working with an AI agent

Any agent works: agy, Claude Code, Codex, Cursor, Copilot, Gemini CLI. Tell it: **"Follow AGENTS.md."** [`AGENTS.md`](AGENTS.md) makes it check the setup, tell you word for word what to download if something needs you, then take the next board row and keep going. In agy (`/agents` → conductor) and Claude Code (`/conductor`) it runs as a fleet of role subagents with hooks that block pushing and signing; other tools play the roles one by one. Each mod goes through 11 steps (spec from the decompiled Java, code written from the spec only, checks, the original's assets, new art only for what it lacks, a recheck, the live harness with every block placed and no warnings, packaging) and ends with `DONE, CHECK OUT THE PROGRESS: make play MOD=<id>`: then it is yours to play, review and push ([`planning/workflow.md`](planning/workflow.md) §2). Progress lives in Beads (`bd`, installed by `./setup.sh`): `make fleet-status`, `bd ready`, or the generated [board](planning/fleet-board.md). Unattended: `make fleet-next AGENT=agy|claude|codex`. The fleet is described in [`.agents/fleet-plan.md`](.agents/fleet-plan.md). To change a role or skill, edit `.agents/` and run `make agents`. Agents never push, sign or publish; you do.

## When something goes wrong

| Symptom | Fix |
|---|---|
| `Python 3.11 or newer is required` | Install a newer Python (`brew install python@3.13`); the scripts find it themselves |
| `✗ tool: Rust` | `rustup update` (or install rustup) |
| `! Minecraft 26.3 … not found` | Play 26.3 once in the official launcher, then `./setup.sh` |
| `E_MANIFEST` from horizon-mixins | `make check-mods MOD=<id>` explains it: `[mod]` takes only `id`, `version`, `priority`, and a bundle needs at least one `[[injection]]` |
| A block drops nothing in the live test | Copy the loot table form from `mods/example_mod` (`bd list --label horizon`: the loot-pool `conditions` gap) |
| Anything else in the live test | `target/e2e-report/summary.json` lists every action and reply; the game log and crash reports are next to it |
