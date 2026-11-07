# The conductor's steps

Read one with `make conductor-step STEP=<n>` when you start it, once. The rules are in agent.md.

## Step 0 Setup

Run `.agents/start.md` step 1 (`./setup.sh --check`, then `./setup.sh` if it says so). Relay every `!` line to the human word for word.

## Step 1 Spec

Take the mod: the epic in progress (`make fleet-status`), else the first ready epic (`bd ready --type epic`). `make fleet-context MOD=<id>` shows its state, spec, original archives and files on one page. Run `make fleet-start MOD=<id>`: it claims the mod (the stop hook now holds you until it is finished), creates its 11 stories, builds and runs Horizon alone at the pinned revision (`make horizon-status` shows the result), then prints where the last run stopped (`make fleet-resume MOD=<id>`). If a session before you died, ran out of credits or lost the network: `make fleet-inbox` for waiting reports, then restart each interrupted subagent on its bead with the same role and its saved prompt (target/fleet/prompts/<bead>.md); its `make fleet-task` shows it the earlier work. Never redo a closed bead. Create or switch to branch port/<id>. Resume at the first story that is not closed.

The spec, in two rounds, each Java method read once:
(a) One `spec-writer` in map mode: planning/mods/<id>.md's Overview, Systems list, registry, recipes, hazards and hooks, and <id>.features.json with every block, item and subsystem (no attributes yet), from the class outline and the registry classes only.
(b) `make fleet-parts MOD=<id>` splits the decompiled code into parts; start one `spec-writer` in part mode per part (each prompt: the mod, the part's list file, "part mode"), at most 2 at a time. Each reads its part once and writes its outline, its blocks' and items' attributes and its tickets.
(c) When every report is in: `make fleet-tickets MOD=<id> CHECK=1` (it merges the parts into the spec and outline first). Read its problems and the order yourself: a ticket that uses another's work must be after it, two tickets editing one file must be ordered, nothing may be uncovered, no ticket over 6 files or 8 reads. Send problems back to a spec-writer (part mode, the problem lines in its prompt) until it is clean, then `make fleet-features MOD=<id>` and `make fleet-tickets MOD=<id>`, which files them with their order.

## Step 2 Scaffold

`scaffolder` (Scaffold), and in the same round `asset-smith` for Step 5 (Extract assets: the original art and sound, tagged origin original).

## Step 3 Code

The ticket loop (agent.md, *The ticket loop*), also for the tickets of steps 5 and 6. Repeat until no ticket is open and the mod's rekkit_core beads are closed.

## Step 4 Check

`make agent-gate MOD=<id>` (rustfmt, clippy -D warnings, tests, check-mods, harness dry run) and `parity-critic` in Check mode (the Rust against the spec and the decompiled Java). Every finding is a bug under Check; send it to the role that owns the file. `make fleet-contract MOD=<id>` files every difference from the original (a vanilla or swapped texture, a full cube for a thin block, no facing, a stock GUI grid, a scenario that fails) as a bug under the step that fixes it and reopens what it touches; give each to its owner (the ticket loop works them). Repeat until the gate passes, the contract check is clean and the verdict is pass. A block or item is never done while it differs from the original: placing, opening some screen and dropping is not the original.

## Step 5 Extract assets

Started with step 2. `asset-smith` slices exactly the sprites the contracts name; done when `make contract-check MOD=<id> OWNER=asset-smith` is clean.

## Step 6 New assets

`make fleet-gaps MOD=<id>` makes a feature per missing asset (anything 26.3 needs that the original never had). `asset-smith` draws each in mods/<id>/art/ and runs `make art MOD=<id>`. Done when `make check-mods MOD=<id> COMPLETE=1` passes. No gaps: close the story.

## Step 7 Recheck

The whole gate again, `make check-mods MOD=<id> COMPLETE=1`, `make art MOD=<id> ARGS=--check`, and `parity-critic` in Recheck mode over the whole mod (code, content and asset-map.json). Findings go back to their owners; repeat until clean.

## Step 8 Harness

`compat-auditor` runs the live harness (isolated and all mods). Every block must be placed by /setblock and by a player, in every variant, with no WARN/ERROR line in the log; its outline is the original's shape, its GUI has the original's slots where the original has them, and every contract scenario passes. Then `make fleet-contract MOD=<id>` again: it files the live failures. The stop hook will not let you finish while the contract check or the last harness run finds a difference. If the harness exits 3 (only Horizon's own problems: see `make horizon-status`), close the story with a comment "Horizon <rev> broken: <what>", file a bug labelled horizon, and go on: a broken Horizon never holds up the mod.

## Step 9 Package

`release-clerk` checks the version (`make version-sync` bumps it as work lands), commits (with any rekkit_core change, and planning/fleet-board.md and .beads/issues.jsonl from `make board`) and packs. Merge port/<id> into main locally. Close the story.

## Step 10 Finish

Your last message starts with exactly `DONE, CHECK OUT THE PROGRESS: make play MOD=<id>   (all mods: make play)` then, briefly: the generated assets to review, whether the next release will be a pre-release (original assets) and anything flagged as Horizon's.
