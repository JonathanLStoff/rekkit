#!/usr/bin/env python3
"""Headless: run the conductor on exactly ONE mod with any agent CLI until it is finished.

Takes the mod given, else the one in progress, else the next ready epic in Beads (`bd ready`). When
the agent exits before the mod is finished (step 9 Package closed, or the epic blocked), it is
started again to continue, so the run never ends early whatever the tool does at a turn end. It
ends when the mod is finished, after `make fleet-stop`, or when MAX_STALLED runs in a row change
nothing in Beads (stuck). A lock (target/fleet.lock) stops two runs overlapping; the output also
goes to target/fleet-<id>.log.

Checkpoints: everything a relaunched conductor needs survives the crash (Beads, the hooks' journal,
saved subagent prompts, progress notes): `make fleet-start` prints it (scripts/fleet/resume.py). When
the network drops, the run waits for it to return instead of counting stalled runs.

Models: each run uses the first model of .agents/models.json that is installed, offered and not
resting (scripts/fleet/models.py). A run that ends on a quota, usage or rate limit rests that model
until its reset and the next run continues the mod on the next model, in any tool; when every model
is resting, it waits for the first reset. Runs cut short by a limit do not count as stalled.

Agents (--agent, or AGENT): the tool to try first (default: the order in .agents/models.json)
  agy     agy -p PROMPT --model M, its terminal sandbox off (scripts/fleet/agy.py)
  claude  claude -p PROMPT --model M --fallback-model <the rest of its chain> --permission-mode
          acceptEdits, with the build, test and git commands the fleet needs allowed (its hooks
          still block push, sign and release)
  codex   codex exec --full-auto [-m M] PROMPT (workspace-write sandbox)
  opencode  opencode run [-m M] PROMPT, on opencode's selected model when the chain says null (local
          models: the conductor sizes its subagents with `make fleet-local`)
Any other CLI: --agent-cmd (or AGENT_CMD) "cli --flags"; the prompt is added as the last argument.
It has no model chain: on a limit the run waits for the reset.

Usage:
  python3 scripts/fleet/run_next.py
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

import agy
import beads
import models
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


def agent_command(c: models.Candidate, chain: list[models.Candidate], prompt: str, tool_log: Path) -> list[str]:
    model = ["--model", c.model] if c.model else []
    if c.tool == "agy":
        return [*agy.command(["-p", prompt]), *model, *([] if c.model else ["--effort", "high"]),
                "--log-file", str(tool_log)]
    if c.tool == "claude":
        # Claude Code's own fallback covers an overloaded model within the run; a quota ends the run.
        rest = [o.model for o in chain[chain.index(c) + 1:] if o.tool == "claude" and o.model]
        return ["claude", "-p", prompt, *model, *(["--fallback-model", ",".join(rest)] if rest else []),
                "--permission-mode", "acceptEdits", "--allowedTools", *CLAUDE_ALLOWED]
    if c.tool == "codex":
        return ["codex", "exec", "--full-auto", *(["-m", c.model] if c.model else []), prompt]
    if c.tool == "opencode":
        return ["opencode", "run", *(["-m", c.model] if c.model else []), prompt]
    raise ValueError(c.tool)


def run_agent(cmd: list[str], log) -> tuple[int, str]:
    """Exit code and the output (also shown and logged)."""
    seen: list[str] = []
    try:
        with subprocess.Popen(cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                              encoding="utf-8", errors="replace") as proc:
            for line in proc.stdout:
                sys.stdout.write(line)
                log.write(line)
                log.flush()
                seen.append(line)
                del seen[:-models.TAIL]
        return proc.returncode, "".join(seen)
    except FileNotFoundError:
        sys.exit(f"{cmd[0]} is not installed or not on PATH.")


def wait_for(until: float, what: str) -> bool:
    """Sleep until `until`; False if `make fleet-stop` asked to stop meanwhile."""
    print(f"{what}; waiting until {models.when(until)} (make fleet-stop ends the wait).", flush=True)
    while time.time() < until:
        if STOP_FILE.is_file():
            return False
        time.sleep(min(60, max(until - time.time(), 1)))
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("mod", nargs="?", help="Board row to run (default: active, else next queued)")
    parser.add_argument("--agent", default=os.environ.get("AGENT") or None, choices=["agy", "claude", "codex", "opencode"],
                        help="Tool to try first (default: the order in .agents/models.json)")
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
    custom = models.Candidate(args.agent_cmd, None) if args.agent_cmd else None
    chain = [custom] if custom else models.candidates(args.agent)
    if not chain:
        sys.exit("No agent CLI from .agents/models.json is installed (agy, claude, codex, opencode); "
                 "install one or give --agent-cmd \"<cli and flags>\".")

    LOCK.parent.mkdir(parents=True, exist_ok=True)
    try:
        LOCK.mkdir()  # atomic, so it doubles as a lock
    except FileExistsError:
        sys.exit(f"Another fleet run holds {LOCK.relative_to(ROOT)}. Remove it if that run is gone.")
    STOP_FILE.unlink(missing_ok=True)
    stalled, code, run, limited, launches = 0, 0, 0, 0, 0
    tool_log = ROOT / "target" / "fleet" / "agent-tool.log"
    tool_log.parent.mkdir(parents=True, exist_ok=True)
    try:
        with (ROOT / "target" / f"fleet-{mod}.log").open("a", encoding="utf-8") as log:
            while run < args.max_runs:
                cand = models.first_ready(chain)
                if not cand:
                    if not wait_for(models.earliest_reset(chain), "Every model is resting after a quota or rate limit"):
                        print("Stopped by `make fleet-stop`.")
                        return code
                    continue
                run += 1
                launches += 1
                # Follow the mod in progress: a rekkit_core bump the conductor started runs first.
                mod = beads.active_mod() or mod
                step = beads.mod_status(mod)["next"] if beads.epic(mod)["status"] != "open" else "step 1 Spec"
                prompt = (
                    f"Follow AGENTS.md. Act as the conductor (.agents/agents/conductor/agent.md) for the mod "
                    f"'{mod}' only. First run `make fleet-start MOD={mod}`. "
                    + ("" if launches == 1 else f"An earlier run ended before the mod was finished (perhaps on another "
                       f"model, out of credits or offline); continue from {step}. `make fleet-start` prints where it "
                       "stopped: file waiting reports and restart interrupted subagents on their beads. ")
                    + "Do not stop until it is finished: step 9 Package closed in Beads, or its epic blocked with the "
                      "reason as a comment if only a person can unblock it. Never push.")
                before = progress_mark()
                print(f"\n=== Fleet run {run}: {mod}, {step} ({cand.key})", flush=True)
                tool_log.unlink(missing_ok=True)
                cmd = [*shlex.split(cand.tool), prompt] if custom else agent_command(cand, chain, prompt, tool_log)
                code, tail = run_agent(cmd, log)
                try:
                    tool_text = tool_log.read_text(encoding="utf-8", errors="replace")
                except OSError:
                    tool_text = ""
                hit = models.limit_in(tail, tool_text)
                if hit:
                    until = models.rest(cand, *hit)
                    nxt = models.first_ready(chain)
                    print(f"{cand.key} hit a limit: {hit[0]}\nIt rests until {models.when(until)}; "
                          + (f"continuing on {nxt.key}." if nxt else "no other model is ready."), flush=True)
                elif not code:
                    models.ran_ok(cand)
                if STOP_FILE.is_file():
                    print("Stopped by `make fleet-stop`.")
                    return code
                st = beads.mod_status(mod)
                if st["finished"]:
                    print(f"{mod} is finished ({st['epic']['status']}; next: {st['next']}).")
                    return 0
                if not models.online():
                    run -= 1  # the network is down: nothing an agent can do, so it is not a stall either
                    print("The network is down; waiting for it to come back, then continuing on the same model.",
                          flush=True)
                    while not models.online():
                        if STOP_FILE.is_file():
                            print("Stopped by `make fleet-stop`.")
                            return code
                        time.sleep(30)
                    continue
                if hit and progress_mark() == before:
                    run -= 1  # cut short by a limit: neither a stall nor one of --max-runs
                    limited += 1
                    if limited > 10 * len(chain):
                        print("Every run ends on a limit; stopping. `make fleet-models` shows what is resting.")
                        return 1
                    continue
                limited = 0
                stalled = stalled + 1 if progress_mark() == before else 0
                if stalled >= MAX_STALLED:
                    print(f"{MAX_STALLED} runs in a row changed nothing in Beads; stopping. See target/fleet-{mod}.log.")
                    return 1
                if code:
                    time.sleep(min(60 * stalled, 300))  # an agent error (network, crash): back off
            print(f"Stopped after {args.max_runs} runs (--max-runs); {mod} is not finished.")
            return 1
    finally:
        LOCK.rmdir()


if __name__ == "__main__":
    raise SystemExit(main())
