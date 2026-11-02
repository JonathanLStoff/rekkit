---
name: mod-spec
description: Template and rules for writing planning/mods/<id>.md specifications for Rekkit ports. Use when writing a new mod spec, gap-checking an existing one, or writing the rekkit_core spec.
---

# mod-spec

Existing specs (`planning/mods/redpower_2.md`, `equivalent_exchange_2.md`, …) set the format. New specs use the same section order and add three sections at the end.

## Sections

1. **Overview & Version Target**: every version of the mod in our packs (version, pack, Minecraft version, archive under `assets/<pack>/`), the decompiled folder of each (`assets/decompiled/<archive>/`), and the original's **authors and licence**.
2. **Core systems & mechanics**: one subsection per system, with exact formulas, units (EU/t, MJ/t, mB, ticks), thresholds and state machines.
3. **Machines / devices**: per block: purpose, inputs/outputs, energy use, GUI layout (slot indexes and positions), redstone behavior.
4. **Tools, armor, items**: per item: behavior, durability or charge, special effects.
5. **World generation & resources**.
6. **Complete registry**: tables of every block and item with the Rekkit id `<id>:<name>`.
7. **Recipes**: every crafting, smelting and machine recipe, with custom machine recipe type names (`"<id>:<machine>"`).
8. **Hazards**: explosions, meltdowns, radiation, griefing vectors, with trigger conditions.
9. **Recreation notes for Rekkit (Rust)**.
10. **Hooks needed**: table of Horizon API level → entrypoint → feature (see horizon-hooks skill).
11. **Uses from rekkit_core**: the core types this mod relies on, and anything core lacks.
12. **Open questions**: what could not be verified.

## Rules

- Work from the decompiled code (`make decompile JAR=<archive>`, the client build when there is one), its configs and its lang files first; mark anything else as **UNVERIFIED**.
- Read the Java in pieces: `make java-outline DIR=assets/decompiled/<archive> ARGS=--classes` for the map, `GREP=<name>` for one class's members, `make java-show ... MEMBER=<Class#method>` for the code a section needs, `make java-find ... TEXT=<name>` to trace a value. Write each section as soon as you have read for it.
- **Clean-room.** The builders implement from this spec alone and never see the Java. Describe behaviour in your own words, tables and formulas, and cite where it comes from (`TileAlloyFurnace.updateEntity`, `RedPowerMachine.java:212`), but never paste Java code, comments or method bodies.
- Every formula, constant, threshold, tick rate and table cites its class and method.
- 1.2.5 and 1.4.7 mods use numeric block/item ids and damage values. Record the original id:meta next to each registry row so assets and recipes can be traced.
- **Several versions: merge, newer wins.** When a mod ships in more than one pack (RedPower 2.0pr5b2 in Tekkit 1.2.5 and 2.0pr6 in Direwolf20 1.4.7), the port is the union of every version: each block, item, recipe and system any version has. Where versions share a feature, the newest version's behaviour, numbers and art win (both have the alarm: the 1.4.7 alarm). A feature only an older version has stays, as that version made it (1.2.5's microblocks stay even if 1.4.7 dropped them). Decompile every version. Tag each registry row, system and recipe with the version it comes from (`[2.0pr6]`), and add a **Versions** table: feature → versions that have it → version used. Builders port one merged mod, never one per version.
- **Uses from rekkit_core**: what the mod takes from the core, and what the core lacks. Something belongs in the core when another ported or queued mod needs it too (name those mods); the conductor files each lack with `make core-need`. What only this mod uses stays in the mod.

## Feature list and contract

End step 1 with `planning/mods/<id>.features.json`: the conductor's source for the feature beads (`make fleet-features MOD=<id>`) and the **contract**, what the original is, block by block and item by item, so a port that only places, opens a stock grid and drops cannot pass. `make contract-check MOD=<id>` (and `check-mods COMPLETE=1`) compares the port's models and textures with it pixel for pixel; the live harness checks its GUIs, shapes and scenarios. The builders never see the Java: everything below comes from you, read from the original's code, never guessed. One entry per registry row and per subsystem, ids as in section 6:

```json
{
  "blocks": [{
    "id": "alloy_furnace", "name": "Alloy Furnace", "java": "BlockAppliance meta 0",
    "shape": "full",
    "facing": "horizontal",
    "faces": {"top": "eloraam/base/base1.png#19", "front": "eloraam/base/base1.png#17",
              "side": "eloraam/base/base1.png#16", "bottom": "eloraam/base/base1.png#16"},
    "gui": {"size": [176, 166], "title": "Alloy Furnace",
            "slots": [[48, 17], [66, 17], [84, 17], [48, 35], [66, 35], [84, 35], [48, 53], [66, 53], [84, 53], [17, 42], [141, 35]]},
    "energy": null
  }, {
    "id": "and_gate", "name": "AND Gate", "java": "BlockLogic meta 1 (RenderLogic)",
    "shape": [0, 0, 0, 16, 2, 16], "facing": "horizontal",
    "faces": {"top": "eloraam/logic/logic1.png#2", "side": "eloraam/logic/logic1.png#0"}, "gui": null
  }],
  "items": [{"id": "voltmeter", "name": "Voltmeter", "java": "ItemVoltmeter",
             "sprite": "eloraam/base/items1.png#24", "max_stack": 1, "durability": null,
             "use": "right-click a blutricity block: chat 'Reading %.2fV %.2fA (%.2fW)'"}],
  "features": [{"id": "blutricity_grid", "name": "Blutricity power network", "spec": "3.5"}],
  "scenarios": [{"name": "solar panel charges in daylight", "for": ["solar_panel", "voltmeter"], "steps": [
    {"command": "time set noon"}, {"setblock": [0, 0, 0, "redpower_2:solar_panel"]}, {"wait": 200},
    {"use": "redpower_2:voltmeter", "on": [0, 0, 0]},
    {"expect_message": "Reading (?!0\\.00V)\\d+\\.\\d\\dV"}]}]
}
```

Every block needs `shape`, `facing`, `faces` and `gui`; every item `sprite` and `max_stack`. Where to find each in the original:

- **shape**: `"full"`, or the outline of its default state as `/setblock` places it, `[x1, y1, z1, x2, y2, z2]` in pixels (0-16), from `setBlockBounds` (a gate or plate `[0,0,0,16,2,16]`, a wire, a panel). Plants: `"cross"`.
- **facing**: `"horizontal"` or `"all"` when it turns to the player as placed (`onBlockPlacedBy` storing a direction), `"none"` otherwise.
- **faces**: the sprite on each face as placed facing north: `top`, `bottom`, `front` (north), `back`, `left`, `right`, `side` (every side not named), `all`; from `getBlockTextureFromSideAndMetadata`/`getBlockTexture` and the block's texture file. A sprite is `"<sheet path in the archive>#<index>"` (index = row × 16 + column on a 16-wide sheet), as in `asset-map.json`'s slices; `"minecraft:block/<name>"` when the original reuses a vanilla texture. Name every face the original draws, also on a model drawn by code (`RenderLogic`, a TESR): its base, plate or top sprite.
- **gui**: `null` when right-clicking opens nothing; else its `Gui*` class's `xSize`/`ySize` as `size`, and the position of each of the block's own slots in its `Container*` (the `new Slot(inv, n, x, y)` calls, without the player's inventory), in slot order.
- **energy**: `null`, or `{"role": "producer|consumer|storage|conductor"}`.
- **items**: `sprite` from `setIconIndex`/`iconIndex` and the item's texture file (a list for subtypes: one sprite each); `max_stack` from `setMaxStackSize` (64 if never set); `durability` from `setMaxDamage` (null if none); `use` what right-clicking does, in your words.
- **scenarios**: one at least for every machine, generator, tool and item that does something a player sees: what goes in, how long, and what the player can read afterwards (a chat line, a block state, `data get block`). Steps: `setblock [dx,dy,dz,block]`, `command`, `wait` (ticks), `use` (an item on a block `on [dx,dy,dz]`), `interact [dx,dy,dz]`, and checks `expect_message` (regex on chat and the action bar), `expect_block [dx,dy,dz,regex]` (its state), `expect_data [dx,dy,dz,regex]`. Write the expectation from the original's numbers and messages, never from the port.
