---
name: parity-checklist
description: What the parity-critic checks when comparing a Rekkit mod port against its planning/mods/<id>.md spec. Use when reviewing a mod's diff before merge.
---

# parity-checklist

Compare the diff on `port/<id>` against `planning/mods/<id>.md`, `AGENTS.md` and `.agents/fleet-plan.md`. Every miss is a finding with file:line, severity, owner agent and fix.

## Registry

- [ ] Every block in the spec registry has `content/data/<id>/horizon/block/<name>.json`, a blockstate, a model, a lang key, a loot table, and a texture (or an `UNVERIFIED` entry in `asset-map.json`).
- [ ] Every item has its item JSON, item model, lang key and texture.
- [ ] Mineable tags exist for blocks that need a tool.

## Logic

- [ ] Each formula in the spec appears in `src/` with the same constants and units (EU/t, MJ/t, mB, ticks). Cite both lines.
- [ ] It behaves as the original did: check the cited Java (`assets/decompiled/`) where the spec is ambiguous, and flag a spec that misreads the Java (owner: spec-writer).
- [ ] Nothing in `src/`, `native/` or `tests/` is copied from the Java (clean-room): no Java method bodies, comments or obfuscated names.
- [ ] Each state machine has every state and transition from the spec.
- [ ] Each hazard is implemented or listed under DEFERRED with a reason.
- [ ] Shared concepts use `rekkit_core` types rather than local copies.
- [ ] Tests cover every formula, threshold and state machine.

## Recipes

- [ ] Every crafting, smelting and machine recipe in the spec has a JSON file; custom types use `"<id>:<machine>"` and are served by `native_recipes_of_type`.

## Hooks

- [ ] Every row of the spec's *Hooks needed* table is implemented or filed as a Beads bug labelled `horizon`.
- [ ] Every hook returns early for ids outside `<id>:` and never clears another mod's out-param.

## Assets (Recheck)

- [ ] Every texture and sound under `content/assets/` has an `asset-map.json` entry; originals are tagged `original` and `credits` names the original mod and authors.
- [ ] Every `generated` entry has its source in `art/`, the originals' size (16 px; effect icons 18) and palette, no anti-aliasing, and fills a gap the original never had (`make fleet-gaps`), never redraws one it had.
- [ ] `make check-mods MOD=<id> COMPLETE=1` and `make art MOD=<id> ARGS=--check` pass.

## Rules

- [ ] No `Cargo.toml` or `.rs` under `content/`. Nothing changed under `horizon/`.
- [ ] Each changed file is owned by the agent that changed it (fleet-plan §3).
- [ ] No `"replace": true` on shared tags.
- [ ] Versions match across the four lockstep files.

## Verdict

Any blocker → `VERDICT: fail`. Majors fail unless the spec marks the item deferred. Minors alone → `pass` with notes.
