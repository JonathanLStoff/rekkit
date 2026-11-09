# Rekkit Fleet Plan

How a fleet of agents (agy, Claude Code, or a single agent in any other tool) ports every Java mod in Tekkit Classic 1.2.5, Direwolf20 1.4.7 and Agrarian Skies 2 to Horizon's Rust dual-artifact format. The fleet works on **one mod at a time**, in the order of the epics in Beads (shown in `planning/fleet-board.md`), starting with the shared `rekkit_core` crate. Each mod follows the 11-step process in [`planning/workflow.md`](../planning/workflow.md) §2.

Source of truth for the goal: `AGENTS.md`, `planning/workflow.md`, `planning/harness.md`, `planning/priority.md`, `planning/{tekkit,dw20,agrarianskies2}_mods.md`, `planning/mods/*.md`.

**Start of every session:** follow `.agents/start.md`: check the setup (`./setup.sh --check`), tell the user word for word what they must download if a `!` line says so, then take the next mod in Beads (`make fleet-status`, `bd ready --type epic`) and keep working.

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
 trigger ── agy (/agents → conductor) · Claude Code (/conductor) · "Follow AGENTS.md" · make fleet-next · /schedule
    │
 Conductor (pro, high) ── the ONE epic in progress in Beads, else the next ready one (bd ready)
    │   make fleet-start MOD=<id>: claim · 11 stories · Horizon built and run alone (make horizon-status)
 branch port/<id>
    │   1 Spec (from the decompiled Java) ──────────────┐
    │   2 Scaffold → 3 Code [domain | hooks | content | tests] → 4 Check
    │                                       5 Extract assets (beside 2-4)
    │   6 New assets (only what the original never had) → 7 Recheck → 8 Harness (live) → 9 Package
    │   every subagent: make fleet-task → work → report in target/fleet/inbox/ → conductor: make fleet-inbox
    │
 DONE, CHECK OUT THE PROGRESS: make play MOD=<id>
    │
 YOU ── play it · review · git push (the release workflow signs and publishes a pre-release)
    │
 next epic
```

**One mod at a time.** The conductor never starts a second epic while one is in progress. Parallelism happens only inside the current mod, where the builders own disjoint paths. This keeps model quota spending predictable, keeps `main` always green, and means every later mod builds on a reviewed predecessor. `make fleet-next` takes a lock so two headless runs cannot overlap.

**Progress lives in Beads** (`bd`, `.beads/`): an epic per mod, a story per step, a feature per block, item, subsystem or missing asset, a bug per finding. Only the conductor writes to it; `make board` regenerates `planning/fleet-board.md` and `.beads/issues.jsonl` for GitHub, and `bd dolt push` (the git pre-push hook) syncs the database when a person pushes.

---

## 3. Roster

| Agent | Steps | Thinking | Reads the Java | Owns (may write) |
|---|---|---|---|---|
| **conductor** | all | the session | yes | Beads (the only writer), `planning/fleet-board.md` (`make board`), `planning/priority.md`, `planning/todo.md`, merges |
| **scout** | context | low | yes | nothing (read-only report) |
| **spec-writer** | 1 | medium | yes | `planning/mods/<id>.md`, `planning/mods/<id>.features.json`, `planning/mods/<id>.parts/`, `planning/mods/<id>.tickets/` |
| **scaffolder** | 2 | low | after the spec | `mods/<id>/Cargo.toml`, `horizon-registry.toml`, `content/horizon-mod.toml`, `native/horizon-native.json`, `native/mixins/<id>/horizon-mixin.toml` (for `rekkit_core`: `crates/rekkit_core/Cargo.toml` and the `test-mods` Makefile target) |
| **domain-builder** | 3 | medium | after the spec | `mods/<id>/src/**` (Standard tier) |
| **domain-builder-pro** | 3 | high | after the spec | `mods/<id>/src/**` (Hard tier, and `crates/rekkit_core/src/**`) |
| **hook-engineer** | 3 | medium | after the spec | `mods/<id>/native/mixins/<id>/*.rs` |
| **content-author** | 3 | low | after the spec | `mods/<id>/content/data/**`, `content/assets/<id>/{lang,models,blockstates,items}/**` |
| **test-smith** | 3 | medium | after the spec | `mods/<id>/tests/**` incl. `tests/e2e_actions.json` (or `crates/rekkit_core/tests/**`) |
| **parity-critic** | 4, 7 | high | yes | nothing (findings in its report) |
| **asset-smith** | 5, 6 | low | no need | `assets/extracted/<id>/` (gitignored), `mods/<id>/asset-map.json`, `mods/<id>/art/**`, `content/assets/<id>/{textures,sounds}/**` |
| **compat-auditor** | 8 | low | no need | nothing (runs the harness, reports) |
| **release-clerk** | 9 | low | no need | version fields in the 4 lockstep files; local commit; `make pack-mods` |
| **scribe** | docs | low | no need | `mods/<id>/README.md`, mod table in `README.md` |
| **Hooks** | all | none | — | `scripts/fleet/guard.py`: shell, paths and read guards (no whole decompiled class, no folder sweeps or toolchain reads, nothing read twice; a subagent wraps up at 60 turns, 500k tokens billed or 200k of context); the stop gate; the Beads context (`bd prime`, the resume page) at session start |

"Reads the Java" = may open `assets/decompiled/`. Every role may now, after the spec: the column says who needs it.

Models: every role runs on the model the session is on (the person picks it by starting the session on it); a role sets only its thinking level, `effort:` in its role file (Claude Code: the subagent's `effort`; headless agy and claude runs: `--effort`). Why: planning/token_audit/fix-plan.md.

---

## 4. The process for the current mod

The 11 steps, what each needs and when it is done: [`planning/workflow.md`](../planning/workflow.md) §2. The conductor's procedure: `.agents/agents/conductor/agent.md`. In short:

1. **Spec from the Java**: decompile, describe the behaviour with citations, list every block, item and subsystem (`<id>.features.json` → `make fleet-features`).
2. **Scaffold**, then 3. **Code** in parallel (domain, hooks, content, tests), from the spec only.
4. **Check**: `make agent-gate` + parity-critic (the Rust against the spec and the Java). Findings are bugs back to their owners until clean.
5. **Extract assets** (beside 2–4): the original's art and sound from a client build, tagged `original`, credited.
6. **New assets**: only what 26.3 needs and the original never had (`make fleet-gaps`), drawn in `mods/<id>/art/`, built by `make art` at the original size and palette.
7. **Recheck**: everything again on the whole mod.
8. **Harness (live)**: every block placed by `/setblock` and by a player, every variant, all 7 stages, no warnings. Horizon's own failures (it fails the same with no mods) are filed as `horizon` bugs and do not hold the mod up.
9. **Package**: version, commit (with the board), pack. Then `DONE, CHECK OUT THE PROGRESS: make play MOD=<id>`.
10. **Human**: play, review, push. 11. **Own art**: a later pass; full releases need it (`.agents/release.md`).

An agent whose tool has no subagents plays every role itself, in the same order, and still writes each role's report to the inbox and files it.

If a needed engine feature is missing, it becomes a Beads bug labelled `horizon`, the feature is deferred in the spec, and the mod still finishes.

---

## 5. Queue

The epics' priorities and dependencies in Beads hold the exact order (`bd ready --type epic`, `planning/fleet-board.md`). Summary:

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
- Later mods that need something new in core get a `rekkit_core` story for it, done before that mod continues.

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
- `horizon/` is a read-only git submodule. The paths guard (`scripts/fleet/guard.py paths`, agy and Claude Code hooks) denies writes there, and to `planning/harness-allowed-warnings.txt` (a person's list).
- Builders never read `assets/decompiled/` (the read guard denies it); only the spec-writer, parity-critic, scout and conductor do.
- Only the conductor writes to Beads; a role subagent that tries is told to put it in its report.
- Mods depend on `rekkit_core` and on earlier mods by path; a mod never edits another mod's crate.

---

## 7. Progress: Beads and the board

| In Beads | Is | Labels |
|---|---|---|
| epic | a mod (47, from the old board, in its order; blocked by the epics of the mods it needs) | `mod`, `mod:<id>`, `wave:<w>`, `tier:<t>`, `path:<path>`, `needs:<id>` |
| story | a step of §4, under the epic, blocked by the steps before it; step 11 deferred | `step:<n>` |
| feature | a block, item or subsystem (Code), the assets of one (Extract assets), a missing asset (New assets), a block's harness result (Harness) | `block`, `item`, `subsystem`, `asset`, `new-asset`, `harness` |
| bug | a review finding or harness failure, under the step it blocks; an engine gap | `horizon` for engine gaps |

- `make fleet-status [MOD=<id>]`: where a mod stands; `bd ready`: what can be worked now; `bd show <id>`: one bead and its comments.
- `planning/fleet-board.md` is generated (`make board`): one row per mod, a column per step with its state and `done/total` of its features. Never edit it by hand.
- A mod is in progress when its epic is `in_progress`; finished for the agents when stories 1–9 are closed (or the epic is `blocked`). `make fleet-next` resumes it, else starts the next ready epic.

---

## 8. Files

```
rekkit/
├── AGENTS.md                # every tool's instructions; CLAUDE.md and GEMINI.md import it
├── .beads/                  # Beads (bd): the progress database; issues.jsonl is its export for GitHub
├── .agents/                 # the source: edit here, then `make agents`
│   ├── fleet-plan.md
│   ├── release.md           # pre-release while original assets remain; full release with our own
│   ├── agents/<14 agents>/agent.md
│   ├── skills/
│   │   ├── extract-jar-assets/SKILL.md
│   │   ├── port-mod/SKILL.md
│   │   ├── horizon-hooks/SKILL.md
│   │   ├── mod-spec/SKILL.md
│   │   ├── legacy-asset-mapping/SKILL.md
│   │   └── parity-checklist/SKILL.md
│   └── hooks.json           # agy hooks
├── .claude/                 # generated by `make agents`: agents/, skills/, settings.json hooks
├── planning/fleet-board.md          # generated from Beads (make board)
├── planning/harness-allowed-warnings.txt   # the only accepted game-log warnings; a person edits it
└── scripts/fleet/
    ├── guard.py           # guards (shell, paths, read, task budget), the stop gate, bd prime, fleet-start/task/stop
    ├── beads.py           # the Beads layer: start, features, gaps, inbox, board, seed
    ├── hook.sh            # what the hooks run: finds Python, calls guard.py
    ├── sync_agents.py     # writes each tool's copy of .agents/ (make agents / agents-check)
    ├── run_next.py        # headless: one conductor run on one mod with any agent CLI, with a lock
    ├── run-next.sh        # the same with AGENT=agy by default (older schedules)
    └── python.sh          # prints a Python ≥ 3.11 (scripts need tomllib)
```

---

## 9. Running it

**Interactive, agy**
1. `make agy` in the repo (agy with its terminal sandbox off: the port needs the network, Beads and the Minecraft install; the hooks still guard every command) → `/agents` → **conductor**.
2. `/model` → Gemini 4 Argon, `/effort high`.
3. "Continue the board." The conductor resumes the mod in progress or starts the next ready one.
4. `Alt+J` jumps to the next approval, `Ctrl+K` approves inline, `K` kills a runaway.
5. Talk to one worker: `@hook-engineer also guard native_update_shape on redpower_2:wire`.

**Interactive, Claude Code**
1. `claude` in the repo; `CLAUDE.md` loads `AGENTS.md`, `.claude/` holds the roles, skills and hooks.
2. `/conductor` (or "Continue the board."). The main session conducts and starts the roles as subagents.

**Interactive, any other tool** (Codex, Cursor, Copilot, Gemini CLI…)
1. Open the repo and say **"Follow AGENTS.md."** The agent conducts and plays each role itself (or through its tool's subagents, given the role file as the prompt).
2. Nothing blocks a forbidden command in these tools: the *Rules* in `AGENTS.md` and `make agent-gate` before stopping are the guard.

**A run does not stop until its mod is finished**
- The conductor runs `make fleet-start MOD=<id>` when it takes the mod. The hooks tie that session to the mod, and from then on the Stop hook (agy and Claude Code) sends it back to work at every turn end until the mod is finished in Beads: stories 1–9 closed, or the epic `blocked` with the reason as a comment. It also holds the conductor while subagents are still running or their reports are unfiled, and holds each role subagent until it has written its report.
- It lets the agent stop when you cancel it (agy `USER_CANCELED`; Claude Code never runs the hook on an interrupt), after `make fleet-stop` (any tool, any terminal), or after 10 turn ends in a row with no change in Beads or the inbox (stuck).
- Headless runs (`make fleet-next`) relaunch the agent with "continue from step N" whenever it exits early, for any tool, until the mod is finished, `make fleet-stop`, or 3 runs in a row change nothing in Beads.
- **Checkpoints.** The hooks keep everything a successor needs, whatever kills a session (crash, credits, network): each session's last tool call, a journal of commands, writes and subagent launches (`target/fleet/journal.jsonl`), every subagent's prompt under its bead (`target/fleet/prompts/`), and the subagents' progress notes (`target/fleet/progress/`). `make fleet-resume MOD=<id>` (printed by `make fleet-start`) turns it into what to do next: reports to file, interrupted subagents to restart (a silent session counts as dead after 15 minutes), the last conductor's actions. A restarted subagent's `make fleet-task` shows it its predecessor's notes and files. Feature beads are created 200 per `bd` call, so `make fleet-features` takes seconds and a rerun adds only what is missing. Headless runs wait out a network outage instead of counting it as stalled.
- **Quotas.** A headless run (`make fleet-next`, `make fleet-dispatch`) uses the model the tool is on and stops on a quota, usage or rate limit (`RESOURCE_EXHAUSTED`, 429, "usage limit reached"…), saying when that model is back. With `FALLBACK=1` it uses the first model in `.agents/models.json` that is installed, offered by its CLI (agy: `agy models`) and not resting instead: a limit rests that model until the reset its error names (else 30 min, doubling, at most 6 h; a rejected model a day) and the next run continues on the next model, switching tool if needed (agy → claude → codex); only when every model rests does it wait. `make fleet-models` shows the chain and the rests; `make fleet-models ARGS=--clear` forgets them.
- Tools without hooks only have the instruction in `AGENTS.md`; run them headless to get the relaunching.

**Headless, any tool**
- `make fleet-next` (the mod in progress, else the next ready, on the model the tool is on), `make fleet-next AGENT=claude MOD=rekkit_core` (claude first, a specific mod), or `AGENT_CMD="gemini --approval-mode auto_edit -p" make fleet-next` for any CLI that takes a prompt as its last argument (check that tool's own flags for unattended runs).
- `/schedule` a nightly `make fleet-next AGENT=…`. It exits early if a run is already going or the mod in progress is waiting on you.

---

## 10. Human checkpoints and open decisions

| When | You do |
|---|---|
| After `rekkit_core` spec | Approve what goes into core |
| After each mod's release prep | Review diff, `git push`, `minisign -S`, publish the GitHub Release |
| New Beads bug labelled `horizon` (`bd list --label horizon`) | Upstream a Horizon PR, work around it, or defer |
| `DONE, CHECK OUT THE PROGRESS` | `make play MOD=<id>`, review (generated art first), push |
| A warning the harness should accept | Add it, with a reason, to `planning/harness-allowed-warnings.txt` |
| Wave 4 triage done | Approve which Agrarian Skies mods get ported |

Open decisions:
1. **How native hooks reach domain logic.** `example_mod/native/mixins/example_mod/hooks.rs` redeclares constants instead of calling the domain crate. Decide during `redpower_2` whether mixin bundles can depend on the crate (and on `rekkit_core`) or get it vendored in at pack time.
2. **Version era.** BuildCraft, Forestry and Thermal Expansion appear in two or three packs. Recommended: one crate per mod, the earliest pack is canonical, later behavior in a spec appendix behind a config flag.
3. **Legacy textures.** 1.2.5/1.4.7 mods ship sprite sheets. Asset-smith records slices in `mods/<id>/asset-map.json`; a shared slicing script under `scripts/` would make that reproducible.
4. **Client QoL mods.** Confirm NEI, Inventory Tweaks, minimaps and MAtmos stay out of scope.

Resolved: the shared crate exists and goes first, at `crates/rekkit_core`.

---

*agy specifics (agent frontmatter fields, tool names, `hooks.json` shape, `/boost`, model names) follow the Antigravity docs as of 31 Oct 2026 and have not been tested against a live agy install. Tool names in `tools:` must match exactly; an unknown name can hang a subagent.*

*Claude Code's copy (`.claude/`) is generated by `scripts/fleet/sync_agents.py`; its skills and hooks were checked in a live Claude Code session on 1 Nov 2026. Other tools get no generated copy: they read `AGENTS.md` and the files in `.agents/` directly.*
