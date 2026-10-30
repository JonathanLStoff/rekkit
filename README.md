# rekkit
Rust port of classic Tekkit and a few other mods


## Party up!
Contribute to creating a substantial technical recreation of historical Minecraft modifications for an entirely different game implementation!
## Legacy Minecraft modpack asset collector

Downloads the mod JARs for three pinned legacy packs and extracts only the `assets/` contents embedded in each mod JAR/ZIP.

## Packs

- Tekkit Classic 1.2.5 — Technic server package 3.1.2
- Direwolf20 1.4.7 — FTB pack 5.3.2 (pack ID 70, version ID 185)
- Agrarian Skies 2 1.7.10 — CurseForge 2.0.6a (project ID 225550, file ID 3045516)

## Run

Python 3.10+ and `make` are required. No Python packages are required.

```sh
make list
make tekkit
make direwolf20
CURSEFORGE_API_KEY=YOUR_KEY make agrarian
# Or all three:
CURSEFORGE_API_KEY=YOUR_KEY make all
```

CurseForge requires an API key for direct file downloads. Create one at <https://forms.monday.com/forms/dce5ccb7afda9a1c21dab1a1aa1d84eb?r=use1> and pass it as an environment variable. Do not put the key in this repository or commit it.

## Output

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

## Notes

- Only JARs under `mods/` or `coremods/` are copied from ZIP-based packs. Configs, worlds, scripts, and launcher files are intentionally excluded.
- Direwolf20's FTB manifest is read from the FTB API and files tagged as mods are downloaded directly; SHA-1 values supplied by FTB are verified when present.
- Agrarian Skies 2's CurseForge manifest is used to download required mod files. Optional manifest entries are skipped.
