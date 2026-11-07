# Token audit: fix plan

Goal: spend fewer tokens per ported mod. When saving tokens and speed pull in opposite directions, this plan picks tokens: fewer agents at once, more scripts, more work done before an agent starts.

Based on [`performance.md`](performance.md) (one sample per step, `gemini-3.8-flash-medium`, S1c made worse by the audit's worktree setup). Every "expected" figure below is an estimate from that data, not a measurement. Phase E re-measures each one.

## The cost model the numbers fit

A run with `T` turns, a fixed floor `F` per turn and context growing about `g` tokens per turn bills about

```
billed ≈ T·F + g·T²/2
```

Checked against the audit:

| Run | T | F (floor + role) | g (peak−floor)/T | Predicted | Measured |
|---|---|---|---|---|---|
| S3 domain-builder, 1.5M cap | 46 | ~12k | ~0.9k | 0.55M + 0.95M = 1.50M | 1.50M |
| S1c tickets | 50 | ~12k | ~1.25k | 0.60M + 1.56M = 2.16M | 2.16M |

What follows from it:

- **Cost grows with the square of the turn count.** Halving `T` cuts a code ticket from about 1.5M to about 0.52M, a 65% saving. Halving `g` (less output per tool call) saves about 0.48M. Doing both gives about 0.40M.
- The floor `F` costs `T·F`, so trimming the floor helps in proportion to the turn count, and does little until the turns are down.
- So the order is: **(1) fewer turns, (2) less context added per turn, (3) a smaller floor**, and fewer agents running at once so a mistake isn't repeated N times.

Targets for a code ticket, which is about 1.5M billed and often unfinished today:

| Metric | Today | Target |
|---|---|---|
| Turns | 42–50 | ≤ 20 |
| Billed input | 1.4–1.5M | ≤ 0.45M |
| Discovery commands (listings, `git status`, recursive search) | ~40% of commands | 0 |
| Reads of `scripts/`, `Makefile` by role agents | several per run | 0 |
| Finished within budget | 1–2 of 4 | 4 of 4 |

---

## Phase A: stop the waste (small changes, done first)

### A1. Block discovery sweeps for role subagents
**Why:** 110 listing sweeps put 81k tokens (21% of everything tools added) into context, and text searches another 54k (14%). Each token added is paid about 26 more times. `AGENTS.md` already forbids sweeps, and flash-medium ignored it, so the rule has to be enforced, not just written.

**What:** in `scripts/fleet/guard.py` `check_shell` (and the agy hook in `.agents/hooks*.json`), when `session.tasks` is set (a role subagent), refuse:
- recursive listings: `Get-ChildItem -Recurse`, `ls -R`, `dir /s`, `find <dir>` with no `-maxdepth 1`, `tree`;
- `git status`, `git log -p`, `git diff` with no path;
- `rg`/`grep`/`Select-String` over `assets/decompiled/` (the message points to `make java-find`), and over `planning/mods/<id>.md` (points to `make spec-toc` / `spec-show`);
- any search with `-C` greater than 2.

Each refusal names the command to use instead, so the agent loses one short turn and doesn't go on exploring.

**Expected:** most of the 21% + 14% goes away, so `g` falls by about a quarter to a third.

### A2. Block role agents from reading the toolchain
**Why:** S1c read `scripts/fleet/beads.py` 6 times and ran `git log -p` on it. S7 read `scripts/contract_check.py` 4 times. The S3 reruns opened `guard.py` and `e2e_harness.py`. The rules forbid all of this.

**What:** `check_read` refuses `scripts/**`, `Makefile`, `.agents/fleet-plan.md`, `.agents/reference.md` and `horizon/**` source for role subagents, with the message "run `<cmd> --help` or `make help`". Where an agent read a script to learn a format, add the format to `--help`: the ticket JSON schema in `make fleet-tickets SCHEMA=1`, the contract schema in `make contract-check SCHEMA=1`.

### A3. Refuse repeated reads
**Why:** the parity-critic ran the same search 2–3 times, S1c read one file 6 times, S7 read one file 4 times. A second read of an unchanged file adds nothing but cost.

**What:** the guard already keeps a per-session journal (`journal()`). Record `(path, range, mtime)` for every read and `command` for every search, and refuse an exact repeat while the file is unchanged ("you read this at turn N; it has not changed").

### A4. Short output from every make target an agent runs
**Why:** S8c read a 10k-token `target/e2e-report/summary` and a 6.5k-token report through `read_file`. The 2.5k-token output cap fills up on listings. Build output is small today (3%), but one failed gate can add 20–50k per retry (H5, not measured).

**What:** a summary mode, on by default:
- `make e2e-mod` / `e2e-all-mods`: at most 30 lines: pass/fail per stage, then for each failure the action, the expected and actual result, and the first log line. Print the path of the full report, and don't tell agents to read it.
- `make agent-gate`, `check-mods`, `contract-check`: only failures, grouped, at most 40 lines, plus "N more, `VERBOSE=1`".
- cargo: run with `--message-format short` and `-q` inside the gate.

### A5. Fix S1c's environment problem before re-measuring
`make fleet-tickets` failed in a worktree on a git-common-dir error, and the agent spent turns on it. Either make `fleet-tickets` worktree-safe or make the audit harness run in a full clone, so the next S1c sample measures the step, not the setup.

---

## Phase B: fewer turns (the biggest saving)

### B1. `make fleet-task` gives the agent its whole brief in one turn
**Why:** a ticket already lists its spec sections, Java methods and files (`scripts/fleet/beads.py` around line 487), but only by name. The worker then spends a turn on each one (`spec-show`, `java-show`, reading the template, listing the contract dir). Those are the 40+ turns. The content-author wrote nothing in 42 turns because they all went on fetching.

**What:** `make fleet-task BEAD=<id> ROLE=<role>` prints, in one bounded block of at most about 12k tokens:
1. the ticket body and what done means;
2. the text of each `spec` section (as `spec-show` prints it);
3. the source of each `java` member (as `java-show` prints it, trimmed to the method);
4. for each file in `files`: whether it exists, and if not, the matching `mods/example_mod` template (one file per kind: a block JSON, a blockstate, a model, a loot table, a lang entry, a creative tab), so nobody lists `example_mod`;
5. the contract schema and the section of `mod-spec` the role needs (see C3), not the whole skill;
6. the exact check commands to run before reporting;
7. an interrupted predecessor's progress notes (already shown today).

The role files then say: "everything you need is in `fleet-task`'s output. Read nothing else unless a check fails and names the file."

**Expected:** turns per ticket drop from 42–50 to 15–20. By the cost model that is 1.5M → about 0.4–0.5M, even though the floor is bigger (+12k loaded once and cached).

**Trade-off:** the brief is paid on every turn. Keep it to what the ticket names, and refuse to inline more than about 12k (`fleet-tickets CHECK=1` rejects a ticket whose brief would be bigger: split it).

### B2. Size tickets in turns and billed input, not peak context
**Why:** tickets are sized at "≤ 200k tokens of context", but peak context never passed 75k. What costs money is turns, and the 200k guard (`TASK_TOKENS` in `guard.py`) never fires. A "200k ticket" billed 1.5M.

**What:**
- `guard.py`: alongside peak context, track **billed input so far** (the sum of the transcript's per-turn usage) and **turns**. Wrap up at 500k billed or 25 turns, whichever comes first, instead of 200k peak context.
- spec-writer tickets: replace `effort_k` with `writes` (the number of files the ticket creates or edits) and `reads` (spec sections + Java members). `fleet-tickets CHECK=1` caps a ticket at about 6 files written and about 8 reads, and rejects bigger ones so they get split. Change the spec-writer role file and the `mod-spec` skill to say so.

### B3. Don't blindly restart an unfinished ticket
**Why:** a restart pays the floor and the brief again, and if the ticket was too big, the second agent stops at the same place.

**What:** when a report comes back `partial`, the conductor (or the script in D1) gives the rest to a fresh agent only if the progress notes show at least one file finished. Otherwise it **splits** the ticket. A ticket that is partial twice is split, never started a third time.

### B4. One spec pass per part, not two
**Why:** S1b (outline, 0.82M for one block) and S1c (tickets, 2.16M for one ticket) both read the same Java (H6, not measured, but the role file sends both modes to `java-outline` + `java-show` on the same part).

**What:** one spec-writer per part writes the outline sections, `features.json` and the tickets in a single pass, reading each method once. If that pass is too long, split by part size (fewer classes per part in `make fleet-parts`), not by mode.

---

## Phase C: less context per turn, and a smaller floor

### C1. Generate the mechanical content with a script, not an agent
**Why:** content-author is the most wasteful role (1.5M, unfinished, no file written). Most of its output (blockstates, block and item models, loot tables, lang keys, creative-tab lists, item JSON with stack size and durability) follows directly from the contract file and `example_mod`'s shapes.

**What:** `make content-gen MOD=<id> [NAME=<block>]` writes those files from `planning/mods/<id>.contract/*.json` and `features.json`, deterministically, in `example_mod`'s format. The content-author then writes only the **contract** (from the Java) and the files no template covers (custom recipe types, unusual models). In the same way, `make e2e-gen` turns the contracts' scenarios into a draft `tests/e2e_actions.json` for test-smith.

**Expected:** content-author tickets become contract tickets, about 10–15 turns. Test-smith tickets get shorter too. This costs developer time once and saves tokens on every mod.

### C2. Lower the thinking level for mechanical roles
**Why:** thinking is the largest weighted line in the long runs (26k of 29.5k output tokens in S1c), and output is weighted ×5.

**What:** add an `effort:` field to each role file in `.agents/agents/*/agent.md` and have `make agents` carry it into each tool's copy (Claude Code agent frontmatter, agy and opencode model settings in `.agents/models.json`):
- `low`: scaffolder, content-author (after C1), asset-smith, release-clerk, scribe, compat-auditor;
- `medium`: test-smith, hook-engineer, domain-builder, spec-writer;
- `high`: domain-builder-pro, parity-critic.

Check with E1 that `low` doesn't add turns or failed gates. If it does, move that role back up.

### C3. Load skills and the conductor's file by section
**Why:** `mod-spec` (12 KB, about 3k tokens) was read in full by content-author. `conductor/agent.md` (14 KB, about 3.5k tokens) stays in the conductor's context on every turn.

**What:**
- Split `mod-spec` into a short core plus sections (`outline`, `contracts`, `test levels`, `tickets`), and add `make skill-show SKILL=<name> SECTION=<heading>`, the same tool as `spec-show`. `fleet-task` (B1) inlines only the section the role needs.
- Split `conductor/agent.md` into the loop (about 4 KB) and one appendix per step, read when that step starts.

### C4. Shrink what every agent loads
**Why:** the floor is about 11.1k on every turn of every agent. Most of it is the tool's own system prompt, but `AGENTS.md` (8.4 KB, about 2k tokens) is loaded into every role subagent even though it tells them they need only their prompt and role file.

**What:** move everything that applies only to the main session (opencode and local models, releases, versions, the full completion definition) out of `AGENTS.md` into `.agents/start.md`. Keep `AGENTS.md` at the rules a role subagent needs (about 2 KB). Give each role only the tools it uses: the frontmatter lists them already, so check that each tool honours the list.

**Expected:** about 1.5k × turns × agents. At 20 turns and 10 tickets that is about 0.3M per mod. Small, but it applies everywhere.

---

## Phase D: the conductor and fan-out

### D1. Run the routine dispatch loop as a script, not an agent
**Why:** the conductor proxy cost about 100k billed per report event (C2), and its context grows with every event, so cost goes up with the square of the event count. A mod has dozens to hundreds of tickets.

**What:** `make fleet-dispatch MOD=<id> [MAX=<n>]`, a Python loop: `fleet-ready` → start one headless worker per ready ticket (`agy -p --output-format json`, as `harness/run_sample.py` already does; `claude -p` for Claude Code) → wait → `fleet-inbox` → repeat. It records each worker's `usage` next to its report. It calls the LLM conductor only when a report is `blocked` or `failed`, or partial twice (B3), or has follow-ups that need a judgement, and then in a **fresh** session given just that report and `make fleet-resume`. The conductor role keeps steps 1, 4–9 and those exceptions.

**Expected:** routine events cost about zero LLM tokens instead of about 100k each. The conductor's sessions stay short.

### D2. Fewer agents at once
**Why:** fan-out doesn't change what one ticket costs, but it multiplies every mistake. Ten agents that each hit the same environment problem (like S1c's) or the same missing rule pay for it ten times before anyone sees it. It is also how 30% of the week went in 20 minutes.

**What:** `fleet-dispatch` defaults to `MAX=2`. Start the **first ticket of each role alone**, check its report and usage (as in E2), and only then run the rest in parallel. This is slower, and the slowdown is the price of the token saving.

### D3. A spending stop
**What:** `fleet-dispatch` adds up the billed input of its workers and stops starting new ones past a per-mod limit (`BUDGET=<tokens>`, default set from E1's numbers), then asks the person. `make fleet-stop` already exists for the manual case.

---

## Phase E: measure each change

### E1. Turn the audit harness into a regression check
`planning/token_audit/harness/` already runs one sample per step and builds `performance.md`. Add:
- `make token-audit [SAMPLES=s0,c1,s1b,s3a-full,...] [N=2]`: runs the chosen samples **N ≥ 2 times** (the audit had one) and writes a table that compares against the last saved baseline;
- per run: turns, billed input, weighted cost, discovery commands, refused commands (from A1–A3), files written, finished yes/no.

### E2. Order and acceptance
Re-measure after each phase, on `buildcraft` (and `redpower_2` for S4–S9, as in the audit):

| After | Re-run | Accept when |
|---|---|---|
| A1–A5 | S1c, S3a–d (1.5M cap), S4b, S7 | discovery commands = 0, script reads = 0, billed down ≥ 25% |
| B1–B2 | S3a–d | ≤ 20 turns, ≤ 0.5M billed, 4 of 4 finished |
| B4 | S1b + S1c merged | one pass ≤ the old S1b alone |
| C1 | content-author ticket | contract-only ticket ≤ 15 turns, `check-mods` clean |
| C2 | every role at its new effort | no more turns or failed gates than before |
| C3–C4 | S0, C1, S3a | floor per turn down by about 1.5k or more |
| D1–D2 | a real 10-ticket slice of a mod | conductor tokens per routine event ≈ 0 |

Also run each sample once on the fleet's usual first model (not only flash-medium), because a model that follows `AGENTS.md` may need fewer turns. That tells us whether a stronger model is cheaper per ticket in quota, even if it costs more per token.

### E3. The `bd setup` follow-up
Try `bd setup agy`, `bd setup claude` and `bd setup opencode` on a branch and re-run S0 and C1. Keep it only if the first-turn floor and boot cost don't go up. It may put Beads text into every role subagent, which A1–C4 are trying to avoid. Role subagents never use `bd`, so it should apply only to the main session if it is used at all.

---

## Order of work

1. **A1–A5**: guards and short output. Hours of work, needed before any measurement means anything.
2. **B1, B2**: the ticket brief and budgeting by turns. The biggest single saving.
3. **E1**, then re-measure.
4. **C1**: content and e2e generators.
5. **D1–D3**: scripted dispatch, low concurrency, spending stop.
6. **B3, B4, C2–C4**: retry rule, single spec pass, thinking levels, smaller floor.
7. **E2/E3** after each step. Drop any change that doesn't pay for itself.

## Where the changes go

- Guards and budget: `scripts/fleet/guard.py` (`check_shell`, `check_read`, `over_budget`, `TASK_TOKENS`) and the hook config in `.agents/hooks.json` / `.agents/hooks.windows.json`.
- Brief and ticket checks: `scripts/fleet/beads.py` (`fleet-task`, `fleet-tickets CHECK=1`).
- Role files, skills, conductor: `.agents/agents/*/agent.md`, `.agents/skills/*/SKILL.md`, then `make agents` (never edit the copies in `.claude/`).
- Generators and dispatch: new scripts under `scripts/` with `make` targets and `--help`.
- `AGENTS.md` / `.agents/start.md` for C4.

---

## Implementation status (2026-11-06)

Everything is built and uncommitted, and nothing has been measured yet. Run `make token-audit` (it uses agy quota, so a person runs it) to get `after.md`.

Decisions made in review: no `bd setup` anywhere. Our own `prime` hook is the single source of Beads context. Every role runs on the session's model (`model: inherit`) and only its thinking level differs. A quota stops a run, and `FALLBACK=1` is a person's choice. Tickets stay aimed at 200k, with the stop at 25 turns or 500k billed. 2 agents at a time.

| Item | Done | Where |
|---|---|---|
| Beads context with no commands (E3, decided) | Claude Code: SessionStart (also after compaction). agy: SessionStart. opencode: the plugin puts it in the main session's system prompt (once per session, so it stays cached) and on compaction. Role subagents get none of it. `bd setup` would have doubled the Claude hook, has no agy recipe, and would have put bd's generic text ("track ALL work in bd") into `AGENTS.md` for every agent | `scripts/fleet/sync_agents.py` (plugin), `guard.py prime` |
| A1 discovery guards | recursive listings, `find`, `tree`, `git status`, `git log -p`, bare `git diff`, grep over the Java or the spec, search context over 2 lines: refused for role subagents, a warning for the conductor (Claude: `additionalContext`) | `guard.py` `waste()` |
| A2 toolchain reads | `scripts/`, `Makefile`, `fleet-plan.md`, `reference.md` refused; formats moved to `make fleet-tickets SCHEMA=1` and `make contract-check SCHEMA=1` | `guard.py` `read_waste()` |
| A3 repeats | the same lines of an unchanged file, and the same `make java-*`/`spec-*`/`skill-*`/`fleet-context`, refused; a whole spec or skill over 60 lines refused | `guard.py` |
| A4 short output | the harness prints its first 25 failures itself; `contract-check` prints 40 lines (`OWNER=`, `VERBOSE=1`). `check-mods` was already one line | `e2e_harness.py`, `contract_check.py` |
| A5 worktrees | tickets and the outline are read from the checkout that wrote them | `beads.py` `WORK` |
| B1 brief | `fleet-task` prints the ticket, the outline and contract entries it covers, its files (a new file with its `example_mod` template), the contract format, its spec sections (a long one: only the subsections about what it covers), its Java members from every decompiled version, and its checks, in at most about 12k tokens. A redpower ticket came to about 3.9k | `scripts/fleet/brief.py` |
| B2 budget | wrap up at 25 turns, 500k billed or 200k of context, summed from the transcript as it grows (`REKKIT_TASK_TURNS`/`REKKIT_TASK_BILLED` override). Tickets: at most 6 files and 8 reads | `guard.py` `billed_so_far()`, `beads.py` |
| B3 no blind restarts | a second partial, or a partial with no progress notes: labelled `needs-split`, not ready, sent to a spec-writer in split mode | `beads.py` `inbox()`/`ready()` |
| B4 one spec pass | map mode (class outline and registry only), then part mode writes `<id>.parts/<part>.md`, `<part>.features.json` and tickets; `fleet-tickets` merges them | `spec-writer` role, `scripts/fleet/spec_merge.py` |
| C1 generators | `make content-gen` (blockstates, block and item models, loot tables, lang, creative tab, item numbers, thin-block shapes) and `make e2e-gen` | `scripts/content_gen.py`, `scripts/e2e_gen.py` |
| C2 thinking levels | `effort:` per role (low/medium/high, as in Q10). Claude Code: in the subagent's frontmatter. Headless agy and claude: `--effort`. opencode has no effort setting, so its roles keep the model's default | role files, `sync_agents.py`, `dispatch.py` |
| C3 by section | `make skill-toc`/`skill-show`. The conductor's file went from 14 KB to 7 KB, and each step's detail is in `steps.md` (`make conductor-step STEP=<n>`) | `scripts/fleet/spec.py` |
| C4 smaller floor | `AGENTS.md` went from 8.4 KB to 2.8 KB; the main session's rules moved to `.agents/start.md` | |
| D1 dispatcher | `make fleet-dispatch`: headless workers, reports filed, tokens in `target/fleet/usage.jsonl`; exit 2 for the conductor, 3 for a person; `CONDUCTOR=1` runs a fresh conductor for exceptions | `scripts/fleet/dispatch.py` |
| D2 fewer at once | 2 at a time (dispatcher and the spawn hook), and the first ticket of each role alone | |
| D3 spending stop | `BUDGET=` (default 20M billed per dispatch run) | |
| Models | no role names a model; `models.json` chains are only for `FALLBACK=1`; `fleet-next` and `fleet-dispatch` stop on a quota | `run_next.py`, `.agents/models.json` |
| E1 | `make token-audit [SAMPLES=] [N=2] [MODEL=]` → `runs/after/`, `after.md`. The worktree now also gets untracked files | `harness/audit.py`, `run_sample.py` |

Found along the way:
- The old opencode plugin called `ctx.tool.hook(...)` from a `setup` export. The installed `@opencode-ai/plugin` 1.19.5 has no such API (its module form is `{ id, server }` returning hooks such as `tool.execute.before`), so the opencode guards most likely never ran. It is rewritten to that form; a test load primed 605 characters and refused a write under `horizon/`.
- The harness never reads `tests/e2e_actions.json`, so `e2e-gen` saves a test-smith turns but does not change what is tested.
- The repository root holds about 40 one-off scripts (`fix_*.py`, `regenerate_models_v1` to `v7.py`): earlier agents writing throwaway tools turn after turn. `content-gen` is meant to replace that pattern.

Not tested live (no tickets in Beads on this machine, and agy quota is the person's):
- Headless workers through the dispatcher (`agy -p --agent <role>`, `claude -p --agent <role> --effort`).
- Whether agy reads `effort:` from a role file for its in-session subagents.
- The billed-turn count against a real agy transcript (the guard assumes one row with `input_tokens` per turn, as before).
- The brief through a real bead (tested by ticket key: `python scripts/fleet/brief.py --mod redpower_2 --key part01-ribbon-cable`).
- The guards were tested with 33 hook payloads for a subagent and a conductor, and the budget with fake transcripts.
- The existing redpower_2 tickets fail the new caps (for example 25 files in one ticket). They are already filed, so this matters only if they are re-filed.
