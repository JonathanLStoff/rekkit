# Rekkit agy Fleet Plan

How a fleet of agy agents ports every Java mod in Tekkit Classic 1.2.5, Direwolf20 1.4.7 and Agrarian Skies 2 to Horizon's Rust dual-artifact format. The fleet works on **one mod at a time**, in the order of `planning/fleet-board.md`, starting with the shared `rekkit_core` crate. Each mod follows the 10-step lifecycle in [`planning/workflow.md`](../planning/workflow.md).

Source of truth for the goal: `CLAUDE.md`, `planning/workflow.md`, `planning/harness.md`, `planning/priority.md`, `planning/{tekkit,dw20,agrarianskies2}_mods.md`, `planning/mods/*.md`.

**Start of every session:** follow `AGENTS.md` → *Start of every session*: check the setup (`./setup.sh --check`), tell the user word for word what they must download if a `!` line says so, then take the next board row and keep working.

---

## 1. Scope

| Bucket | Count | Status today |
|---|---:|---|
| Shared core crate `crates/rekkit_core` | 1 | First in the queue |
| Wave 1: priority mods with specs in `planning/mods/` | 10 | Specced, none ported |
| Wave 2: Tekkit addons (no spec yet) | 14 | Queued |
| Wave 3: Direwolf20 extras (no spec yet) | 21 | Queued |
| Wave 4: Agrarian Skies 2 (1.7.10) | ~95 manifest mods | Triage first |
| Libraries / client QoL | 8 from Tekkit + DW20 | Not ported as mods |

Only `mods/example_mod` exists (an Ender Chest + Ender Pouch demo). The live `horizon-ctl` E2E run has not been done yet (`planning/todo.md`).

---

## 2. Fleet shape

```
 trigger ── agy TUI (/agents → conductor) · scripts/fleet/run-next.sh · /schedule
    │
 Conductor (pro, high) ── reads planning/fleet-board.md, takes the ONE active or next queued row
    │
 branch port/<id>
    │   Spec → Scaffold → [Domain | Content | Assets | Hooks | Tests] → Gate → Release Clerk
    │                       (parallel inside the mod, each in its own subagent worktree)
    │
 merge to main ── Compat Auditor: make integration-test (Phase 1 this mod + Phase 2 all mods)
    │
 YOU ── git push · minisign · GitHub Release
    │
 next row
```

**One mod at a time.** The conductor never opens a second mod while one is `active`. Parallelism happens only inside the current mod, where the five builders own disjoint paths. This keeps Argon quota spending predictable, keeps `main` always green, and means every later mod builds on a released, reviewed predecessor. `scripts/fleet/run-next.sh` takes a lock so two headless runs cannot overlap.

---

## 3. Roster

| Agent | Lifecycle step | Model | Workspace | Owns (may write) |
|---|---|---|---|---|
| **conductor** | orchestrates | `pro` | main checkout | `planning/fleet-board.md`, `planning/priority.md`, `planning/todo.md`, `planning/completed.md`, merges |
| **scout** | context | `flash` | `inherit` | nothing (read-only report) |
| **spec-writer** | 1 | `pro` | `branch` | `planning/mods/<id>.md` |
| **scaffolder** | 2 | `flash` | `branch` | `mods/<id>/Cargo.toml`, `horizon-registry.toml`, `content/horizon-mod.toml`, `native/horizon-native.json`, `native/mixins/<id>/horizon-mixin.toml` (for `rekkit_core`: `crates/rekkit_core/Cargo.toml` and the `test-mods` Makefile target) |
| **domain-builder** | 2–3 | `flash` | `branch` | `mods/<id>/src/**` (Standard tier) |
| **domain-builder-pro** | 2–3 | `pro` | `branch` | `mods/<id>/src/**` (Hard tier, and `crates/rekkit_core/src/**`) |
| **content-author** | 2, 5 | `flash` | `branch` | `mods/<id>/content/data/**`, `content/assets/<id>/{lang,models,blockstates,items}/**` |
| **asset-smith** | 4–5 | `flash` | `branch` | `assets/extracted/<id>/` (gitignored), `mods/<id>/asset-map.json`, `content/assets/<id>/{textures,sounds}/**` |
| **hook-engineer** | 2 | `pro` | `branch` | `mods/<id>/native/mixins/<id>/*.rs`; append-only `planning/horizon-gaps.md` |
| **test-smith** | 3, 7 | `flash` | `branch` | `mods/<id>/tests/**` incl. `tests/e2e_actions.json` (or `crates/rekkit_core/tests/**`) |
| **parity-critic** | 6 (gate) | `pro` | `inherit` | nothing (read-only verdict) |
| **compat-auditor** | 7 (gate) | `flash` | main, after merge | nothing (runs the harness, reports) |
| **release-clerk** | 8–10 | `flash` | `branch` | version fields in the 4 lockstep files; local commit; `make pack-mods` |
| **scribe** | docs | `flash` | `share` | `mods/<id>/README.md`, mod table in `README.md` |
| **Stop hook** | gate | none | — | `scripts/fleet/tests-gate.sh` |

Effort: run the conductor at `/effort high`. Frontmatter has no effort field, so `pro` subagents run Argon's default (High) and `flash` subagents run 3.8 Flash's default (Medium).

Free/Plus plans have no Argon: set every `pro` to `flash`.

---

## 4. The pipeline for the current mod

The conductor creates branch `port/<id>` and sets the board row to `active`. A single agent without agy plays every role below in the same order.

1. **Spec** (step 1). If `planning/mods/<id>.md` exists, spec-writer gap-checks it against the JAR and adds `Hooks needed`, `Uses from rekkit_core` and `Open questions` sections. Otherwise it writes the full spec in the structure of the existing specs.
2. **Scaffold** (step 2). Same shape as `mods/example_mod`; version `0.1.0` in all four lockstep files; same pinned `horizon`/`mixins` revisions; path dependency `rekkit_core = { path = "../../crates/rekkit_core" }`.
3. **Parallel build** (steps 2–5). Domain, content, assets, hooks and tests are invoked together. Their paths never overlap.
4. **Gate** (step 6). The Stop hook runs on every finish attempt. Then parity-critic compares the diff to the spec. Findings go to the **idle owner of that file** by message.
5. **Merge and compat** (step 7). Conductor merges `port/<id>` into main; compat-auditor runs `make integration-test DRY_RUN=1`, and the live `make integration-test` once the client works.
6. **Release prep** (step 8). Release-clerk bumps the lockstep versions, commits `feat(<id>): …` locally, runs `make pack-mods MOD=<id>`, then stops.
7. **Human** (steps 9–10). You push, sign with `minisign`, publish the GitHub Release.
8. **Close out.** Scribe updates the README; conductor sets the row to `done`, moves it to `planning/completed.md`, and stops. The next mod starts on the next run.

A mod is **done** when `cargo test` passes for it, `make check-mods MOD=<id>` passes, `make e2e-mod MOD=<id>` passes (dry-run until the live client works), parity-critic says `VERDICT: pass`, and both `.hmod` and `.hnmod` are built.

If a needed engine feature is missing, hook-engineer logs it in `planning/horizon-gaps.md`, the feature is deferred in the spec, and the mod still finishes.

---

## 5. Queue

`planning/fleet-board.md` holds the exact order. Summary:

### Step 0 — Foundation (not a mod)

- `./setup.sh` sets up a fresh clone: tools check, submodule, mod tests, the three modpacks, the Horizon build with every mod's mixins, game resources (from the user's own Minecraft 26.3) and a live `example_mod` smoke test. It names the steps only a person can do.
- Fleet files: `AGENTS.md`, `.agents/agents/*`, `.agents/skills/*`, `.agents/hooks.json`, `scripts/fleet/*`, the board and the gaps log. **Created.**

### 1 — `rekkit_core` (shared crate)

Lives at `crates/rekkit_core/`, **not** under `mods/`: `scripts/pack_mod.py` and `scripts/e2e_harness.py` treat every `mods/*/Cargo.toml` as a full dual-artifact mod, so a library there would fail `check-mods`.

- Spec: spec-writer surveys all ten `planning/mods/*.md` and writes `planning/mods/rekkit_core.md`, listing each shared concept and which mods use it.
- A concept goes into core only if **two or more** queued mods need it. Candidates:
  - energy: EU tiers and packets with over-voltage explosion thresholds, MJ, Blutricity, conversion ratios (Power Converters)
  - item stacks, inventories, slot rules; fluid tanks in mB
  - connected grids: union-find membership and BFS routing (cables, pipes, tubes, ME cable)
  - side configuration (`SideMode`) and auto-output
  - multiblock pattern validation (Coke Oven, Blast Furnace, Alveary, MAC)
  - frequency keys (EnderStorage, WR-CBE, Tesseract)
  - deterministic tick and RNG helpers, serde persistence helpers
- Lifecycle steps that apply: 1, 2 (crate only), 3, 6 (`cargo test`, `cargo clippy`), 8, 9. Steps 4, 5, 7 and 10 are `—`: no content, native bundle, E2E or release artifacts.
- Scaffolder extends `make test-mods` to also run `crates/*/Cargo.toml`.
- Later mods that need something new in core get a small `rekkit_core` bump as its own board row, inserted before that mod.

### 2–11 — Wave 1, in order

| # | Mod | Tier | Note |
|---:|---|---|---|
| 2 | redpower_2 | Hard | microblocks, wires, logic gates, tubes, frames, 6502. Settle how hooks call domain code here (§10); `example_mod`'s `list_mod.rs` shows a working hook. |
| 3 | buildcraft | Hard | MJ, pipes, quarry, engines, gates |
| 4 | industrialcraft_2 | Hard | EU grid, reactor, machines |
| 5 | equivalent_exchange_2 | Hard | EMC solver over every recipe merged so far |
| 6 | computercraft | Hard | embedded Lua VM, turtles |
| 7 | applied_energistics | Hard | ME network, crafting DAG |
| 8 | railcraft | Hard | multiblocks, carts, signals |
| 9 | thermal_expansion | Standard | side config, tesseracts |
| 10 | forestry | Hard | bee genetics, multiblock farms |
| 11 | enderstorage | Standard | `example_mod` already implements most of it |

This is `planning/priority.md`'s order. If they ever disagree, `priority.md` wins.

Escalate one stuck subsystem (6502 emulator, Lua sandbox, EMC solver, reactor sim, AE crafting DAG) to `/boost` when a builder reports `blocked` twice.

### 12–25 — Wave 2, Tekkit addons

Additional Pipes (→ BuildCraft), Advanced Machines, Charging Bench, Compact Solars, IC2 Nuclear Control, MFFS (all → IC2), Power Converters (→ BuildCraft, IC2, RedPower 2), ccSensors (→ ComputerCraft), Tubestuff (→ RedPower 2), WR-CBE, Dimensional Anchors, Iron Chests, Nether Ores, Balkon's WeaponMod.

### 26–46 — Wave 3, Direwolf20 extras

Charge Pads, Gravitation Suite, Modular Powersuits (→ IC2), OmniTools, Misc Peripherals (→ ComputerCraft), Extra Bees (→ Forestry), Thaumcraft 3, Thaumic Bees (→ Thaumcraft 3, Forestry), Equivalent Exchange 3, Factorization, Steve's Carts 2, Mystcraft and Twilight Forest (likely engine gap: dimensions), ExtrabiomesXL, ChickenChunks, Gravity Gun, Portal Gun, Soul Shards, Obsidian Pressure Plates, Xeno's Reliquary, XyCraft.

### 47 — Wave 4, Agrarian Skies 2 triage

`make agrarian` (works without a CurseForge key; see README). Scout + spec-writer sort every manifest mod into **already ported** (1.7.10 deltas go in a spec appendix), **port**, **library** or **client/pack QoL**, writing `planning/agrarianskies2_triage.md`. You approve the port list; the conductor then appends those mods to the board.

### Not ported as mods

Minecraft Forge, CodeChickenCore, Immibis Core (libraries: port only what a consumer needs), Not Enough Items, NEI Plugins (Horizon feature request), Inventory Tweaks, Rei's Minimap, MAtmos (client QoL, out of scope).

---

## 6. Ownership rules

- Agents write only the paths they own (§3). Anything else is reported to the conductor.
- Shared files (`Makefile`, `scripts/*.py`, root `README.md`, planning ledgers, the board) change only on main, by the conductor or scribe. One exception: the `rekkit_core` scaffolder edits the `test-mods` target.
- `horizon/` is a read-only git submodule. `scripts/fleet/guard-paths.sh` denies writes there.
- Mods depend on `rekkit_core` and on earlier mods by path; a mod never edits another mod's crate.

---

## 7. Fleet board

`planning/fleet-board.md`, owned by the conductor. `State` is `queued`, `active`, `blocked` or `done`; there is at most one `active` row. Step cells are `·` todo, `▶` in progress, `✓`, `✗` failed, `—` n/a.

`scripts/fleet/run-next.sh` resumes the `active` row, otherwise starts the first `queued` row whose deps are all `done`.

---

## 8. Files

```
rekkit/
├── AGENTS.md
├── .agents/
│   ├── fleet-plan.md
│   ├── agents/<14 agents>/agent.md
│   ├── skills/
│   │   ├── extract-jar-assets/SKILL.md
│   │   ├── port-mod/SKILL.md
│   │   ├── horizon-hooks/SKILL.md
│   │   ├── mod-spec/SKILL.md
│   │   ├── legacy-asset-mapping/SKILL.md
│   │   └── parity-checklist/SKILL.md
│   └── hooks.json
├── planning/fleet-board.md
├── planning/horizon-gaps.md
└── scripts/fleet/
    ├── tests-gate.sh      # Stop hook: cargo test + check-mods + e2e dry run for touched mods/crates
    ├── guard.sh           # PreToolUse: no push/release/sign/submodule/force/rm -rf
    ├── guard-paths.sh     # PreToolUse: no writes under horizon/, no code under content/
    ├── python.sh          # prints a Python ≥ 3.11 (scripts need tomllib)
    └── run-next.sh        # headless: one conductor run on one mod, with a lock
```

---

## 9. Running it

**Interactive**
1. `agy` in the repo → `/agents` → **conductor**.
2. `/model` → Gemini 4 Argon, `/effort high`.
3. "Continue the board." The conductor resumes the active row or starts the next one.
4. `Alt+J` jumps to the next approval, `Ctrl+K` approves inline, `K` kills a runaway.
5. Talk to one worker: `@hook-engineer also guard native_update_shape on redpower_2:wire`.

**Headless**
- `scripts/fleet/run-next.sh` (next row) or `scripts/fleet/run-next.sh rekkit_core` (a specific row).
- `/schedule` a nightly `scripts/fleet/run-next.sh`. It exits early if a run is already going or the active row is waiting on you.

---

## 10. Human checkpoints and open decisions

| When | You do |
|---|---|
| After `rekkit_core` spec | Approve what goes into core |
| After each mod's release prep | Review diff, `git push`, `minisign -S`, publish the GitHub Release |
| New `planning/horizon-gaps.md` entry | Upstream a Horizon PR, work around it, or defer |
| Wave 4 triage done | Approve which Agrarian Skies mods get ported |

Open decisions:
1. **How native hooks reach domain logic.** `example_mod/native/mixins/example_mod/hooks.rs` redeclares constants instead of calling the domain crate. Decide during `redpower_2` whether mixin bundles can depend on the crate (and on `rekkit_core`) or get it vendored in at pack time.
2. **Version era.** BuildCraft, Forestry and Thermal Expansion appear in two or three packs. Recommended: one crate per mod, the earliest pack is canonical, later behavior in a spec appendix behind a config flag.
3. **Legacy textures.** 1.2.5/1.4.7 mods ship sprite sheets. Asset-smith records slices in `mods/<id>/asset-map.json`; a shared slicing script under `scripts/` would make that reproducible.
4. **Client QoL mods.** Confirm NEI, Inventory Tweaks, minimaps and MAtmos stay out of scope.

Resolved: the shared crate exists and goes first, at `crates/rekkit_core`.

---

*agy specifics (agent frontmatter fields, tool names, `hooks.json` shape, `/boost`, model names) follow the Antigravity docs as of 31 Oct 2026 and have not been tested against a live agy install. Tool names in `tools:` must match exactly; an unknown name can hang a subagent.*
