---
name: mod-spec
description: Template and rules for writing planning/mods/<id>.md specifications for Rekkit ports. Use when writing a new mod spec, gap-checking an existing one, or writing the rekkit_core spec.
---

# mod-spec

Existing specs (`planning/mods/redpower_2.md`, `equivalent_exchange_2.md`, …) set the format. New specs use the same section order and add three sections at the end.

## Sections

1. **Overview & Version Target**: mod version, pack(s) it ships in, Minecraft version, JAR filename under `assets/<pack>/mods/`.
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

- Work from the actual JAR, its configs and its lang files first; mark anything else as **UNVERIFIED**.
- 1.2.5 and 1.4.7 mods use numeric block/item ids and damage values. Record the original id:meta next to each registry row so assets and recipes can be traced.
- When a mod appears in more than one pack, the earliest pack's version is canonical. Put later-version differences in an appendix.
- For `rekkit_core`, list each shared concept with the mods that need it. A concept needs at least two.
