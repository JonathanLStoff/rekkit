#!/usr/bin/env python3
"""Headless: run the conductor on exactly ONE board row with any agent CLI until it is finished.

Takes the row given, else the active row, else the first queued row whose deps are all done
(planning/fleet-board.md). When the agent exits before the row is finished (release prep done and
waiting on the human, `done` or `blocked`), it is started again to continue, so the run never ends
early whatever the tool does at a turn end. It ends when the row is finished, after
`make fleet-stop`, or when MAX_STALLED runs in a row leave the board unchanged (stuck). A lock
(target/fleet.lock) stops two runs overlapping; the output also goes to target/fleet-<id>.log.

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

from guard import BOARD, STOP_FILE, board_digest, board_rows, next_step, row_finished

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


def pick(rows: dict[str, list[str]], wanted: str | None) -> str | None:
    active = next((m for m, c in rows.items() if c[7] == "active"), None)
    if active and wanted and wanted != active:
        sys.exit(f"{active} is still active. Finish it (or mark it blocked) before starting {wanted}.")
    if wanted or active:
        return wanted or active
    for mod, cells in rows.items():
        deps = [d for d in cells[6].replace(" ", "").split(",") if d and d != "—"]
        if cells[7] == "queued" and all(rows.get(d, [""] * 8)[7] == "done" for d in deps):
            return mod
    return None


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

    rows = board_rows()
    mod = pick(rows, args.mod)
    if not mod:
        print("Nothing to run: no active row and no queued row with finished deps.")
        return 0
    if mod not in rows:
        sys.exit(f"No board row {mod!r} in {BOARD.relative_to(ROOT)}.")
    if row_finished(rows[mod]):
        print(f"{mod} is {rows[mod][7]}; nothing for an agent to do (step 9 waits on your push).")
        return 0
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
                rows = board_rows()
                # Follow the active row: a rekkit_core bump the conductor inserted runs first.
                mod = next((m for m, c in rows.items() if c[7] == "active"), mod)
                step = next_step(rows[mod])
                prompt = (
                    f"Follow AGENTS.md. Act as the conductor (.agents/agents/conductor/agent.md) for board row "
                    f"'{mod}' only. First run `make fleet-start MOD={mod}`. "
                    + ("" if run == 1 else f"An earlier run ended before the row was finished; continue from {step}. ")
                    + "Do not stop until the row is finished: release prep done (steps 9-10 ▶), or `blocked` with "
                      "the reason in Notes if only a person can unblock it. Never push.")
                before = board_digest()
                print(f"\n=== Fleet run {run}: {mod}, {step} ({args.agent or args.agent_cmd.split()[0]})", flush=True)
                code = run_agent(agent_command(args.agent, args.agent_cmd, prompt), log)
                if STOP_FILE.is_file():
                    print("Stopped by `make fleet-stop`.")
                    return code
                cells = board_rows().get(mod)
                if cells and row_finished(cells):
                    print(f"{mod} is finished ({cells[7]}; next: {next_step(cells)}).")
                    return 0
                stalled = stalled + 1 if board_digest() == before else 0
                if stalled >= MAX_STALLED:
                    print(f"{MAX_STALLED} runs in a row left the board unchanged; stopping. See target/fleet-{mod}.log.")
                    return 1
                if code:
                    time.sleep(min(60 * stalled, 300))  # an agent error (quota, network): back off
            print(f"Stopped after {args.max_runs} runs (--max-runs); {mod} is not finished.")
            return 1
    finally:
        LOCK.rmdir()


if __name__ == "__main__":
    raise SystemExit(main())
