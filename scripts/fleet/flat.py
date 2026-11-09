#!/usr/bin/env python3
"""Flat mode: no subagents, one linear session, to save tokens.

Every subagent pays the tool's fixed floor (system prompt, tools, AGENTS.md, its role file) on every
turn, and the conductor pays for each report again. In flat mode the one session you started does
every role itself, one ticket after another, from `.agents/agents/<role>/agent.md`, still writing
each report to `target/fleet/inbox/` and filing it with `make fleet-inbox`. Nothing is dispatched.

The setting is `REKKIT_FLAT=1` in the environment or in `.env` (the environment wins; `.env` is
read here too, because hooks do not go through make). Where it is enforced:
  - the spawn hook (Claude Code `Agent|Task`, agy `invoke_subagent`, opencode `subagent|task`) denies;
  - `make fleet-dispatch` starts nothing and says how to work the tickets yourself;
  - `make fleet-task` tells the session it is working the ticket in its own context (no stop-at-report).

Usage:
  make fleet-flat [MODE=on|off|status]     (default: status)
  python3 scripts/fleet/flat.py [on|off|status]
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ENV = ROOT / ".env"
KEY = "REKKIT_FLAT"
TRUE = {"1", "true", "yes", "on"}

HOW = ("Flat mode (REKKIT_FLAT=1): no subagents. Do every role yourself, one ticket at a time: "
       "`make fleet-ready MOD=<id>`, then `make fleet-task BEAD=<id> ROLE=<role>`, do the work, write "
       "the report to target/fleet/inbox/<bead>.json, `make fleet-inbox`. Keep the main context small: "
       "read a ticket's brief only, never earlier tickets' files. `make fleet-flat MODE=off` turns it off.")


def _env_file() -> str | None:
    try:
        for line in ENV.read_text(encoding="utf-8").splitlines():
            m = re.match(rf"\s*{KEY}\s*=\s*(.*?)\s*(#.*)?$", line)
            if m:
                return m.group(1).strip("\"'")
    except OSError:
        pass
    return None


def is_flat() -> bool:
    value = os.environ.get(KEY)
    if value is None:
        value = _env_file()
    return (value or "").strip().lower() in TRUE


def set_flat(on: bool) -> None:
    text = ENV.read_text(encoding="utf-8") if ENV.is_file() else ""
    lines = [ln for ln in text.splitlines() if not re.match(rf"\s*{KEY}\s*=", ln)]
    lines.append(f"{KEY}={'1' if on else '0'}")
    ENV.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    mode = (sys.argv[1] if len(sys.argv) > 1 else "status").lower()
    if mode in ("on", "off"):
        set_flat(mode == "on")
    elif mode != "status":
        print(__doc__)
        return 1
    state = "ON" if is_flat() else "off"
    note = " (the environment overrides .env)" if os.environ.get(KEY) is not None else ""
    print(f"flat mode: {state}{note}")
    if is_flat():
        print(HOW)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
