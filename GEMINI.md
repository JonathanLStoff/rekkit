# Rekkit for Gemini CLI

The rules every agent follows are in AGENTS.md (the main session's start-up steps: `.agents/start.md`):

@AGENTS.md

Gemini CLI has no Rekkit roles or hooks of its own: play each role from `.agents/agents/<role>/agent.md`, read the skills in `.agents/skills/` by path, and run `make agent-gate` before you stop.
