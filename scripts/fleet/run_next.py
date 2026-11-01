#!/usr/bin/env python3
"""Headless: run the conductor on exactly ONE mod with any agent CLI until it is finished.

Takes the mod given, else the one in progress, else the next ready epic in Beads (`bd ready`). When
the agent exits before the mod is finished (step 9 Package closed, or the epic blocked), it is
started again to continue, so the run never ends early whatever the tool does at a turn end. It
ends when the mod is finished, after `make fleet-stop`, or when MAX_STALLED runs in a row change
nothing in Beads (stuck). A lock (target/fleet.lock) stops two runs overlapping; the output also
goes to target/fleet-<id>.log.

Agents (--agent, or AGENT):
  agy     agy -p PROMPT --effort high
  claude  claude -p PROMPT --permission-mode acceptEdits, with the build, test and git commands the
          fleet needs allowed (its hooks still block push, sign and release)
  codex   codex exec --full-auto PROMPT (workspace-write sandbox)
Any other CLI: --agent-cmd (or AGENT_CMD) "cli --flags"; the prompt is added as the last argument.

Usage:
  python3 scripts/fleet/run_next.py --agent claude
  python3 scripts/fleet/run_next.py --agent agy redpower_2
  AGENT_CMD="gemini --approval-mode auto_edit -p" python3 scripts/fleet/run_next.py
"""

from __future__ import annotations

import argparse
import os
import shlex
import subprocess
import sys
import time
from pathlib import Path

import beads
from guard import STOP_FILE, progress_mark

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / "target" / "fleet.lock"
MAX_STALLED = 3
CLAUDE_ALLOWED = [
    "Read", "Grep", "Glob", "Edit", "Write", "Agent", "Task", "Skill", "TodoWrite",
    *(f"Bash({c}:*)" for c in (
        "cargo", "make", "python3", "python", "./setup.sh", "scripts/fleet/python.sh", "unzip", "ls", "mkdir",
        "cp", "mv", "git status", "git diff", "git log", "git show", "git add", "git commit", "git checkout",
        "git switch", "git branch", "git merge", "git rev-parse", "git ls-files")),
]


def pick(wanted: str | None) -> str | None:
    active = beads.active_mod()
    if active and wanted and wanted != active:
        sys.exit(f"{active} is still in progress. Finish it (or block its epic) before starting {wanted}.")
    return wanted or active or beads.next_mod()


def agent_command(agent: str | None, agent_cmd: str | None, prompt: str) -> list[str]:
    if agent_cmd:
        return [*shlex.split(agent_cmd), prompt]
    if agent == "agy":
        return ["agy", "-p", prompt, "--effort", "high"]
    if agent == "claude":
        return ["claude", "-p", prompt, "--permission-mode", "acceptEdits", "--allowedTools", *CLAUDE_ALLOWED]
    if agent == "codex":
        return ["codex", "exec", "--full-auto", prompt]
    sys.exit("Say which agent runs it: --agent agy|claude|codex, or --agent-cmd \"<cli and flags>\".")


def run_agent(cmd: list[str], log) -> int:
    try:
        with subprocess.Popen(cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                              encoding="utf-8", errors="replace") as proc:
            for line in proc.stdout:
                sys.stdout.write(line)
                log.write(line)
                log.flush()
        return proc.returncode
    except FileNotFoundError:
        sys.exit(f"{cmd[0]} is not installed or not on PATH.")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("mod", nargs="?", help="Board row to run (default: active, else next queued)")
    parser.add_argument("--agent", default=os.environ.get("AGENT") or None, choices=["agy", "claude", "codex"])
    parser.add_argument("--agent-cmd", default=os.environ.get("AGENT_CMD") or None,
                        help="Any other agent CLI and its flags; the prompt is appended")
    parser.add_argument("--max-runs", type=int, default=30, help="Agent launches at most (default 30)")
    args = parser.parse_args()

    try:
        mod = pick(args.mod)
        if not mod:
            print("Nothing to run: no mod in progress and no ready epic in Beads.")
            return 0
        if beads.mod_status(mod)["finished"]:
            print(f"{mod} is finished; nothing for an agent to do (the human's review and push are next).")
            return 0
    except beads.BeadsError as exc:
        sys.exit(f"[beads] {exc}")
    agent_command(args.agent, args.agent_cmd, "")  # fail now on a missing --agent

    LOCK.parent.mkdir(parents=True, exist_ok=True)
    try:
        LOCK.mkdir()  # atomic, so it doubles as a lock
    except FileExistsError:
        sys.exit(f"Another fleet run holds {LOCK.relative_to(ROOT)}. Remove it if that run is gone.")
    STOP_FILE.unlink(missing_ok=True)
    stalled, code = 0, 0
    try:
        with (ROOT / "target" / f"fleet-{mod}.log").open("a", encoding="utf-8") as log:
            for run in range(1, args.max_runs + 1):
                # Follow the mod in progress: a rekkit_core bump the conductor started runs first.
                mod = beads.active_mod() or mod
                step = beads.mod_status(mod)["next"] if beads.epic(mod)["status"] != "open" else "step 1 Spec"
                prompt = (
                    f"Follow AGENTS.md. Act as the conductor (.agents/agents/conductor/agent.md) for the mod "
                    f"'{mod}' only. First run `make fleet-start MOD={mod}`. "
                    + ("" if run == 1 else f"An earlier run ended before the mod was finished; continue from {step}. ")
                    + "Do not stop until it is finished: step 9 Package closed in Beads, or its epic blocked with the "
                      "reason as a comment if only a person can unblock it. Never push.")
                before = progress_mark()
                print(f"\n=== Fleet run {run}: {mod}, {step} ({args.agent or args.agent_cmd.split()[0]})", flush=True)
                code = run_agent(agent_command(args.agent, args.agent_cmd, prompt), log)
                if STOP_FILE.is_file():
                    print("Stopped by `make fleet-stop`.")
                    return code
                st = beads.mod_status(mod)
                if st["finished"]:
                    print(f"{mod} is finished ({st['epic']['status']}; next: {st['next']}).")
                    return 0
                stalled = stalled + 1 if progress_mark() == before else 0
                if stalled >= MAX_STALLED:
                    print(f"{MAX_STALLED} runs in a row changed nothing in Beads; stopping. See target/fleet-{mod}.log.")
                    return 1
                if code:
                    time.sleep(min(60 * stalled, 300))  # an agent error (quota, network): back off
            print(f"Stopped after {args.max_runs} runs (--max-runs); {mod} is not finished.")
            return 1
    finally:
        LOCK.rmdir()


if __name__ == "__main__":
    raise SystemExit(main())
