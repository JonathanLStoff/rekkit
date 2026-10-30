# Rekkit - Project Instructions for Claude

## Overview & Architecture Context
Rekkit is a Rust port of classic Minecraft modpacks (Tekkit Classic 1.2.5, Direwolf20 1.4.7) designed to run on the **Horizon** engine (`horizon/`).

> [!IMPORTANT]
> **Horizon Engine Status:**
> Horizon has not yet been fully updated to support third-party mod loading and integration (mod API levels 0.2/0.3 are under active development). We are currently waiting on Horizon development to finish enough to start live engine integration.
> **Strategy:** Implement the complete domain logic, state machines, registries, math formulas, and asset pipelines as standalone Rust crates under `./mods/<modname>` so they are fully tested and ready to plug in as Horizon matures.

---

## Mod Porting Workflow
Whenever converting or implementing a Java mod for Horizon in Rekkit, follow this strict 10-step lifecycle (detailed in [`planning/workflow.md`](file:///C:/Users/legoc/Desktop/Code_Projects/rekkit/planning/workflow.md)):

1. **Gather Requirements:** Document complete specifications into `planning/mods/<modname>.md` (blocks, items, tile entities, GUI layout, formulas, energy, EMC, recipes, hazards).
2. **Create Rust Mod Scaffolding:** Create crate under `./mods/<modname>` containing:
   * `Cargo.toml` (package config & dependencies)
   * `horizon-mod.toml` (Horizon mod manifest: `id`, `name`, `version`, `api = "0.2"`, `side = "server"`)
   * `src/lib.rs` (domain models, state machines, lifecycle traits)
3. **Initial Testing:** Write and execute unit tests in `./mods/<modname>/tests/` (`cargo test --manifest-path mods/<modname>/Cargo.toml`).
4. **Extract Legacy Assets:** Use the `extract-jar-assets` skill or `make extract-jar JAR=assets/.../<mod>.jar` to unpack sprites, audio, models, and scripts from the original Java mod archive.
5. **Add Assets to Mod:** Place extracted assets into standard resource structure: `./mods/<modname>/assets/<modname>/` (textures, sounds, lang, models).
6. **Asset & Logic Validation:** Re-run test suites (`cargo test`) to verify asset parity, translation keys, and serialization.
7. **[TODO / TBD] Integration Testing via Mod Harness:** Test against a headless mock world harness (`crates/mod-harness`) simulating ticks, chunk grids, and inventory transfers. *(TODO: Mod harness runner to be built)*.
8. **Update Versioning:** Increment version in both `./mods/<modname>/Cargo.toml` and `./mods/<modname>/horizon-mod.toml`.
9. **Git Push:** Stage, commit with conventional commit format, and push (`git commit -m "feat(<modname>): ..."`).
10. **[TODO / TBD] CI/CD Actions Upload:** GitHub Actions workflow to build release artifacts (`.hmod` / `.wasm`) and upload to the Horizon mod registry. *(TODO: CI workflow to be built)*.

---

## Reference Example
A fully working template implementing this workflow is located in [`mods/example_mod/`](file:///C:/Users/legoc/Desktop/Code_Projects/rekkit/mods/example_mod):
* [`mods/example_mod/Cargo.toml`](file:///C:/Users/legoc/Desktop/Code_Projects/rekkit/mods/example_mod/Cargo.toml)
* [`mods/example_mod/horizon-mod.toml`](file:///C:/Users/legoc/Desktop/Code_Projects/rekkit/mods/example_mod/horizon-mod.toml)
* [`mods/example_mod/src/lib.rs`](file:///C:/Users/legoc/Desktop/Code_Projects/rekkit/mods/example_mod/src/lib.rs)
* [`mods/example_mod/tests/mod_tests.rs`](file:///C:/Users/legoc/Desktop/Code_Projects/rekkit/mods/example_mod/tests/mod_tests.rs)
* [`mods/example_mod/assets/example_mod/lang/en_us.json`](file:///C:/Users/legoc/Desktop/Code_Projects/rekkit/mods/example_mod/assets/example_mod/lang/en_us.json)

---

## Common Build & Asset Commands
* `make tekkit` - Downloads Tekkit Classic 1.2.5 server pack and unpacks mod archives.
* `make direwolf20` - Fetches Direwolf20 1.4.7 modpack files from FTB API.
* `make agrarian` - Fetches Agrarian Skies 2 (Note: CurseForge API key is ignored if non-working/missing).
* `make all` - Fetches all packs, ignoring missing or invalid API keys.
* `make extract-jar JAR=<path_to_mod.jar> [EXTRACT_OUT=<path>] [ARGS="..."]` - Extracts textures, sprites, audio, and scripts from a mod archive.
* `make extract-assets [DIR=<path_to_mods_folder>] [EXTRACT_OUT=<path>]` - Batch extracts assets from all mod archives in a folder.

## Skills
* `extract-jar-assets` (in `.claude/skills/extract-jar-assets/SKILL.md` and `.claude/skills/extract-jar-assets.md`): Use whenever extracting textures, sprites, audio, models, localizations, or Lua/ROM scripts from mod JARs/ZIPs for reverse engineering or recreation in Rekkit.

## Planning & Specifications
* Workflow Guide: `planning/workflow.md`
* Priority list: `planning/priority.md`
* Comprehensive mod recreation specifications: `planning/mods/*.md`
