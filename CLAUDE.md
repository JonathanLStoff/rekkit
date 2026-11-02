# Rekkit for Claude Code

The rules every agent follows are in AGENTS.md (the main session's start-up steps: `.agents/start.md`):

@AGENTS.md

## Claude Code specifics

- `/conductor` conducts one mod; the roles are subagents in `.claude/agents/`, the skills in `.claude/skills/`. Both, and the hooks in `.claude/settings.json`, are generated from `.agents/` by `make agents`: edit `.agents/`, never the copies.
- Hooks: a short Beads note at session start; the guards (no push, sign, release, submodule, force or recursive delete; nothing under `horizon/`; no code under `mods/<id>/content/`; the allowed-warnings list is a person's; no whole decompiled classes; subagents never run `bd`; a subagent past 200k tokens of context only wraps up); and the stop gate (a subagent until its report is in `target/fleet/inbox/`, the conductor until its mod is finished).
