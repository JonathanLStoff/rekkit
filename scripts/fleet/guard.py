#!/usr/bin/env python3
"""The fleet's guards, for any agent tool.

Checks a hook runs:
  shell  a shell command an agent is about to run: no publishing, signing, submodule changes, history
         rewrites or recursive deletes (a human does those). It notices `make fleet-start MOD=<id>`
         (this session conducts that mod) and `make fleet-task BEAD=<id> ROLE=<role>` (this session
         works one bead as that role). A role subagent may not write to Beads (it reports instead), and
         a builder may not read the decompiled Java (clean-room: builders work from the spec).
  paths  a file an agent is about to write: horizon/ is a read-only submodule, mods/<id>/content/ holds
         data only (no Cargo.toml, no .rs), and the harness's list of accepted warnings is the human's.
         It records which mods and crates the session wrote.
  read   a file a builder is about to read: not under assets/decompiled/.
  gate   before an agent (or, in Claude Code, a subagent) stops:
           - a role subagent has written its report for every bead it took (target/fleet/inbox/);
           - the conductor of a fleet run has filed every report (`make fleet-inbox`) and its mod is
             finished in Beads: stories 1-9 closed, or the epic blocked with a reason;
           - outside a fleet run, tests and checks pass for what the session wrote.
         The agent may stop when the user cancels it, after `make fleet-stop`, or after MAX_IDLE_STOPS
         blocked stops in a row with no change in Beads (a stuck agent).
  prime  at session start: `bd prime` (Beads' workflow and state) for the agent's context.

Commands for people and agents:
  claim <id>   what `make fleet-start MOD=<id>` runs first: checks the mod and clears a stop request
  task <bead>  what `make fleet-task BEAD=<id> ROLE=<role>` runs: prints where the report goes
  stop         what `make fleet-stop` runs: every fleet session may stop at its next turn end

Formats: plain (by hand: `make agent-gate`; exit 1 on a failure), agy (.agents/hooks.json: JSON on
stdout), claude (.claude/settings.json: exit 2 with the reason on stderr blocks; Claude Code never runs
the Stop hook when the user interrupts).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

import beads

ROOT = Path(__file__).resolve().parents[2]  # this checkout (a worktree's own files)
MAIN = beads.MAIN  # the main checkout: shared state, inbox, stop request
STATE = MAIN / "target" / "agent-gate"
STOP_FILE = MAIN / "target" / "fleet.stop"
ALLOWED_WARNINGS = "planning/harness-allowed-warnings.txt"
BLOCKED_SHELL = re.compile(
    r"git push|gh release|minisign|git submodule|git -C horizon|git reset --hard|--force|rm -rf")
BEADS_WRITE = re.compile(r"\bbd\s+(create|update|close|reopen|comments?|dep|defer|undefer|label|delete|edit|remember)\b")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1[^\n]*\n.*?^\s*\2\s*$", re.S | re.M)
FLEET_START = re.compile(r"(?:make\s+fleet-start\s+MOD=|guard\.py\s+claim\s+)['\"]?([a-z0-9_]+)")
FLEET_TASK = re.compile(r"(?:make\s+fleet-task|guard\.py\s+task)\s+(?:BEAD=)?['\"]?([A-Za-z0-9_.-]+)['\"]?"
                        r"(?:\s+(?:ROLE=|--role\s+)['\"]?([a-z-]+))?")
# Roles that write the mod from the spec and must never see the original's code.
BUILDERS = {"scaffolder", "domain-builder", "domain-builder-pro", "content-author", "hook-engineer", "test-smith"}
DECOMPILED = "assets/decompiled"
MAX_IDLE_STOPS = 10
# agy's reasons a run ended that the gate must not override (the user, or agy's own limits).
AGY_FINAL = {"USER_CANCELED", "MAX_FORCED_INVOCATIONS", "MAX_TOKEN_BUDGET_EXCEEDED", "ERROR"}
REPORT_SHAPE = ('{"bead": "<id>", "role": "<role>", "status": "done|partial|blocked|failed", "summary": "...", '
                '"files": ["..."], "notes": "...", "follow_ups": [{"title": "...", "type": "bug|feature", "why": "..."}]}')


# ---------------------------------------------------------------- session state

def session_id(payload: dict) -> str | None:
    """The session, or in Claude Code a subagent of it, the hook was called for."""
    raw = payload.get("agent_id") or payload.get("session_id") or payload.get("conversationId")
    return (re.sub(r"[^A-Za-z0-9_-]", "", str(raw))[:64] or None) if raw else None


class Session:
    """What the hooks know about one agent session: its fleet mod or tasks, what it wrote, its stops."""

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
        self.tasks: list[str] = data.get("tasks", [])
        self.role: str | None = data.get("role")
        self.touched: set[str] = set(data.get("touched", []))
        self.idle_stops: int = data.get("idle_stops", 0)
        self.progress: str = data.get("progress", "")

    def save(self) -> None:
        if not self.path:
            return
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(json.dumps({"fleet": self.fleet, "tasks": self.tasks, "role": self.role,
                                         "touched": sorted(self.touched), "idle_stops": self.idle_stops,
                                         "progress": self.progress}), encoding="utf-8")


# ---------------------------------------------------------------- payloads

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


def repo_paths(payload: dict) -> list[str]:
    """Paths the tool call names, relative to the checkout, for those inside it."""
    out = []
    for value in strings_named(tool_input(payload), lambda k: "path" in k or k in ("file", "target", "targetfile")):
        target = Path(value)
        full = (target if target.is_absolute() else ROOT / target).resolve().as_posix()
        for base in {ROOT.as_posix(), MAIN.as_posix()}:
            # macOS and Windows volumes are usually case-insensitive, so compare prefixes that way.
            if full.lower() == base.lower():
                out.append("")
            elif full.lower().startswith(base.lower() + "/"):
                out.append(full[len(base) + 1:])
    return out


def owner(rel: str) -> str | None:
    """mods/<id> or crates/<name> for a path inside one."""
    parts = rel.split("/")
    return "/".join(parts[:2]) if len(parts) >= 3 and parts[0] in ("mods", "crates") else None


# ---------------------------------------------------------------- shell, paths, read

def check_shell(payload: dict) -> str | None:
    commands = strings_named(tool_input(payload), lambda k: k in ("command", "commandline", "cmd"))
    # A heredoc's body is text being written, not commands being run.
    text = HEREDOC.sub("", "\n".join(commands)) if commands else json.dumps(payload)
    if BLOCKED_SHELL.search(text):
        return ("Blocked by the Rekkit fleet guard: pushing, releasing, signing, submodule changes, "
                "history rewrites and recursive deletes are for the human. Ask them to run it.")
    session = Session(payload)
    if m := FLEET_TASK.search(text):
        if m.group(1) not in session.tasks:
            session.tasks.append(m.group(1))
        session.role = m.group(2) or session.role
        session.save()
    # A session that conducts keeps the conductor's rights even if a subagent shares its id.
    if session.tasks and not session.fleet and BEADS_WRITE.search(text):
        return ("Only the conductor writes to Beads. Put this in your report instead (follow_ups, notes): "
                f"{INBOX_HINT}")
    if session.role in BUILDERS and not session.fleet and DECOMPILED in text:
        return f"Clean-room: a {session.role} works from planning/mods/<id>.md, never from {DECOMPILED}/."
    if m := FLEET_START.search(text):
        session.fleet, session.idle_stops, session.progress = m.group(1), 0, ""
        session.save()
    return None


def check_paths(payload: dict) -> str | None:
    units = set()
    for rel in repo_paths(payload):
        if rel == "horizon" or rel.startswith("horizon/"):
            return "horizon/ is read-only. File the gap as a Beads bug labelled horizon (via your report)."
        if re.match(r"mods/[^/]+/content/", rel) and (rel.endswith("Cargo.toml") or rel.endswith(".rs")):
            return "No Cargo.toml or .rs files under mods/<id>/content/."
        if rel == ALLOWED_WARNINGS:
            return f"{ALLOWED_WARNINGS} is changed only by a person. Report the warning instead."
        if unit := owner(rel):
            units.add(unit)
    session = Session(payload)
    if session.id and not units <= session.touched:
        session.touched |= units
        session.save()
    return None


def check_read(payload: dict) -> str | None:
    session = Session(payload)
    if session.role in BUILDERS and not session.fleet and any(p.startswith(DECOMPILED) for p in repo_paths(payload)):
        return f"Clean-room: a {session.role} works from planning/mods/<id>.md, never from {DECOMPILED}/."
    return None


# ---------------------------------------------------------------- gate

INBOX_HINT = f"write your report to {beads.INBOX}/<bead>.json as {REPORT_SHAPE}"


def reported(bead: str) -> bool:
    return (beads.INBOX / f"{bead}.json").is_file() or any((beads.INBOX / "done").glob(f"{bead}-*.json"))


def uncommitted() -> set[str]:
    def lines(*args: str) -> list[str]:
        r = subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True)
        return r.stdout.splitlines() if r.returncode == 0 else []
    changed = lines("diff", "--name-only", "HEAD") + lines("ls-files", "--others", "--exclude-standard")
    return {u for p in changed if (u := owner(p))}


def run_checks(units: set[str]) -> tuple[list[str], str]:
    """Failed mods/crates (of `units`, as mods/<id> and crates/<name>) and the end of their output: the
    syntax checks (rustfmt, clippy -D warnings, tests) and check-mods."""
    py = sys.executable
    steps: list[tuple[str, list]] = []
    for unit in sorted(units):
        kind, name = unit.split("/")
        manifest = f"{unit}/Cargo.toml"
        if not (ROOT / manifest).is_file():
            continue
        steps += [(unit, ["cargo", "fmt", "--manifest-path", manifest, "--check"]),
                  (unit, ["cargo", "clippy", "--manifest-path", manifest, "--all-targets", "--", "-D", "warnings"]),
                  (unit, ["cargo", "test", "--manifest-path", manifest])]
        bundle = ROOT / unit / "native" / "mixins" / name
        if kind == "mods" and bundle.is_dir():
            # The mixin helpers only compile inside Horizon (the harness build); rustfmt still parses them.
            rs = sorted(str(p) for p in bundle.rglob("*.rs"))
            if rs:
                steps.append((unit, ["rustfmt", "--check", "--edition", "2024", *rs]))
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


def progress_mark() -> str:
    """Changes whenever Beads or the inbox changes, so a working agent is never taken for a stuck one."""
    newest = max((p.stat().st_mtime_ns for p in (beads.BEADS_DIR / "embeddeddolt").rglob("*") if p.is_file()),
                 default=0) if (beads.BEADS_DIR / "embeddeddolt").is_dir() else 0
    reports = sorted(p.name for p in beads.INBOX.rglob("*.json")) if beads.INBOX.is_dir() else []
    return hashlib.sha256(f"{newest}{reports}".encode()).hexdigest()[:16]


def gate(payload: dict, units: set[str] | None) -> str | None:
    """Why the agent must keep working, or None when it may stop."""
    session = Session(payload)
    if payload.get("terminationReason", "") in AGY_FINAL or STOP_FILE.is_file():
        if session.fleet:
            session.fleet = None
            session.save()
        return None

    problems = []
    if session.tasks:
        missing = [t for t in session.tasks if not reported(t)]
        if missing:
            problems.append(f"Before you stop, report on {', '.join(missing)}: {INBOX_HINT}. Report partial or "
                            "failed work honestly; the conductor decides what happens next.")
    if session.fleet:
        waiting = sorted(p.stem for p in beads.INBOX.glob("*.json")) if beads.INBOX.is_dir() else []
        if waiting:
            problems.append(f"Unfiled subagent reports ({', '.join(waiting)}): run `make fleet-inbox`, then act on them.")
        try:
            st = beads.mod_status(session.fleet)
            if not st["finished"]:
                problems.append(
                    f"{session.fleet} is not finished (next: {st['next']}). Keep working: do not stop to report "
                    "progress or to ask whether to continue. Stop only when step 9 Package is closed, when the "
                    "user tells you to, or when only a person can unblock it: then set the epic to blocked with "
                    "the reason as a comment.")
            else:
                session.fleet = None
        except beads.BeadsError as exc:
            problems.append(f"Could not read {session.fleet} in Beads: {exc}")
        if payload.get("fullyIdle") is False and session.fleet:
            problems.append("Subagents or background tasks are still running: wait for them and file their reports.")
    if not session.tasks and not session.fleet:
        failed, log = run_checks(units if units is not None else session.touched)
        if failed:
            problems.append(f"Checks fail for {', '.join(failed)}; fix them before stopping.\n{log}")

    if not problems:
        session.idle_stops = 0
        session.save()
        return None
    mark = progress_mark()
    session.idle_stops = 1 if mark != session.progress else session.idle_stops + 1
    session.progress = mark
    session.save()
    if session.idle_stops > MAX_IDLE_STOPS:
        return None  # stuck: nothing changed in Beads or the inbox for MAX_IDLE_STOPS turns
    return "\n\n".join(problems)


def prime(fmt: str) -> int:
    text = ""
    if beads.available():
        try:
            text = beads.bd("prime", parse=False).strip()
        except beads.BeadsError:
            text = ""
    if not text:
        return 0
    if fmt == "agy":
        print(json.dumps({"injectSteps": [{"systemMessage": {"systemMessage": text}}]}))
    else:
        print(text)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("check", choices=["shell", "paths", "read", "gate", "prime", "claim", "task", "stop"])
    parser.add_argument("target", nargs="?", help="claim: the mod; task: the bead")
    parser.add_argument("--role", help="task: the role working the bead")
    parser.add_argument("--format", choices=["plain", "agy", "claude"], default="plain")
    parser.add_argument("--mod", action="append", default=[],
                        help="gate: these mod ids (or crates/<name>) instead of every uncommitted one")
    args = parser.parse_args()

    if args.check == "claim":
        try:
            beads.epic(args.target or "")
        except beads.BeadsError as exc:
            print(exc, file=sys.stderr)
            return 1
        STOP_FILE.unlink(missing_ok=True)
        print(f"Fleet run on {args.target}. Agents with hooks keep working until it is finished; "
              "`make fleet-stop` stops them.")
        return 0
    if args.check == "task":
        if not args.target:
            print("make fleet-task BEAD=<id> ROLE=<role>", file=sys.stderr)
            return 1
        beads.INBOX.mkdir(parents=True, exist_ok=True)
        print(f"You work {args.target}{' as ' + args.role if args.role else ''}. When you finish (or cannot go on), "
              f"write {beads.INBOX / (args.target + '.json')} as {REPORT_SHAPE}. Do not write to Beads yourself.")
        return 0
    if args.check == "stop":
        STOP_FILE.parent.mkdir(parents=True, exist_ok=True)
        STOP_FILE.write_text("stop\n", encoding="utf-8")
        print("Fleet agents may stop at the end of their current turn; headless runs end after it. "
              "`make fleet-start MOD=<id>` clears this.")
        return 0
    if args.check == "prime":
        return prime(args.format)

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

    check = {"shell": check_shell, "paths": check_paths, "read": check_read}[args.check]
    reason = check(payload)
    if args.format == "agy":
        print(json.dumps({"decision": "deny", "reason": reason} if reason else {"decision": "allow"}))
        return 0
    if reason:
        print(reason, file=sys.stderr)
        return 2 if args.format == "claude" else 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
