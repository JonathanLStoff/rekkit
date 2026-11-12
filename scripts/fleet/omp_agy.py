#!/usr/bin/env python3
"""agy picks the next ready ticket, writes its subagent prompt file, and runs it on omp without thinking.

agy is the orchestrator for one run: it finds the ticket (`make fleet-ready`), writes
target/fleet/prompts/<bead>.md from the ticket's brief (`make fleet-task`), starts oh-my-pi on that file
with the local gemma model and --thinking=off, then files the report (`make fleet-inbox`). One ticket per run.

Usage:
  make omp-agy MOD=<id>
  python3 scripts/fleet/omp_agy.py <id>
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

import agy

ROOT = Path(__file__).resolve().parents[2]
MODEL = "ollama/gemma4:26b-mlx"


def prompt(mod: str) -> str:
    return f"""You are the orchestrator for the mod {mod} in this repository. Do these steps in order, and nothing else:
1. Run `make fleet-ready MOD={mod}`. If it says no ticket is ready, stop and say so.
2. Take the first ticket it lists: its id (BEAD) and title. Its role is the role: label; use domain-builder if it has none.
3. Run `make fleet-task BEAD=<BEAD> ROLE=<ROLE>` and write its whole output to target/fleet/prompts/<BEAD>.md, with this line first: "You are the <ROLE> subagent for the mod {mod}. Your bead is <BEAD>. Do the ticket, write your report to target/fleet/inbox/<BEAD>.json, then stop. Never run bd, git push, or anything a person does."
4. Run `omp -p --cwd {ROOT} --model {MODEL} --thinking=off @target/fleet/prompts/<BEAD>.md` and wait for it to finish.
5. Run `make fleet-inbox`, then stop.
Never edit files under mods/ or crates/ yourself, and never run bd."""


def main() -> int:
    if len(sys.argv) != 2:
        sys.exit("usage: omp_agy.py <mod>")
    mod = sys.argv[1]
    for tool in ("agy", "omp"):
        if not shutil.which(tool):
            sys.exit(f"{tool} is not installed or not on PATH.")
    return subprocess.call(agy.command(["-p", prompt(mod)]), cwd=ROOT)


if __name__ == "__main__":
    raise SystemExit(main())
