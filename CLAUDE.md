# Rekkit for Claude Code

The rules every agent follows are in AGENTS.md (the main session's start-up steps: `.agents/start.md`):

@AGENTS.md

## Claude Code specifics

- `/conductor` conducts one mod; the roles are subagents in `.claude/agents/`, the skills in `.claude/skills/`. Both, and the hooks in `.claude/settings.json`, are generated from `.agents/` by `make agents`: edit `.agents/`, never the copies.
- Hooks: the Beads context (`bd prime`, and the resume page while a mod is in progress) at session start and after compaction, so no one runs `bd prime`; the guards (no push, sign, release, submodule, force or recursive delete; nothing under `horizon/`; no code under `mods/<id>/content/`; the allowed-warnings list is a person's; no whole decompiled classes; subagents never run `bd`; a subagent sweeping folders, reading the toolchain or a whole spec, or reading a file twice is refused, and the conductor is warned; a subagent past 25 turns, 500k tokens billed or 200k of context only wraps up); and the stop gate (a subagent until its report is in `target/fleet/inbox/`, the conductor until its mod is finished).
- Subagents run on the session's model (`model: inherit`) with their role's `effort`.
