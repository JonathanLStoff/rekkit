# Rekkit for Claude Code

The rules, the start-of-session steps and the project reference are in AGENTS.md, shared by every agent tool:

@AGENTS.md

## Claude Code specifics

- `/conductor` runs one board row as the conductor; the roles are subagents in `.claude/agents/`, the skills are in `.claude/skills/`.
- `.claude/agents/`, `.claude/skills/` and the `hooks` in `.claude/settings.json` are generated from `.agents/` by `make agents`. Edit `.agents/`, never the copies.
- At session start a hook runs `bd prime`: Beads' state of the port. As the conductor you are the only writer to Beads; a subagent you start runs `make fleet-task BEAD=<id> ROLE=<role>` first and ends with its report in `target/fleet/inbox/` (SubagentStop holds it until it does); you file reports with `make fleet-inbox`.
- The hooks block pushing, releasing, signing, submodule changes, force flags and recursive deletes, writes under `horizon/`, code under `mods/<id>/content/` and edits to `planning/harness-allowed-warnings.txt`, and a builder subagent reading `assets/decompiled/` (clean-room). After `make fleet-start MOD=<id>` the Stop hook keeps you working until steps 1–9 of that mod are closed in Beads (or its epic is blocked with a reason, or the user runs `make fleet-stop`); outside a fleet run, while `make agent-gate` fails for what you wrote. Ten turn ends in a row with no change in Beads and it lets you stop.
