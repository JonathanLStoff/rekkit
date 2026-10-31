# Rekkit - Project Instructions for Claude

## Start here
Run `./setup.sh --check` at the start of every session and follow `AGENTS.md` → *Start of every session*: relay any `!` line to the user word for word (usually: play Minecraft 26.3 once in the official launcher), then take the next row of `planning/fleet-board.md` (`rekkit_core`, then `planning/priority.md` order, then the other modpacks) and keep working. Contributor guide: `CONTRIBUTING.md`.

## Overview & Architecture Context
Rekkit is a Rust port of classic Minecraft modpacks (Tekkit Classic 1.2.5, Direwolf20 1.4.7) designed to run on the **Horizon** engine ([`horizon/`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/horizon)).

> [!IMPORTANT]
> **Horizon Dual-Artifact Mod Architecture (No WebAssembly):**
> Horizon has removed its legacy WebAssembly (`.wasm`) runtime and splits mods into two artifacts:
> 1. **Content Mod (`.hmod`, API `0.2`, `capabilities = ["registries"]`)**: Pure data and assets (`content/horizon-mod.toml`, `content/data/`, `content/assets/`) declaring blocks (with optional item `components`), items (with optional `crafting_remainder`), custom `mob_effect` types, recipes (including custom machine recipe types), loot tables, tags, models, and textures. **No code (`Cargo.toml`) is allowed inside `content/`** (`cargo-horizon-mod` explicitly rejects directories with `Cargo.toml`).
> 2. **Native Mod (`.hnmod`, `NATIVE_HOOKS_API = 40`)**: Source-level [horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle (`native/horizon-native.json` + `native/mixins/<id>/horizon-mixin.toml` + `.rs` helpers) hooking into [`crates/horizon-client/src/game/native_hooks.rs`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/horizon/crates/horizon-client/src/game/native_hooks.rs) (Hooks 8–40: block entities, containers, machine ticks, custom machine recipe types, redstone signals, dynamic cable/pipe shape updates, player/inventory ticks, custom `horizon:mod_net` packets, custom GUI screens, and server commands).
> 3. **Pure Rust Domain Crate (`mods/<modname>/Cargo.toml` + `src/lib.rs`)**: Standalone, 100% unit-testable Rust crate housing domain logic, energy/EMC math, state machines, and `HorizonNativeMod` trait implementations.

---

## Mod Porting Workflow
Whenever converting or implementing a Java mod for Horizon in Rekkit, follow this strict 10-step lifecycle (detailed in [`planning/workflow.md`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/planning/workflow.md)):

1. **Gather Requirements:** Document complete specifications into `planning/mods/<modname>.md` (blocks, items, tile entities, GUI layout, formulas, energy, EMC, recipes, hazards).
2. **Create Dual-Artifact Rust Mod Scaffolding:** Create crate under `./mods/<modname>` containing:
   * `Cargo.toml` & `src/lib.rs` (domain models, state machines, and `HorizonNativeMod` API 40 lifecycle traits)
   * `horizon-registry.toml` (Horizon Mod Registry repository page metadata: `name`, `summary`, `description`, `authors`, `license`)
   * `content/horizon-mod.toml` (`api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`) + `content/data/<id>/horizon/{block,item,mob_effect}/*.json`
   * `native/horizon-native.json` (`schema = 1`, `game = "26.3"`, pinned 40-hex `horizon` & `mixins` revisions, `hooks = 40`) + `native/mixins/<id>/horizon-mixin.toml` & `hooks.rs`
3. **Initial Testing:** Write and execute unit & native hook lifecycle tests in `./mods/<modname>/tests/` (`cargo test --manifest-path mods/<modname>/Cargo.toml` or `make test-mods`).
4. **Extract Legacy Assets:** Use the `extract-jar-assets` skill or `make extract-jar JAR=assets/.../<mod>.jar` to unpack sprites, audio, models, and scripts from the original Java mod archive.
5. **Add Assets to Content Mod:** Place extracted assets into `./mods/<modname>/content/assets/<modname>/` (textures, sounds, lang, models, blockstates, items).
6. **Asset, Schema & Logic Validation:** Run `make test-mods` and `make check-mods MOD=<modname>` to verify asset parity, translation keys, manifest rules, and serialization.
7. **Integration & Compatibility Testing (`planning/harness.md`):** Run `make integration-test` (`scripts/e2e_harness.py`), which executes Phase 1 (`make e2e-mod MOD=<modname>`: boots `horizon-client` with `horizon-ctl`, places all blocks in a world, and executes all 7 action stages) and Phase 2 (`make e2e-all-mods`: boots Horizon with all Rekkit mods loaded together to verify cross-mod compatibility). Use `make integration-test DRY_RUN=1` for pre-flight validation.
8. **Update Versioning:** Increment version in Lockstep across `Cargo.toml`, `content/horizon-mod.toml`, `native/horizon-native.json`, and `native/mixins/<id>/horizon-mixin.toml`.
9. **Git Push:** Stage, commit with conventional commit format, and push (`git commit -m "feat(<modname>): ..."`).
10. **Package & Publish GitHub Release:** Run `make pack-mods MOD=<modname>` to produce `<id>.hmod` and `<id>.hnmod` in `target/horizon-mod/`, sign with `minisign`, and publish as GitHub Release assets for automatic indexing by `horizon-rust/horizon-mods`.

---

## Reference Example
A fully working template implementing this dual-artifact workflow is located in [`mods/example_mod/`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod):
* [`mods/example_mod/Cargo.toml`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/Cargo.toml)
* [`mods/example_mod/horizon-registry.toml`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/horizon-registry.toml)
* [`mods/example_mod/src/lib.rs`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/src/lib.rs)
* [`mods/example_mod/tests/mod_tests.rs`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/tests/mod_tests.rs)
* [`mods/example_mod/content/horizon-mod.toml`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/content/horizon-mod.toml)
* [`mods/example_mod/native/horizon-native.json`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/native/horizon-native.json)
* [`mods/example_mod/native/mixins/example_mod/horizon-mixin.toml`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/native/mixins/example_mod/horizon-mixin.toml)

---

## Common Build, Test & Asset Commands
* `make test-mods` - Runs `cargo test` across all Rust mod crates under `./mods/`.
* `./setup.sh` / `make setup` - Sets up a fresh clone end to end; `./setup.sh --check` / `make doctor` reports only.
* `make check-mods [MOD=<id>]` - Validates `.hmod` content and `.hnmod` native manifests/assets against Horizon rules.
* `make pack-mods [MOD=<id>]` - Builds deterministic `.hmod` and `.hnmod` archives in `mods/<id>/target/horizon-mod/`.
* `make tekkit` - Downloads Tekkit Classic 1.2.5 server pack and unpacks mod archives.
* `make direwolf20` - Fetches Direwolf20 1.4.7 modpack files from FTB API.
* `make agrarian` - Fetches Agrarian Skies 2 2.0.6a with `CURSEFORGE_API_KEY`; if the key is missing or rejected, falls back to the public 2.0.6 download (CurseForge file 3016703).
* `make all` - Fetches all packs (Agrarian Skies 2 uses the keyless fallback when needed).
* `make extract-jar JAR=<path_to_mod.jar> [EXTRACT_OUT=<path>] [ARGS="..."]` - Extracts textures, sprites, audio, and scripts from a mod archive.
* `make extract-assets [DIR=<path_to_mods_folder>] [EXTRACT_OUT=<path>]` - Batch extracts assets from all mod archives in a folder.

## Skills
* `extract-jar-assets` (in `.agents/skills/extract-jar-assets/SKILL.md` and `.claude/skills/extract-jar-assets/SKILL.md`): Use whenever extracting textures, sprites, audio, models, localizations, or Lua/ROM scripts from mod JARs/ZIPs for reverse engineering or recreation in Rekkit.

## Planning & Specifications
* Workflow Guide: [`planning/workflow.md`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/planning/workflow.md)
* Priority list: [`planning/priority.md`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/planning/priority.md)
* Comprehensive mod recreation specifications: [`planning/mods/*.md`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/planning/mods)
