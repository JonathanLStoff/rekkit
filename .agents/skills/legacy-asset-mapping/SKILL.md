---
name: legacy-asset-mapping
description: Map textures and sounds from pre-1.6 Minecraft mods (Tekkit 1.2.5, Direwolf20 1.4.7) to per-block and per-item files in a Rekkit content mod. Use after extract-jar-assets, whenever sprite sheets or legacy texture paths need slicing or renaming.
---

# legacy-asset-mapping

Pre-1.6 mods did not use `assets/<modid>/textures/`. Run the `extract-jar-assets` skill first, then map what it produced.

When the mod ships in several packs, take each texture and sound from the version the spec uses for that feature: the newest version that has it (a feature only an older version has keeps that version's art). Record the source archive in each `asset-map.json` entry.

Use the **client** build of each version: `assets/tekkit/client/mods/` for Tekkit (Technic's client build; `assets/tekkit/mods/` holds the server ports, classes only, no PNG or OGG), `assets/direwolf20/mods/` for Direwolf20. Each pack's `pack-info.json` gives `art_files` per archive. Some Tekkit mods ship unpacked as a folder (`client/mods/ComputerCraft/`). If no client build of the right mod is under `assets/`, report it as a follow_up instead of drawing replacements: new art is only for what the original never had (step 6).

## What to expect

- **1.2.5 (Tekkit)**: blocks and items mostly come from **sprite sheets**: 256×256 PNGs holding a 16×16 grid of 16 px icons. A block's texture index `i` sits at column `i % 16`, row `i / 16`. Sheets live at mod-specific paths such as `/ic2/sprites/block_0.png`, `/eloraam/...`, `/gfx/...`, `/terrain/...`. The index comes from the decompiled block class (`blockIndexInTexture` / `getBlockTextureFromSideAndMetadata`).
- **1.4.7 (Direwolf20)**: still sprite sheets for most mods, though some already ship individual PNGs. Indexes come from `getBlockTextureFromSideAndMetadata` / `getIconFromDamage`.
- **1.7.10 (Agrarian Skies 2)**: individual PNGs under `assets/<modid>/textures/blocks|items/`. Mostly rename and move.
- GUIs are whole 256×256 backgrounds (`/gui/*.png`); copy them as-is to `textures/gui/`.
- Sounds are `.ogg`/`.wav` under various roots; Horizon wants `.ogg` under `sounds/`.

Indexes come from decompiled classes, not guesswork. If an index is unclear, mark the entry `UNVERIFIED` in the map and tell the conductor.

## asset-map.json

Record every slice and copy in `mods/<id>/asset-map.json` so the mapping can be re-run, each with `"origin": "original"` (the original mod's pixels; anything unrecorded also counts as original and keeps the mod on pre-releases, see `.agents/release.md`). Set `"credits"` to the original mod and its authors, `"resolution"` to the source pixel size, and list in `"vanilla_ok"` the blocks/items that borrow a vanilla look on purpose. The paths and index below are illustrative; use the real ones from the JAR:

```json
{
  "source_jar": "assets/direwolf20/mods/IC2.jar",
  "credits": "IndustrialCraft 2 by Alblaka and the IC2 team",
  "resolution": 16,
  "vanilla_ok": [],
  "slices": [
    { "sheet": "ic2/sprites/block_0.png", "index": 18, "size": 16,
      "out": "content/assets/industrialcraft_2/textures/block/macerator_front.png",
      "for": "industrialcraft_2:macerator", "face": "front", "origin": "original" }
  ],
  "copies": [
    { "from": "ic2/sprites/GUIMacerator.png",
      "out": "content/assets/industrialcraft_2/textures/gui/macerator.png", "origin": "original" }
  ]
}
```

## Naming

- Textures: `textures/block/<name>[_<face>].png`, `textures/item/<name>.png`, `textures/gui/<machine>.png`, matching what the content-author's models reference.
- Keep source pixel size (16 px); do not upscale.
- Animated sheets (fluids, lava-like blocks) become a vertical strip PNG plus a `.mcmeta` next to it.
