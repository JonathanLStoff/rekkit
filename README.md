# rekkit
Rust port of classic Tekkit and legacy FTB mods targeting the **[Horizon](https://github.com/horizon-rust/horizon)** engine.

## Quick start

```sh
git clone https://github.com/JonathanLStoff/rekkit.git && cd rekkit
./setup.sh          # sets up everything a machine can; names anything that needs you
                    # same as `make setup`; on Windows: setup.cmd (or make setup)
```

Then read [CONTRIBUTING.md](CONTRIBUTING.md) to pick a mod from [the board](planning/fleet-board.md). Using an AI agent? Tell it **"Follow AGENTS.md."**

The only manual step: own Minecraft: Java Edition and play version 26.3 once in the official launcher (Horizon builds its game files from your copy).

## Architecture: Horizon Dual-Artifact Mod System

Horizon has removed its legacy WebAssembly runtime (`docs/no-wasm-docs`) and splits mods into two artifacts:
1. **Content Mod (`.hmod`, API `0.2`, `capabilities = ["registries"]`)**: Pure data and assets (`content/horizon-mod.toml`, `content/data/`, `content/assets/`) declaring blocks (with optional item `components`), items (with optional `crafting_remainder`), custom `mob_effect` types, recipes (including custom machine recipe types), loot tables, tags, models, and textures.
2. **Native Mod (`.hnmod`, `NATIVE_HOOKS_API = 40`)**: Source-level [horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle (`native/horizon-native.json` + `native/mixins/<id>/`) hooking into `crates/horizon-client/src/game/native_hooks.rs` (block entities, containers, server ticks, custom machine recipes, redstone signals, neighbor shape updates, player/inventory ticks, custom `horizon:mod_net` packets, custom screens).
3. **Pure Rust Domain Crate (`mods/<id>/Cargo.toml` + `src/lib.rs`)**: Standalone, 100% unit-testable Rust crate containing machine state machines, energy/EMC math, and `HorizonNativeMod` trait implementations.

See [`planning/workflow.md`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/planning/workflow.md) and the reference template in [`mods/example_mod/`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod) for full details.

## Testing & Packaging Mods

```sh
# Run unit & native hook lifecycle tests across all mods in ./mods/
make test-mods

# Validate content (.hmod) and native (.hnmod) manifests and schemas
make check-mods

# Package deterministic .hmod and .hnmod release archives into target/horizon-mod/
make pack-mods

# Run Phase 1 (isolated per-mod) + Phase 2 (all-mods compatibility) E2E harness
make integration-test DRY_RUN=1
make integration-test
```

---

## Legacy Minecraft Modpack Asset Collector

Downloads the mod JARs for three pinned legacy packs and extracts `assets/` and legacy sprites/audio/scripts embedded in each mod JAR/ZIP.

### Packs

- Tekkit Classic 1.2.5 — Technic 3.1.2: the server package (`assets/tekkit/mods/`) and the client build from Technic's Solder API, with the textures and sounds (`assets/tekkit/client/`)
- Direwolf20 1.4.7 — FTB pack 5.3.2 (pack ID 70, version ID 185)
- Agrarian Skies 2 1.7.10 — CurseForge 2.0.6a (project ID 225550, file ID 3045516)

### Run

Python 3.11+ and `make` are required. No third-party Python packages are required.

```sh
make list
make tekkit
make direwolf20
CURSEFORGE_API_KEY=YOUR_KEY make agrarian
# Or all three:
CURSEFORGE_API_KEY=YOUR_KEY make all
```

With a working CurseForge API key, `make agrarian` fetches the pinned 2.0.6a pack through the official API. Create a key at <https://forms.monday.com/forms/dce5ccb7afda9a1c21dab1a1aa1d84eb?r=use1> and pass it as an environment variable. Do not put the key in this repository or commit it.

If the key is missing or rejected, the script falls back to the public download of Agrarian Skies 2 **2.0.6** (<https://www.curseforge.com/minecraft/modpacks/agrarian-skies-2/download/3016703>) and fetches each mod through CurseForge's keyless website endpoint. `pack-info.json` records which pack version was actually fetched.

### Output

```text
assets/
  tekkit/
    mods/
    coremods/
    mod-assets/<jar-name>/assets/<namespace>/...
    pack-assets/assets/...
    pack-info.json
  direwolf20/
    mods/
    coremods/
    mod-assets/...
    pack-info.json
  agrarian/
    mods/
    mod-assets/...
    pack-assets/assets/...
    pack-info.json
```

The `mod-assets` layout intentionally keeps each JAR's assets separate so two mods shipping the same resource path cannot overwrite one another. `pack-assets` contains loose `assets/` files supplied by the pack itself. Vanilla Minecraft assets are not downloaded. The source archives/API manifests are cached under `.cache/modpacks/`.

Use `make clean` to remove the output and cache. Use `make all FORCE=1` to redownload cached files.

### Notes

- Only JARs under `mods/` or `coremods/` are copied from ZIP-based packs. Configs, worlds, scripts, and launcher files are intentionally excluded.
- Direwolf20's FTB manifest is read from the FTB API and files tagged as mods are downloaded directly; SHA-1 values supplied by FTB are verified when present.
- Agrarian Skies 2's CurseForge manifest is used to download required mod files. Optional manifest entries are skipped.
