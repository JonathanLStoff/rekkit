#!/usr/bin/env python3
"""The fleet's guards, for any agent tool.

Checks a hook runs:
  shell  a shell command an agent is about to run: no publishing, signing, submodule changes, history
         rewrites or recursive deletes (a human does those). It notices `make fleet-start MOD=<id>`
         (this session conducts that mod) and `make fleet-task BEAD=<id> ROLE=<role>` (this session
         works one bead as that role). A role subagent may not run bd at all (it reports instead).
         Clean-room, both ways: a coder (a builder role, or a session that wrote under mods/ or crates/)
         never touches the original's Java (assets/decompiled/, .java/.class under assets/, make
         decompile, make java-*), and a session that has read it never writes mods/ or crates/.
         In agy every command also gets DOLT_ROOT_PATH (target/dolt-root), so `bd` also works where
         ~/.dolt is unreadable (agy's terminal sandbox, if a person turns it back on).
  paths  a file an agent is about to write: horizon/ is a read-only submodule, mods/<id>/content/ holds
         data only (no Cargo.toml, no .rs), the harness's list of accepted warnings is the human's, and
         a session that has read the Java writes no mods/ or crates/. It records what the session wrote.
  read   a file an agent is about to read: a coder never reads the original's Java (clean-room), and
         nobody reads more than MAX_JAVA_READ lines of a decompiled class at once (make java-outline).
  gate   before an agent (or, in Claude Code, a subagent) stops:
           - a role subagent has written its report for every bead it took (target/fleet/inbox/);
           - the conductor of a fleet run has filed every report (`make fleet-inbox`) and its mod is
             finished in Beads: stories 1-9 closed, or the epic blocked with a reason;
           - outside a fleet run, tests and checks pass for what the session wrote.
         While a conductor's subagents are still working, the gate waits (up to WAIT_FOR_REPORTS) for
         a report instead of sending it straight back to look for work, and those waits do not count
         as idle. The agent may stop when the user cancels it, after `make fleet-stop`, or after
         MAX_IDLE_STOPS blocked stops in a row with no change in Beads (a stuck agent).
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
import os
import re
import shlex
import subprocess
import sys
import time
from pathlib import Path

import beads

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import version_sync  # noqa: E402 (scripts/, beside this folder)

ROOT = Path(__file__).resolve().parents[2]  # this checkout (a worktree's own files)
MAIN = beads.MAIN  # the main checkout: shared state, inbox, stop request
STATE = MAIN / "target" / "agent-gate"
STOP_FILE = MAIN / "target" / "fleet.stop"
ALLOWED_WARNINGS = "planning/harness-allowed-warnings.txt"
BLOCKED_SHELL = re.compile(
    r"git push|gh release|minisign|git submodule|git -C horizon|git reset --hard|--force|rm -rf")
BEADS_CMD = re.compile(r"(?<![\w./-])bd(?:\s|$)")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1[^\n]*\n.*?^\s*\2\s*$", re.S | re.M)
FLEET_START = re.compile(r"(?:make\s+fleet-start\s+MOD=|guard\.py\s+claim\s+)['\"]?([a-z0-9_]+)")
FLEET_TASK = re.compile(r"(?:make\s+fleet-task|guard\.py\s+task)\s+(?:BEAD=)?['\"]?([A-Za-z0-9_.-]+)['\"]?"
                        r"(?:\s+(?:ROLE=|--role\s+)['\"]?([a-z-]+))?")
# Roles that write the mod from the spec and must never see the original's code.
BUILDERS = {"scaffolder", "domain-builder", "domain-builder-pro", "content-author", "hook-engineer", "test-smith"}
DECOMPILED = "assets/decompiled"
# Shell commands that read or produce the original's Java.
JAVA_SHELL = re.compile(r"assets/decompiled|assets/\S*\.(?:java|class)\b|\bmake\s+(?:decompile|java-(?:outline|show|find))\b"
                        r"|scripts/(?:decompile|java_outline)\.py|\bjavap\b|vineflower")
CLEAN_ROOM = ("Clean-room: an agent that writes a mod's code never reads the original Java, and an agent that "
              "has read it never writes mods/ or crates/. Work from the spec (make spec-show MOD=<id> "
              "SECTION=...); report what it lacks and the spec-writer answers in the spec.")
MAX_IDLE_STOPS = 10
MAX_JAVA_READ = 200  # lines of decompiled Java one read may take
WAIT_FOR_REPORTS = 600  # seconds a conductor's turn end waits for running subagents (hook timeout: 900)
LIVE_WINDOW = 15 * 60  # a session with no tool call for this long is gone (died, out of credits, offline)
JOURNAL = beads.STATE_DIR / "journal.jsonl"  # every session's commands, writes and subagent launches
PROMPTS = beads.STATE_DIR / "prompts"  # the prompt each bead's subagent was started with
PROGRESS = beads.STATE_DIR / "progress"  # each bead's progress notes, one line per finished piece
BEAD_ID = re.compile(r"\b[a-z]{2,}-[a-z0-9]{3,}(?:\.\d+)+\b")
DOLT_ROOT = MAIN / "target" / "dolt-root"  # dolt's global config, inside the repo so agy's sandbox can read it
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
        self.read_java: bool = data.get("read_java", False)
        self.last_seen: float = data.get("last_seen", 0)

    @property
    def who(self) -> str:
        return "conductor" if self.fleet else self.role or ("subagent" if self.tasks else "agent")

    def seen(self) -> None:
        """Every hook call marks the session alive (a dead one stops being counted as running)."""
        if self.id:
            self.last_seen = time.time()
            self.save()

    @property
    def coder(self) -> bool:
        """A builder role, or a session that has written under mods/ or crates/."""
        return self.role in BUILDERS or bool(self.touched)

    def save(self) -> None:
        if not self.path:
            return
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(json.dumps({"fleet": self.fleet, "tasks": self.tasks, "role": self.role,
                                         "touched": sorted(self.touched), "idle_stops": self.idle_stops,
                                         "progress": self.progress, "read_java": self.read_java,
                                         "last_seen": self.last_seen}),
                                     encoding="utf-8")


def journal(session: Session, kind: str, text: str) -> None:
    """One line of the fleet journal: what `make fleet-resume` rebuilds a dead session's work from."""
    if not session.id:
        return
    JOURNAL.parent.mkdir(parents=True, exist_ok=True)
    entry = {"t": time.strftime("%Y-%m-%dT%H:%M:%S"), "s": session.id, "who": session.who,
             "fleet": session.fleet, "tasks": session.tasks, "kind": kind, "text": text[:300]}
    with JOURNAL.open("a", encoding="utf-8") as f:
        f.write(json.dumps(entry) + "\n")


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
    if session.tasks and not session.fleet and BEADS_CMD.search(text):
        return ("Role subagents do not use Beads (not even to read or for --help): your bead and what it needs "
                f"are in your prompt. Put anything for Beads in your report (follow_ups, notes): {INBOX_HINT}")
    if JAVA_SHELL.search(text):
        if session.coder:
            return CLEAN_ROOM
        if not session.read_java:
            session.read_java = True
            session.save()
    if m := FLEET_START.search(text):
        session.fleet, session.idle_stops, session.progress = m.group(1), 0, ""
        session.save()
    journal(session, "run", re.sub(r"^export DOLT_ROOT_PATH=\S+; ", "", text.strip()))
    return None


def earlier_work(bead: str, limit: int = 40) -> str:
    """What earlier sessions did on a bead: its progress notes and the files they wrote."""
    out = []
    notes = PROGRESS / f"{bead}.md"
    if notes.is_file():
        lines = [ln for ln in notes.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()]
        out += ["Progress notes:"] + [f"  {ln}" for ln in lines[-limit:]]
    files: dict[str, str] = {}
    if JOURNAL.is_file():
        for line in JOURNAL.read_text(encoding="utf-8", errors="replace").splitlines():
            try:
                e = json.loads(line)
            except ValueError:
                continue
            if e.get("kind") == "write" and bead in (e.get("tasks") or []):
                files[e["text"]] = e["t"]
    if files:
        out += ["Files written (last write):"] + [f"  {t}  {f}" for f, t in sorted(files.items(), key=lambda x: x[1])[-limit:]]
    return "\n".join(out)


def check_spawn(payload: dict) -> str | None:
    """A subagent is being started: keep its prompt under each bead it names, for a restart after a crash."""
    args = tool_input(payload)
    if not isinstance(args, dict):
        return None
    prompt = args.get("Prompt") or args.get("prompt") or ""
    role = args.get("TypeName") or args.get("subagent_type") or args.get("Role") or "subagent"
    session = Session(payload)
    found = sorted(set(BEAD_ID.findall(prompt)))
    PROMPTS.mkdir(parents=True, exist_ok=True)
    for bead in found:
        (PROMPTS / f"{bead}.md").write_text(f"<!-- role: {role}; started {time.strftime('%Y-%m-%dT%H:%M:%S')} -->\n"
                                            f"{prompt}\n", encoding="utf-8")
    journal(session, "spawn", f"{role} on {', '.join(found) or 'no bead'}")
    return None


def agy_env(payload: dict) -> dict:
    """The `overwrite` that gives an agy command DOLT_ROOT_PATH (see the module docstring)."""
    args = tool_input(payload)
    cmd = args.get("CommandLine") if isinstance(args, dict) else None
    if not isinstance(cmd, str) or os.name == "nt" or "DOLT_ROOT_PATH" in cmd:
        return {}
    DOLT_ROOT.mkdir(parents=True, exist_ok=True)
    return {"CommandLine": f"export DOLT_ROOT_PATH={shlex.quote(DOLT_ROOT.as_posix())}; {cmd}"}


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
    if units and session.read_java:
        return CLEAN_ROOM
    if session.id and not units <= session.touched:
        session.touched |= units
        session.save()
    for rel in repo_paths(payload):
        journal(session, "write", rel)
    return None


def java_path(rel: str) -> bool:
    """The original's Java: the decompiled tree, any .java or .class under assets/, or a search over assets/."""
    return rel == "assets" or rel.startswith(DECOMPILED) or (rel.startswith("assets/") and rel.endswith((".java", ".class")))


def check_read(payload: dict) -> str | None:
    session = Session(payload)
    paths = repo_paths(payload)
    if any(java_path(p) for p in paths):
        if session.coder:
            return CLEAN_ROOM
        if not session.read_java:
            session.read_java = True
            session.save()
    return whole_java_read(payload, paths)


def whole_java_read(payload: dict, paths: list[str]) -> str | None:
    """Whole decompiled classes fill an agent's context for the rest of its run: read them in pieces."""
    args = tool_input(payload)
    if not isinstance(args, dict):
        return None
    start = args.get("StartLine") or args.get("offset")
    end = args.get("EndLine") or ((start or 0) + args["limit"] if args.get("limit") else None)
    for rel in paths:
        full = MAIN / rel
        if not (rel.startswith(DECOMPILED) and rel.endswith(".java") and full.is_file()):
            continue
        size = full.read_text(encoding="utf-8", errors="replace").count("\n")
        span = (end - (start or 1) + 1) if end else size
        if size > MAX_JAVA_READ and span > MAX_JAVA_READ:
            folder = "/".join(rel.split("/")[:3])
            name = Path(rel).stem
            return (f"{rel} has {size} lines. Read decompiled Java in pieces: `make java-outline DIR={folder} "
                    f"GREP={name}` (members with line ranges), then `make java-show DIR={folder} "
                    f"MEMBER={name}#<method>`, or read at most {MAX_JAVA_READ} lines at a time.")
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
    version_sync.sync_units(sorted(u.split("/", 1)[1] for u in units))
    if "crates/rekkit_core" in units:
        # A core change must keep every mod that uses it building and passing.
        units = set(units) | {f"mods/{m.parent.name}" for m in (ROOT / "mods").glob("*/Cargo.toml")
                              if "rekkit_core" in m.read_text(encoding="utf-8", errors="replace")}
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


def running_tasks() -> list[str]:
    """Beads that subagents took (`make fleet-task`) and have not reported on yet, while they still act."""
    now, out = time.time(), []
    for f in STATE.glob("*.json") if STATE.is_dir() else []:
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if now - (data.get("last_seen") or f.stat().st_mtime) < LIVE_WINDOW:
            out += [t for t in data.get("tasks") or [] if not reported(t)]
    return sorted(set(out))


def wait_for_reports(fully_idle: object) -> list[str]:
    """Wait while subagents run and none has reported; the beads still running when it stops waiting."""
    end = time.time() + WAIT_FOR_REPORTS
    while True:
        running = running_tasks()
        if not (running or fully_idle is False) or STOP_FILE.is_file() \
                or (beads.INBOX.is_dir() and any(beads.INBOX.glob("*.json"))):
            return running
        if time.time() >= end:
            return running or ["background work"]
        time.sleep(min(10, max(end - time.time(), 0.5)))


def gate(payload: dict, units: set[str] | None) -> str | None:
    """Why the agent must keep working, or None when it may stop."""
    session = Session(payload)
    if session.touched:  # whatever ends this turn, what it changed is one version ahead of the last commit
        version_sync.sync_units(sorted(u.split("/", 1)[1] for u in session.touched))
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
    still_running: list[str] = []
    if session.fleet:
        # Spend the wait here, not in empty turns: return as soon as a report lands.
        still_running = wait_for_reports(payload.get("fullyIdle"))
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
        if still_running and session.fleet and not waiting:
            problems.append(
                f"Subagents are still working ({', '.join(still_running)}); this stop already waited "
                f"{WAIT_FOR_REPORTS // 60} minutes for them. Start any other step whose inputs are ready; if "
                "none is, end your turn again to keep waiting. Do not repeat status messages or invent work.")
    if not session.tasks and not session.fleet:
        failed, log = run_checks(units if units is not None else session.touched)
        if failed:
            problems.append(f"Checks fail for {', '.join(failed)}; fix them before stopping.\n{log}")

    if not problems:
        session.idle_stops = 0
        session.save()
        return None
    mark = progress_mark()
    if still_running and still_running != ["background work"]:
        session.progress = mark  # waiting on live subagents is not being stuck
        session.save()
        return "\n\n".join(problems)
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
    parser.add_argument("check", choices=["shell", "paths", "read", "spawn", "gate", "prime", "claim", "task", "stop"])
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
        notes = PROGRESS / f"{args.target}.md"
        rel = notes.relative_to(MAIN).as_posix()
        print(f"You work {args.target}{' as ' + args.role if args.role else ''}. After each piece of work you finish "
              f"(a file, a block, a test passing), append one line saying what is done to {rel}: if you are "
              "interrupted, the next agent continues from it. When you finish (or cannot go on), write "
              f"{beads.INBOX / (args.target + '.json')} as {REPORT_SHAPE}. Do not write to Beads yourself.")
        earlier = earlier_work(args.target)
        if earlier:
            print("\nAn earlier agent worked this bead and was interrupted. Continue from its work; do not redo "
                  "what is listed as done (check a file before rewriting it):\n" + earlier)
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

    check = {"shell": check_shell, "paths": check_paths, "read": check_read, "spawn": check_spawn}[args.check]
    reason = check(payload)
    Session(payload).seen()
    if args.format == "agy":
        if reason:
            print(json.dumps({"decision": "deny", "reason": reason}))
        else:
            env = agy_env(payload) if args.check == "shell" else {}
            print(json.dumps({"decision": "allow", **({"overwrite": env} if env else {})}))
        return 0
    if reason:
        print(reason, file=sys.stderr)
        return 2 if args.format == "claude" else 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
