# Rekkit reference

Read the part you need, when you need it; `make help` lists every command.

### Architecture: Horizon dual-artifact mods (no WebAssembly)

Every mod in `mods/<id>/` is three parts:

1. **Content mod (`.hmod`, API `0.2`, `capabilities = ["registries"]`)**: data and assets only (`content/horizon-mod.toml`, `content/data/`, `content/assets/`): blocks (with optional item `components`), items (with optional `crafting_remainder`), custom `mob_effect` types, creative tabs (`creative_tab/`: every item must be in one; needs Horizon 74ff545e6 or later), recipes (including custom machine recipe types), loot tables, tags, models and textures. No code: `cargo-horizon-mod` rejects a `content/` with a `Cargo.toml`.
2. **Native mod (`.hnmod`, `NATIVE_HOOKS_API = 40`)**: a source-level [horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle (`native/horizon-native.json` + `native/mixins/<id>/horizon-mixin.toml` + `.rs` helpers) hooking into `horizon/crates/horizon-client/src/game/native_hooks.rs` (hooks 8–40: block entities, containers, machine ticks, custom machine recipe types, redstone signals, cable/pipe shape updates, player/inventory ticks, `horizon:mod_net` packets, custom GUI screens, server commands).
3. **Pure Rust domain crate (`Cargo.toml` + `src/lib.rs`)**: standalone, fully unit-testable domain logic, energy/EMC math, state machines and `HorizonNativeMod` implementations.

The template is [`mods/example_mod/`](../mods/example_mod): `Cargo.toml`, `horizon-registry.toml` (registry page: `name`, `summary`, `description`, `authors`, `license`), `src/lib.rs`, `tests/mod_tests.rs`, `content/horizon-mod.toml`, `native/horizon-native.json` (`schema = 1`, `game = "26.3"`, pinned 40-hex `horizon` and `mixins` revisions, `hooks = 40`), `native/mixins/example_mod/horizon-mixin.toml`.

### The 11-step port process ([`planning/workflow.md`](../planning/workflow.md) §2)

| # | Step | Done when |
|---|---|---|
| 1 | **Spec and tickets from the Java**: an outline with attributes and test levels, then one spec-writer per part of the code writing ≤200k-token tickets with their order | tickets filed (`make fleet-tickets`) |
| 2 | **Scaffold** the three parts above | it builds; check-mods passes |
| 3 | **Code**: one subagent per ready ticket, many at once; spec first, then the Java; each block's contract file first | every ticket closed |
| 4 | **Check** syntax and logic: `make agent-gate`, parity-critic against spec and Java | clean |
| 5 | **Extract assets**: the original's art and sound from a client build, `origin: original`, credited | every block and item mapped |
| 6 | **New assets**: only what 26.3 needs and the original never had, at its size and palette (`make fleet-gaps`, `make art`) | `make check-mods COMPLETE=1` passes |
| 7 | **Recheck** everything on the whole mod | clean |
| 8 | **Harness (live)**: every block placed (by `/setblock` and a player, every variant), 7 stages, no warnings | every block's harness feature closed (Horizon's own failures don't block) |
| 9 | **Package**: version, commit with the board, pack | committed and packed: **complete, ready for a human** |
| 10 | **Human**: `make play MOD=<id>`, review, push; the workflow publishes a pre-release while original assets remain | pushed |
| 11 | **Own art** (later): replace every original asset; then full releases | 0 original assets |

### Commands

| Command | What it does |
|---|---|
| `./setup.sh` / `setup.cmd` / `make setup` | Set up a fresh clone end to end; `--check` / `make doctor` reports only |
| `make test-mods` | `cargo test` for every crate under `mods/` |
| `make check-mods [MOD=<id>] [COMPLETE=1]` | Validate `.hmod` content and `.hnmod` manifests against Horizon's rules; `COMPLETE=1` also fails on any asset 26.3 needs that is missing, and on any contract difference |
| `make contract-check MOD=<id>` | The port against its contract (`planning/mods/<id>.features.json`): shapes, facing, each face's and icon's sprite pixel for pixel, stack sizes, durability |
| `make fleet-contract MOD=<id>` | File each contract difference (static and the last harness run's GUI, shape and scenario failures) as a bug under its step, reopening what it touches |
| `make pack-mods [MOD=<id>]` | Build deterministic `.hmod` and `.hnmod` in `mods/<id>/target/horizon-mod/` |
| `make e2e-setup` / `e2e-mod` / `e2e-all-mods` / `integration-test` | The harness: build Horizon with the mods' mixins, then boot it and run the action plans (exit 3: only Horizon's own problems) |
| `make horizon-baseline` / `horizon-status` | Build and run Horizon alone at the pinned revision / show whether it is healthy |
| `make play [MOD=<id>]` | Open Horizon in a window with the mod (default: all) and a world with every block placed |
| `make decompile JAR=<archive>` | The original's Java in `assets/decompiled/` (spec-writer, reviewer) |
| `make art MOD=<id>` | New art from `mods/<id>/art/` at the original's size and palette |
| `make agent-gate [MOD=<id>]` | rustfmt, clippy `-D warnings`, tests, check-mods and dry run of what you changed (what the stop gate runs) |
| `make agents` / `make agents-check` | Regenerate / check each tool's copy of `.agents/` |
| `make fleet-next [AGENT=…] [MOD=<id>]` | A headless conductor run on one mod, relaunched until it is complete, on the next model whenever one hits a limit |
| `make fleet-models [ARGS=--clear]` | The model chain (`.agents/models.json`) and which models are resting after a limit |
| `make fleet-start MOD=<id>` / `fleet-stop` | Start a mod (claim, stories, Horizon check) / let every fleet agent stop |
| `make fleet-task BEAD=<id> ROLE=<role>` / `fleet-inbox` | A subagent takes a bead / the conductor files the reports |
| `make fleet-local [ARGS=...]` | Local models only: how many agents the machine's VRAM (or unified memory, or RAM) holds on the selected model, and the Ollama settings to serve them (`target/fleet/local.json`) |
| `make fleet-parts MOD=<id>` | The decompiled code in ~200k-token parts, one spec-writer each (lists in `target/fleet/parts/<id>/`) |
| `make fleet-tickets MOD=<id> [CHECK=1]` | Validate the spec-writers' tickets (sizes, roles, order, shared files, coverage) and file them with their order |
| `make fleet-ready MOD=<id>` | Tickets whose predecessors are all closed: one subagent each |
| `make fleet-features MOD=<id>` / `fleet-gaps MOD=<id>` / `fleet-status [MOD=<id>]` / `board` | Beads: features from the spec / from missing assets / progress / regenerate the board |
| `make tasks [MOD=<id>]` / `watch` / `board-live` / `board-html` | Look at progress: the mod in progress on one screen / redrawn every 30 s / the board in your browser, live or as a file |
| `make fleet-resume MOD=<id>` | Where a run stands after a crash, lost credits or a network drop: interrupted subagents, waiting reports, the last actions |
| `make fleet-context MOD=<id>` / `spec-toc` / `spec-show` / `java-outline` / `java-show` / `java-find` | Read narrowly: one page per mod, a spec by section, decompiled Java by member |
| `make agy` / `make fleet-models` | agy with its sandbox off / the model chain and which models are resting |
| `make tekkit` / `direwolf20` / `agrarian` / `all` | Download the modpacks into `assets/` (Agrarian Skies 2 falls back to the public 2.0.6 without `CURSEFORGE_API_KEY`) |
| `make extract-jar JAR=<path> [EXTRACT_OUT=<dir>]` / `make extract-assets [DIR=<dir>]` | Extract textures, sprites, audio, models and scripts from mod archives |

Planning: [`planning/workflow.md`](../planning/workflow.md), [`planning/priority.md`](../planning/priority.md), specs in [`planning/mods/`](../planning/mods). Contributor guide: [`CONTRIBUTING.md`](../CONTRIBUTING.md).
