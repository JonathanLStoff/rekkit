#!/usr/bin/env python3
"""The fleet's guards, for any agent tool.

Checks a hook runs:
  shell  a shell command an agent is about to run: no publishing, signing, submodule changes,
         history rewrites or recursive deletes (a human does those). It also notices
         `make fleet-start MOD=<id>` and ties that session to the board row.
  paths  a file an agent is about to write: horizon/ is a read-only submodule, and mods/<id>/content/
         holds data only (no Cargo.toml, no .rs). It records which mods and crates the session wrote.
  gate   before an agent stops:
           1. cargo test, check-mods and the harness dry run pass for the mods and crates the session
              wrote to (by hand: every uncommitted one, or --mod);
           2. a session that started a fleet run keeps working until its board row is finished: release
              prep done (step 9 waiting on the human), `done`, or `blocked` with a reason in Notes.
         It lets the agent stop when the user cancels it, after `make fleet-stop`, or after
         MAX_IDLE_STOPS blocked stops in a row with no change to the board (a stuck agent).

Commands for people and agents:
  claim <id>   what `make fleet-start MOD=<id>` runs: checks the row and clears an old stop request
  stop         what `make fleet-stop` runs: every fleet session may stop at its next turn end

Formats (how a hook's verdict is reported):
  plain   by hand (`make agent-gate`); exit 1 on a failure
  agy     Antigravity hooks (.agents/hooks.json): JSON {"decision": ...} on stdout
  claude  Claude Code hooks (.claude/settings.json): exit 2 with the reason on stderr blocks the
          tool call or the stop (Claude Code never runs the Stop hook when the user interrupts)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BOARD = ROOT / "planning" / "fleet-board.md"
STATE = ROOT / "target" / "agent-gate"
STOP_FILE = ROOT / "target" / "fleet.stop"
BLOCKED_SHELL = re.compile(
    r"git push|gh release|minisign|git submodule|git -C horizon|git reset --hard|--force|rm -rf")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1[^\n]*\n.*?^\s*\2\s*$", re.S | re.M)
FLEET_START = re.compile(r"(?:make\s+fleet-start\s+MOD=|guard\.py\s+claim\s+)['\"]?([a-z0-9_]+)")
# Blocked stops in a row with no change to the board before the gate lets a stuck agent stop.
MAX_IDLE_STOPS = 10
# agy's reasons a run ended that the gate must not override (the user, or agy's own limits).
AGY_FINAL = {"USER_CANCELED", "MAX_FORCED_INVOCATIONS", "MAX_TOKEN_BUDGET_EXCEEDED", "ERROR"}
STEP_DONE = {"✓", "—", "◐"}


# ---------------------------------------------------------------- board

def board_rows() -> dict[str, list[str]]:
    """Mod id -> its cells (1 #, 2 Mod, 6 Deps, 7 State, 8-17 steps 1-10, 18 Notes)."""
    rows = {}
    if BOARD.is_file():
        for line in BOARD.read_text(encoding="utf-8").splitlines():
            cells = [c.strip() for c in line.split("|")]
            if len(cells) > 17 and cells[1].isdigit():
                rows[cells[2]] = cells
    return rows


def row_finished(cells: list[str]) -> bool:
    """Nothing left for an agent: done, blocked, or release prep done and waiting on the human."""
    return cells[7] in ("done", "blocked") or cells[16] in ("▶", "✓")


def next_step(cells: list[str]) -> str:
    for i, cell in enumerate(cells[8:18], start=1):
        if cell not in STEP_DONE:
            return f"step {i} ({cell})"
    return "closing out"


def board_digest() -> str:
    return hashlib.sha256(BOARD.read_bytes()).hexdigest()[:16] if BOARD.is_file() else ""


# ---------------------------------------------------------------- session state

def session_id(payload: dict) -> str | None:
    raw = payload.get("session_id") or payload.get("conversationId")
    return re.sub(r"[^A-Za-z0-9_-]", "", str(raw))[:64] or None if raw else None


class Session:
    """What the hooks know about one agent session: its fleet row, the units it wrote, its stops."""

    def __init__(self, payload: dict):
        self.id = session_id(payload)
        self.path = STATE / f"{self.id}.json" if self.id else None
        data = {}
        if self.path and self.path.is_file():
            try:
                data = json.loads(self.path.read_text(encoding="utf-8"))
            except ValueError:
                data = {}
        self.fleet: str | None = data.get("fleet")
        self.touched: set[str] = set(data.get("touched", []))
        self.idle_stops: int = data.get("idle_stops", 0)
        self.board: str = data.get("board", "")

    def save(self) -> None:
        if not self.path:
            return
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(json.dumps({"fleet": self.fleet, "touched": sorted(self.touched),
                                         "idle_stops": self.idle_stops, "board": self.board}), encoding="utf-8")


# ---------------------------------------------------------------- shell and paths

def read_payload() -> dict:
    if sys.stdin.isatty():
        return {}
    try:
        data = json.loads(sys.stdin.read() or "{}")
    except ValueError:
        return {}
    return data if isinstance(data, dict) else {}


def tool_input(payload: dict) -> object:
    # Claude Code: tool_input; agy: toolCall.args; older payloads: the arguments themselves.
    if "tool_input" in payload:
        return payload["tool_input"]
    if isinstance(payload.get("toolCall"), dict):
        return payload["toolCall"].get("args", {})
    return payload


def strings_named(node: object, want) -> list[str]:
    found = []
    if isinstance(node, dict):
        for key, value in node.items():
            if isinstance(value, str) and want(key.lower()):
                found.append(value)
            else:
                found += strings_named(value, want)
    elif isinstance(node, list):
        for item in node:
            found += strings_named(item, want)
    return found


def owner(rel: str) -> str | None:
    """mods/<id> or crates/<name> for a path inside one."""
    parts = rel.split("/")
    return "/".join(parts[:2]) if len(parts) >= 3 and parts[0] in ("mods", "crates") else None


def check_shell(payload: dict) -> str | None:
    commands = strings_named(tool_input(payload), lambda k: k in ("command", "commandline", "cmd"))
    # A heredoc's body is text being written, not commands being run.
    text = HEREDOC.sub("", "\n".join(commands)) if commands else json.dumps(payload)
    if BLOCKED_SHELL.search(text):
        return ("Blocked by the Rekkit fleet guard: pushing, releasing, signing, submodule changes, "
                "history rewrites and recursive deletes are for the human. Ask them to run it.")
    if (m := FLEET_START.search(text)) and m.group(1) in board_rows():
        session = Session(payload)
        session.fleet, session.idle_stops, session.board = m.group(1), 0, board_digest()
        session.save()
    return None


def check_paths(payload: dict) -> str | None:
    # Keys that name a file, not its contents.
    raw = strings_named(tool_input(payload), lambda k: "path" in k or k in ("file", "target", "targetfile"))
    base = ROOT.as_posix().lower()
    units = set()
    for value in raw:
        target = Path(value)
        target = (target if target.is_absolute() else ROOT / target).resolve()
        full = target.as_posix()
        # macOS and Windows volumes are usually case-insensitive, so compare prefixes that way.
        if not full.lower().startswith(base + "/"):
            continue
        rel = full[len(base) + 1:]
        if rel == "horizon" or rel.startswith("horizon/"):
            return "horizon/ is read-only. Log the gap in planning/horizon-gaps.md."
        if re.match(r"mods/[^/]+/content/", rel) and (rel.endswith("Cargo.toml") or rel.endswith(".rs")):
            return "No Cargo.toml or .rs files under mods/<id>/content/."
        if unit := owner(rel):
            units.add(unit)
    session = Session(payload)
    if session.id and not units <= session.touched:
        session.touched |= units
        session.save()
    return None


# ---------------------------------------------------------------- gate

def uncommitted() -> set[str]:
    def lines(*args: str) -> list[str]:
        r = subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True)
        return r.stdout.splitlines() if r.returncode == 0 else []
    changed = lines("diff", "--name-only", "HEAD") + lines("ls-files", "--others", "--exclude-standard")
    return {u for p in changed if (u := owner(p))}


def run_checks(units: set[str]) -> tuple[list[str], str]:
    """Failed mods/crates (of `units`, as mods/<id> and crates/<name>) and the end of their output."""
    py = sys.executable
    steps: list[tuple[str, list]] = []
    for unit in sorted(units):
        kind, name = unit.split("/")
        if not (ROOT / unit / "Cargo.toml").is_file():
            continue
        steps.append((unit, ["cargo", "test", "--manifest-path", f"{unit}/Cargo.toml"]))
        if kind == "mods":
            steps += [(unit, [py, "scripts/pack_mod.py", "--check-only", "--mod", name]),
                      (unit, [py, "scripts/e2e_harness.py", "--phase", "isolated", "--mod", name, "--dry-run"])]
    failed: list[str] = []
    log: list[str] = []
    for unit, cmd in steps:
        if unit in failed:
            continue
        r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace")
        if r.returncode:
            failed.append(unit)
            log += [f"$ {' '.join(map(str, cmd))}", *(r.stdout + r.stderr).strip().splitlines()[-40:]]
    return failed, "\n".join(log[-60:])


def gate(payload: dict, units: set[str] | None) -> str | None:
    """Why the agent must keep working, or None when it may stop."""
    session = Session(payload)
    reason = payload.get("terminationReason", "")
    if reason in AGY_FINAL or STOP_FILE.is_file():
        if session.fleet:
            session.fleet = None
            session.save()
        return None

    problems = []
    failed, log = run_checks(units if units is not None else session.touched)
    if failed:
        problems.append(f"Checks fail for {', '.join(failed)}; fix them before stopping.\n{log}")
    if session.fleet:
        cells = board_rows().get(session.fleet)
        if cells and not row_finished(cells):
            problems.append(
                f"Board row {session.fleet} is not finished (next: {next_step(cells)}). Keep working: do not stop "
                "to report progress or to ask whether to continue. Stop only when release prep is done (steps 9-10 "
                "▶), when the user tells you to, or when only a person can unblock it: then set the row to "
                "`blocked` with the reason in Notes.")
        else:
            session.fleet = None
    if payload.get("fullyIdle") is False and session.fleet:
        problems.append("Subagents or background tasks are still running: wait for them and merge their work.")

    if not problems:
        session.idle_stops = 0
        session.save()
        return None
    digest = board_digest()
    session.idle_stops = 1 if digest != session.board else session.idle_stops + 1
    session.board = digest
    session.save()
    if session.idle_stops > MAX_IDLE_STOPS:
        return None  # stuck: no board change in MAX_IDLE_STOPS turns
    return "\n\n".join(problems)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("check", choices=["shell", "paths", "gate", "claim", "stop"])
    parser.add_argument("mod_id", nargs="?", help="claim: the board row")
    parser.add_argument("--format", choices=["plain", "agy", "claude"], default="plain")
    parser.add_argument("--mod", action="append", default=[],
                        help="gate: these mod ids (or crates/<name>) instead of every uncommitted one")
    args = parser.parse_args()

    if args.check == "claim":
        rows = board_rows()
        if args.mod_id not in rows:
            print(f"No board row {args.mod_id!r} in planning/fleet-board.md.", file=sys.stderr)
            return 1
        STOP_FILE.unlink(missing_ok=True)
        print(f"Fleet run on {args.mod_id} ({rows[args.mod_id][7]}, next: {next_step(rows[args.mod_id])}). "
              "Agents with hooks keep working until the row is finished; `make fleet-stop` stops them.")
        return 0
    if args.check == "stop":
        STOP_FILE.parent.mkdir(parents=True, exist_ok=True)
        STOP_FILE.write_text("stop\n", encoding="utf-8")
        print("Fleet agents may stop at the end of their current turn; headless runs end after it. "
              "`make fleet-start MOD=<id>` clears this.")
        return 0

    payload = read_payload() if args.format != "plain" else {}

    if args.check == "gate":
        units = None
        if args.mod:
            units = {m if "/" in m else f"mods/{m}" for v in args.mod for m in v.split(",") if m}
        elif args.format == "plain" or not session_id(payload):
            units = uncommitted()
        if args.format == "plain":
            failed, log = run_checks(units)
            print(f"Checks fail for {', '.join(failed)}.\n{log}" if failed else
                  f"Gate passed ({', '.join(sorted(units)) or 'no changes under mods/ or crates/'}).")
            return 1 if failed else 0
        reason = gate(payload, units)
        if args.format == "agy":
            print(json.dumps({"decision": "continue", "reason": reason} if reason else {"decision": "allow"}))
            return 0
        if reason:
            print(reason, file=sys.stderr)
            return 2
        return 0

    reason = check_shell(payload) if args.check == "shell" else check_paths(payload)
    if args.format == "agy":
        print(json.dumps({"decision": "deny", "reason": reason} if reason else {"decision": "allow"}))
        return 0
    if reason:
        print(reason, file=sys.stderr)
        return 2 if args.format == "claude" else 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
