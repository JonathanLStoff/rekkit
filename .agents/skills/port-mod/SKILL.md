---
name: port-mod
description: Checklist for porting one Java mod (or the shared rekkit_core crate) to a Horizon dual-artifact Rust mod in Rekkit. Use for any scaffold, domain, content, test or release work on mods/<id>/ or crates/rekkit_core/.
---

# port-mod

The 11-step process is defined in `planning/workflow.md` and `.agents/fleet-plan.md` §4; progress lives in Beads (`make fleet-status MOD=<id>`). This skill is the short version every builder works from. Builders work from the spec only (clean-room): never from `assets/decompiled/`.

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
│   ├── data/<id>/horizon/{block,item,mob_effect}/*.json
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

The scripts need Python 3.11+ (`tomllib`). If `python3` is older, pass `PYTHON=$(scripts/fleet/python.sh)` to make.

| What | Command |
|---|---|
| Unit tests for one crate | `cargo test --manifest-path mods/<id>/Cargo.toml` |
| All crates | `make test-mods` |
| Manifest + content validation | `make check-mods MOD=<id>` (`COMPLETE=1`: also every asset 26.3 needs) |
| Syntax and checks of what you changed | `make agent-gate MOD=<id>` (rustfmt, clippy -D warnings, tests, check-mods, dry run) |
| New art | `make art MOD=<id>` |
| Look at it in the game | `make play MOD=<id>` |
| Harness pre-flight | `make e2e-mod MOD=<id> DRY_RUN=1` |
| Live isolated run | `make e2e-mod MOD=<id>` |
| Both phases | `make integration-test` |
| Package | `make pack-mods MOD=<id>` |
| Extract legacy assets | `make extract-jar JAR=assets/<pack>/mods/<jar> EXTRACT_OUT=assets/extracted/<id>` |

## Rules

- Block loot tables use Horizon's form, copied from `mods/example_mod/content/data/example_mod/loot_table/blocks/ender_chest.json`: a pool `"condition": { "type": ... }`, not vanilla's `"conditions": [...]` array, which drops nothing at the pinned Horizon (`planning/horizon-gaps.md`). The live E2E stage 7 catches a table that drops nothing.
- Versions move in lockstep across `Cargo.toml`, `content/horizon-mod.toml`, `native/horizon-native.json` and `native/mixins/<id>/horizon-mixin.toml`.
- Never edit `horizon/`. A missing engine feature is a follow_up in your report titled `horizon: ...`; the conductor files it as a Beads bug labelled `horizon`.
- Never `git push`, `gh release` or `minisign`. A human pushes; `.github/workflows/release-mods.yml` releases (`.agents/release.md`).
- Start with `make fleet-task BEAD=<id> ROLE=<role>`, end with your report in `target/fleet/inbox/<bead>.json`. Only the conductor writes to Beads.
- Only write paths you own (fleet-plan §3).

## Done

A mod is complete (ready for the human) when steps 1–9 are closed in Beads: `make agent-gate MOD=<id>` and `make check-mods MOD=<id> COMPLETE=1` pass, parity-critic's Check and Recheck say `VERDICT: pass`, and the live `make e2e-mod MOD=<id>` and `make e2e-all-mods` pass with every block placed (by /setblock and by a player, every variant) and no WARN/ERROR in the log, unless Horizon itself is broken at the pinned revision (`make horizon-status`).
