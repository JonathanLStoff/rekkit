---
name: legacy-asset-mapping
description: Map textures and sounds from pre-1.6 Minecraft mods (Tekkit 1.2.5, Direwolf20 1.4.7) to per-block and per-item files in a Rekkit content mod. Use after extract-jar-assets, whenever sprite sheets or legacy texture paths need slicing or renaming.
---

# legacy-asset-mapping

Pre-1.6 mods did not use `assets/<modid>/textures/`. Run the `extract-jar-assets` skill first, then map what it produced.

## What to expect

- **1.2.5 (Tekkit)**: blocks and items mostly come from **sprite sheets**: 256×256 PNGs holding a 16×16 grid of 16 px icons. A block's texture index `i` sits at column `i % 16`, row `i / 16`. Sheets live at mod-specific paths such as `/ic2/sprites/block_0.png`, `/eloraam/...`, `/gfx/...`, `/terrain/...`. The index comes from the decompiled block class (`blockIndexInTexture` / `getBlockTextureFromSideAndMetadata`).
- **1.4.7 (Direwolf20)**: still sprite sheets for most mods, though some already ship individual PNGs. Indexes come from `getBlockTextureFromSideAndMetadata` / `getIconFromDamage`.
- **1.7.10 (Agrarian Skies 2)**: individual PNGs under `assets/<modid>/textures/blocks|items/`. Mostly rename and move.
- GUIs are whole 256×256 backgrounds (`/gui/*.png`); copy them as-is to `textures/gui/`.
- Sounds are `.ogg`/`.wav` under various roots; Horizon wants `.ogg` under `sounds/`.

Indexes come from decompiled classes, not guesswork. If an index is unclear, mark the entry `UNVERIFIED` in the map and tell the conductor.

## asset-map.json

Record every slice in `mods/<id>/asset-map.json` so the mapping can be re-run. The paths and index below are illustrative; use the real ones from the JAR:

```json
{
  "source_jar": "assets/tekkit/mods/<ic2 archive>.zip",
  "slices": [
    { "sheet": "ic2/sprites/block_0.png", "index": 18, "size": 16,
      "out": "content/assets/industrialcraft_2/textures/block/macerator_front.png",
      "for": "industrialcraft_2:macerator", "face": "front" }
  ],
  "copies": [
    { "from": "ic2/sprites/GUIMacerator.png",
      "out": "content/assets/industrialcraft_2/textures/gui/macerator.png" }
  ]
}
```

## Naming

- Textures: `textures/block/<name>[_<face>].png`, `textures/item/<name>.png`, `textures/gui/<machine>.png`, matching what the content-author's models reference.
- Keep source pixel size (16 px); do not upscale.
- Animated sheets (fluids, lava-like blocks) become a vertical strip PNG plus a `.mcmeta` next to it.
