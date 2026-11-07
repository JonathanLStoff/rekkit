# Token audit: predictions (written before any sample ran)

Method: file sizes from `wc -c` at about 4 bytes per token, plus the one measured fact so far: `agy -p "pong"` costs **12,152 input tokens** for a one-word answer (system prompt, tool schemas, project instructions). Everything below is a guess to be checked against `performance.md`.

**Unit**: "billed input" = the sum over every turn of that turn's full context (fresh + cache read + cache write), because each turn re-sends it. "Peak ctx" = the largest single context. Output is counted apart.

## Fixed overhead each agent pays before it does anything

| Piece | Bytes | About tokens | Paid by |
|---|---|---|---|
| agy base (system, tools, AGENTS.md) | measured | 12,152 | every agent, every turn |
| `.agents/start.md` | 6.5k | 1.6k | main session |
| `conductor/agent.md` | 14.0k | 3.5k | conductor |
| `mod-spec` skill | 12.3k | 3.1k | spec-writer, test-smith |
| `fleet-plan.md` (if read) | 22.8k | 5.7k | anyone who opens it |
| role file | 1.1k to 3.8k | 0.3k to 1k | each subagent |
| other skills (`port-mod`, `horizon-hooks`, `parity-checklist`, `legacy-asset-mapping`, `extract-jar-assets`) | 3.7k to 4.7k each | 1k to 1.2k each | per role |
| `buildcraft.md` spec | 11.6k | 2.9k | any role that reads it whole |

So a subagent starts at roughly **15k to 25k** tokens, and pays that on every one of its 30 to 80 turns: **0.5M to 2M billed input before any real work**. Hypothesis H1: this fixed floor times the number of subagents and turns is the biggest line.

## Per step

| Step | Who | Turns (guess) | Peak ctx | Billed input | Output | Why |
|---|---|---|---|---|---|---|
| S0 session start | main | 5 to 15 | 20k to 40k | 0.1M to 0.4M | 2k to 8k | `bd prime`, `make doctor` summary, start.md |
| C1 conductor boot | conductor | 5 to 15 | 30k to 60k | 0.2M to 0.7M | 3k to 10k | agent.md plus `fleet-start` output |
| S1a decompile + parts | conductor | 3 to 8 | 25k to 50k | 0.1M to 0.3M | 1k to 3k | shell output only |
| S1b outline, per part | spec-writer | 40 to 120 | 80k to 190k | 2M to 12M | 25k to 80k | Java excerpts accumulate; writes a large spec |
| S1c tickets, per part | spec-writer | 30 to 80 | 70k to 180k | 1M to 6M | 15k to 50k | re-reads spec and Java it already saw |
| S1d file tickets | conductor | 2 to 6 | 30k to 60k | 0.05M to 0.3M | 1k to 3k | validation output |
| S2 scaffold | scaffolder | 15 to 40 | 30k to 70k | 0.4M to 2M | 8k to 25k | copy and rename, build output |
| S3 ticket (each) | content-author, domain-builder, hook-engineer, test-smith | 30 to 100 | 60k to 190k | 1M to 8M | 10k to 60k | contract files, Java a method at a time, cargo output |
| C2 conductor loop | conductor | 100 to 400 | 100k to 200k+ | 10M to 60M | 30k to 120k | every report and board refresh stays in context; one turn per ticket event |
| S4 agent-gate | worker | 10 to 40 | 40k to 120k | 0.3M to 3M | 3k to 15k | clippy and test output |
| S4 parity-critic | parity-critic | 30 to 90 | 80k to 190k | 1.5M to 8M | 8k to 30k | reads spec and diff and Java |
| S5 extract assets | asset-smith | 20 to 60 | 40k to 100k | 0.6M to 3M | 5k to 20k | file listings, sprite maps |
| S6 new assets | asset-smith | 10 to 30 | 30k to 60k | 0.2M to 1M | 3k to 12k | `fleet-gaps`, art scripts |
| S7 recheck | parity-critic | 30 to 90 | 80k to 190k | 1.5M to 8M | 8k to 30k | second full pass |
| S8 harness | test-smith, compat-auditor | 20 to 80 | 50k to 160k | 0.6M to 6M | 5k to 25k | e2e logs are long |
| S9 package | release-clerk, scribe | 10 to 30 | 30k to 60k | 0.2M to 1.5M | 3k to 10k | version and pack output |
| X stop-gate and hooks | all | adds 5% to 20% | n/a | adds 5% to 20% | n/a | retries after a failed gate re-run clippy and tests |

## Hypotheses to test (what I expect is wasteful)

- **H1 fixed floor**: the 12k base plus role plus skills is paid on every turn of every agent. If it is 20k and a ticket has 60 turns, that is 1.2M before any work.
- **H2 turn count**: tickets are sized for "200k of context" but cost is turns times context, so a 200k ticket with 80 turns bills far more than 200k. The sizing rule may be measuring the wrong thing.
- **H3 conductor growth**: one long-lived conductor accumulates every report and board refresh and re-pays them every turn; it may be the largest single line over a mod.
- **H4 fan-out**: "many subagents at once" multiplies H1 and H2, which would explain 30% of the week in 20 minutes.
- **H5 command output**: `cargo`, `clippy`, `make agent-gate` and `e2e` output enters context unfiltered; one failed gate can add 20k to 50k per retry.
- **H6 re-reading**: the outline pass and the ticket pass both read the same Java; the second read may be avoidable.
- **H7 skills and docs**: `mod-spec` (12 KB) loaded in full by several roles is a large fixed cost for a section or two of use.

## What to read from the results

For each step: mean and range of billed input, peak context, turns, output, and the weighted cost (fresh input 1x, cache write 1.25x, cache read 0.1x, output 5x). Then the fan-out product: tickets in flight times turns times mean context.
