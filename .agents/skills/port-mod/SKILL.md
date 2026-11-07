---
name: port-mod
description: Checklist for porting one Java mod (or the shared rekkit_core crate) to a Horizon dual-artifact Rust mod in Rekkit. Use for any scaffold, domain, content, test or release work on mods/<id>/ or crates/rekkit_core/.
---

# port-mod

What a builder (scaffolder, domain-builder, content-author, test-smith) needs. Your brief (`make fleet-task BEAD=<id> ROLE=<role>`) already holds the spec sections and Java methods your ticket names, your files with their templates and the contract format: work from it, and read anything else a section or a method at a time (`make spec-show`, `make java-show`). Write the block's or item's contract file (`planning/mods/<id>.contract/<name>.json`, the mod-spec skill) before building it.

## Layout of a mod

```text
mods/<id>/
├── Cargo.toml                         # domain crate, version = lockstep
├── horizon-registry.toml              # name, summary (<=160 chars), description, authors, license
├── asset-map.json                     # every texture/sound: slices and copies (origin original), generated (make art); credits; vanilla_ok
├── art/                               # sources of new art (SVG, .pixels) -> make art -> content/assets/<id>/...
├── src/lib.rs                         # pure Rust domain logic + HorizonNativeMod impl
├── tests/mod_tests.rs                 # unit + hook lifecycle + manifest tests
├── tests/e2e_actions.json             # harness action descriptors
├── content/                           # .hmod: data and assets only, never Cargo.toml or .rs
│   ├── horizon-mod.toml               # api = "0.2", side = "both", capabilities = ["registries"]
│   ├── data/<id>/horizon/{block,item,mob_effect,creative_tab}/*.json
│   ├── data/<id>/{recipe,loot_table/blocks}/*.json
│   └── assets/<id>/{blockstates,items,lang,models,textures,sounds}/
└── native/                            # .hnmod
    ├── horizon-native.json            # schema 1, game "26.3", pinned horizon + mixins revs, hooks 40
    └── mixins/<id>/{horizon-mixin.toml,hooks.rs}
```

Copy shapes from `mods/example_mod`; do not invent new layouts.

## The shared crate

`crates/rekkit_core/` is a plain rlib. It is **not** under `mods/`, because `scripts/pack_mod.py` and `scripts/e2e_harness.py` treat every `mods/*/Cargo.toml` as a full dual-artifact mod. Mods depend on it with:

```toml
rekkit_core = { path = "../../crates/rekkit_core" }
```

Use core types (energy units, stacks, tanks, grids, side config, multiblock patterns, frequency keys) instead of redefining them. If core lacks something, report `CORE-NEEDS` to the conductor; do not add a local copy.

## Commands

| What | Command |
|---|---|
| The spec sections you need (beyond your brief) | `make spec-toc MOD=<id>`, `make spec-show MOD=<id> SECTION="<heading>"` |
| Content from the contract (content-author) | `make content-gen MOD=<id> NAME=<a,b>`: blockstates, models, item models, loot tables, lang, creative tab |
| A draft of tests/e2e_actions.json (test-smith) | `make e2e-gen MOD=<id>` |
| One role's contract problems | `make contract-check MOD=<id> OWNER=<role>` |
| Unit tests for one crate | `cargo test --manifest-path mods/<id>/Cargo.toml` |
| Manifest + content validation | `make check-mods MOD=<id>` |
| Everything the stop gate checks | `make agent-gate MOD=<id>` (rustfmt, clippy -D warnings, tests, check-mods, harness dry run) |
| Harness pre-flight (test-smith) | `make e2e-mod MOD=<id> DRY_RUN=1` |

## Rules

- Block loot tables use Horizon's form, copied from `mods/example_mod/content/data/example_mod/loot_table/blocks/ender_chest.json`: a pool `"condition": { "type": ... }`, not vanilla's `"conditions": [...]` array, which drops nothing at the pinned Horizon (Beads bug `rk-yls`).
- A missing engine feature is a follow_up in your report titled `horizon: ...`.
- Start with `make fleet-task BEAD=<id> ROLE=<role>`, end with your report in `target/fleet/inbox/<bead>.json`.
