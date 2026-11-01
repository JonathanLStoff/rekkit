# Rekkit for Gemini CLI

The rules, the start-of-session steps and the project reference are in AGENTS.md, shared by every agent tool:

@AGENTS.md

Gemini CLI has no Rekkit roles or hooks of its own: play each role from `.agents/agents/<role>/agent.md`, read the skills in `.agents/skills/` by path, and run `make agent-gate` before you stop.
