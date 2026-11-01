# Rekkit for Claude Code

The rules, the start-of-session steps and the project reference are in AGENTS.md, shared by every agent tool:

@AGENTS.md

## Claude Code specifics

- `/conductor` runs one board row as the conductor; the roles are subagents in `.claude/agents/`, the skills are in `.claude/skills/`.
- `.claude/agents/`, `.claude/skills/` and the `hooks` in `.claude/settings.json` are generated from `.agents/` by `make agents`. Edit `.agents/`, never the copies.
- The hooks block pushing, releasing, signing, submodule changes, force flags and recursive deletes, writes under `horizon/` and code under `mods/<id>/content/`. After `make fleet-start MOD=<id>` the Stop hook keeps you working until that board row is finished (or `blocked` with a reason, or the user runs `make fleet-stop`), and in any session while `make agent-gate` fails for the mods and crates you wrote to. Ten turn ends in a row with no board change and it lets you stop.
