#!/usr/bin/env python3
"""The fleet's guards, for any agent tool.

Checks a hook runs:
  shell  a shell command an agent is about to run: no publishing, signing, submodule changes, history
         rewrites or recursive deletes (a human does those). It notices `make fleet-start MOD=<id>`
         (this session conducts that mod) and `make fleet-task BEAD=<id> ROLE=<role>` (this session
         works one bead as that role). A role subagent may not run bd at all (it reports instead).
         In agy every command also gets DOLT_ROOT_PATH (target/dolt-root), so `bd` also works where
         ~/.dolt is unreadable (agy's terminal sandbox, if a person turns it back on).
         Token waste (planning/token_audit/fix-plan.md): a role subagent may not sweep folders
         (recursive listings, find, tree), run `git status`, `git log -p` or a bare `git diff`, search
         the decompiled Java or the spec with grep/rg (make java-find, spec-show), use more than 2 lines
         of search context, read the toolchain (scripts/, the Makefile, the people's docs) or repeat a
         `make java-*`/`spec-*` command it already ran. The conductor gets the same as warnings.
  paths  a file an agent is about to write: horizon/ is a read-only submodule, mods/<id>/content/ holds
         data only (no Cargo.toml, no .rs), and the harness's list of accepted warnings is the
         human's. It records what the session wrote.
  read   a file an agent is about to read: nobody reads more than MAX_JAVA_READ lines of a decompiled
         class at once (make java-outline, java-show). Anyone may read the Java; the spec comes first.
         A role subagent also may not read the toolchain, a whole spec or skill (spec-show,
         skill-show), search the Java or the spec with the search tool, glob the whole checkout, or
         read the same lines of an unchanged file twice.
  budget a role subagent stops working (only its notes and report) past TASK_TOKENS of context,
         TASK_BILLED tokens billed (the sum of every turn's context) or TASK_TURNS turns.
  gate   before an agent (or, in Claude Code, a subagent) stops:
           - a role subagent has written its report for every bead it took (target/fleet/inbox/);
           - the conductor of a fleet run has filed every report (`make fleet-inbox`) and its mod is
             finished in Beads: stories 1-9 closed, or the epic blocked with a reason;
           - outside a fleet run, tests and checks pass for what the session wrote.
         While a conductor's subagents are still working, the gate waits (up to WAIT_FOR_REPORTS) for
         a report instead of sending it straight back to look for work, and those waits do not count
         as idle. The agent may stop when the user cancels it, after `make fleet-stop`, or after
         MAX_IDLE_STOPS blocked stops in a row with no change in Beads (a stuck agent).
  prime  the Beads context: `bd prime` (.beads/PRIME.md) and, while a mod is in progress, its resume
         page. Run at session start and after compaction (Claude Code's SessionStart covers both; agy's
         SessionStart; opencode's plugin puts it in the main session's system prompt and on compaction),
         so no agent runs `bd prime` itself and role subagents get none of it.

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
    r"git push|gh release|minisign|git submodule|git reset --hard|--force|rm -rf|horizon[-_]pull|make\s+(?:-[Cf]\s*\S+\s+|-\S+\s+|\w+=\S*\s+)*pull(?![\w-])"
    # Git in the submodule only reads (grep, log, show...); anything else would change it.
    r"|git -C \S*horizon/?\s+(?!(?:grep|log|show|diff|status|rev-parse|ls-files|ls-tree|blame|cat-file|merge-base"
    r"|describe)\b)")
BEADS_CMD = re.compile(r"(?<![\w./-])bd(?:\s|$)")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1[^\n]*\n.*?^\s*\2\s*$", re.S | re.M)
# Commands only the conductor runs: a session resumed after a crash or /clear (fleet-resume) or that
# dispatches tickets conducts the mod as surely as one that ran fleet-start.
FLEET_START = re.compile(r"(?:make\s+fleet-(?:start|resume|dispatch)\s+(?:\w+=\S*\s+)*MOD=|guard\.py\s+claim\s+"
                         r"|resume\.py\s+|dispatch\.py\s+)['\"]?([a-z0-9_]+)")
FLEET_TASK = re.compile(r"(?:make\s+fleet-task|guard\.py\s+task)\s+(?:BEAD=)?['\"]?([A-Za-z0-9_.-]+)['\"]?"
                        r"(?:\s+(?:ROLE=|--role\s+)['\"]?([a-z-]+))?")
DECOMPILED = "assets/decompiled"
MAX_IDLE_STOPS = 10
MAX_JAVA_READ = 200  # lines of decompiled Java one read may take
MAX_SUBAGENTS = 2  # subagents working tickets at once; the conductor waits for a report before starting more
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
        self.last_seen: float = data.get("last_seen", 0)
        self.reads: dict[str, float] = data.get("reads", {})  # "path|start|end" -> the file's mtime then
        self.runs: list[str] = data.get("runs", [])  # read-only make commands already run (java-*, spec-*)
        self.usage: dict = data.get("usage", {})  # where the transcript was summed up to, and the sums

    @property
    def who(self) -> str:
        return "conductor" if self.fleet else self.role or ("subagent" if self.tasks else "agent")

    def seen(self) -> None:
        """Every hook call marks the session alive (a dead one stops being counted as running)."""
        if self.id:
            self.last_seen = time.time()
            self.save()

    def save(self) -> None:
        if not self.path:
            return
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(json.dumps({"fleet": self.fleet, "tasks": self.tasks, "role": self.role,
                                         "touched": sorted(self.touched), "idle_stops": self.idle_stops,
                                         "progress": self.progress,
                                         "last_seen": self.last_seen, "reads": self.reads,
                                         "runs": self.runs[-200:], "usage": self.usage}),
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
        # Windows decodes stdin with the ANSI code page; the tools send UTF-8.
        data = json.loads(sys.stdin.buffer.read().decode("utf-8", errors="replace") or "{}")
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

TASK_TOKENS = 200_000  # a role subagent's context budget; past it, it reports and a fresh agent goes on
# What a run costs is every turn's whole context added up, not its peak (planning/token_audit/
# performance.md: a "200k" ticket billed 1.5M over 46 turns). So a subagent also wraps up past these.
TASK_BILLED = int(os.environ.get("REKKIT_TASK_BILLED") or 500_000)
TASK_TURNS = int(os.environ.get("REKKIT_TASK_TURNS") or 60)  # model calls; context (TASK_TOKENS) is what bloats
# What a subagent over budget may still do: write its progress notes and its report.
WRAP_UP = re.compile(r"target/fleet/(?:inbox|progress)/")


def transcripts(payload: dict) -> list[Path]:
    """The session's own transcript: agy's brain log, or Claude Code's JSONL (a subagent's own file under
    <session>/subagents/)."""
    candidates = []
    if conv := payload.get("conversationId"):
        candidates.append(Path.home() / ".gemini" / "antigravity-cli" / "brain" / str(conv) / ".system_generated"
                          / "logs" / "transcript_full.jsonl")
    if payload.get("agent_transcript_path"):
        candidates.append(Path(payload["agent_transcript_path"]))
    if tp := payload.get("transcript_path"):
        if payload.get("agent_id"):
            candidates.append(Path(tp).with_suffix("") / "subagents" / f"agent-{payload['agent_id']}.jsonl")
        else:
            candidates.append(Path(tp))
    return candidates


CACHE_READ_WEIGHT = 0.1  # a cached input token costs about a tenth of a fresh one (Anthropic and Gemini)


def billed_tokens(fresh: int, cached: int) -> int:
    """What a model call costs, in fresh-input tokens: cached reads count at CACHE_READ_WEIGHT. Counting
    them in full stopped lean agents (50k of context) after 20 calls and made the fleet re-brief them."""
    return fresh + int(cached * CACHE_READ_WEIGHT)


def turn_usage(row: dict) -> tuple[str | None, int, int] | None:
    """(the model call's id, its whole context: fresh plus cache, what it billed) for a transcript row that
    is one, else None. Claude Code writes one row per content block of a reply, all with the same id."""
    if "input_tokens" in row:  # agy: one row a turn
        fresh, cached = int(row.get("input_tokens") or 0), int(row.get("cache_read_tokens") or 0)
        return None, fresh + cached, billed_tokens(fresh, cached)
    msg = row.get("message") if isinstance(row.get("message"), dict) else None
    usage = (msg or {}).get("usage")
    if usage:
        fresh = int(usage.get("input_tokens") or 0) + int(usage.get("cache_creation_input_tokens") or 0)
        cached = int(usage.get("cache_read_input_tokens") or 0)
        return msg.get("id"), fresh + cached, billed_tokens(fresh, cached)
    return None


def billed_so_far(payload: dict, session: "Session") -> tuple[int, int] | None:
    """(tokens billed: every turn's context added up, turns) for the session, read on from where the last
    hook call stopped (the sums are kept in the session's state)."""
    for path in transcripts(payload):
        try:
            size = path.stat().st_size
        except OSError:
            continue
        u = session.usage if session.usage.get("path") == str(path) else {"path": str(path), "offset": 0,
                                                                          "billed": 0, "turns": 0, "last": None}
        if size < u["offset"]:  # a new or rewritten transcript
            u = {"path": str(path), "offset": 0, "billed": 0, "turns": 0, "last": None}
        try:
            with path.open("rb") as f:
                f.seek(u["offset"])
                chunk = f.read()
        except OSError:
            continue
        end = chunk.rfind(b"\n") + 1  # only whole lines; the rest is read next time
        for line in chunk[:end].decode("utf-8", "replace").splitlines():
            try:
                row = json.loads(line)
            except ValueError:
                continue
            got = turn_usage(row) if isinstance(row, dict) else None
            if not got:
                continue
            mid, _, tokens = got
            if mid is not None and mid == u["last"]:
                continue
            u["last"] = mid
            u["billed"] += tokens
            u["turns"] += 1
        u["offset"] += end
        session.usage = u
        session.save()
        return u["billed"], u["turns"]
    return None


def context_tokens(payload: dict) -> int | None:
    """The session's context now (prompt plus cache), from its own transcript."""
    for path in transcripts(payload):
        try:
            with path.open("rb") as f:
                f.seek(max(0, path.stat().st_size - 400_000))
                lines = f.read().decode("utf-8", "replace").splitlines()
        except OSError:
            continue
        for line in reversed(lines):
            try:
                row = json.loads(line)
            except ValueError:
                continue
            got = turn_usage(row) if isinstance(row, dict) else None
            if got:
                return got[1]
    return None


def over_budget(payload: dict, session: "Session", text: str) -> str | None:
    """A role subagent past TASK_TOKENS of context, TASK_BILLED billed or TASK_TURNS turns may only wrap up:
    notes, then a partial report."""
    if not session.tasks or session.fleet or WRAP_UP.search(text):
        return None
    why = None
    used = context_tokens(payload)
    if used is not None and used > TASK_TOKENS:
        why = f"{used:,} tokens of context, of {TASK_TOKENS:,}"
    elif (sums := billed_so_far(payload, session)) is not None:
        billed, turns = sums
        if billed > TASK_BILLED:
            why = f"{billed:,} tokens billed over {turns} turns, of {TASK_BILLED:,}"
        elif turns > TASK_TURNS:
            why = f"{turns} turns, of {TASK_TURNS}"
    if not why:
        return None
    beads_list = ", ".join(session.tasks)
    return (f"Token budget reached ({why}): stop working now. Append what you finished to "
            f"target/fleet/progress/<bead>.md, then write your report for {beads_list} with status partial, "
            "listing exactly what is left; the conductor gives the rest to a fresh agent or splits the ticket.")


# ---------------------------------------------------------------- token waste (fix-plan A1-A3)

# A command between separators: what each pattern below looks at.
SEG = r"[^|;&\n]*"
SWEEPS = [
    (re.compile(rf"(?<![\w-])(?:Get-ChildItem|gci|ls|dir)\b{SEG}\s(?:-Recurse\b|-r\b|-\w*R\w*\b|/s\b)", re.I),
     "recursive listings"),
    (re.compile(r"(?<![\w-])find\s+(?!.*-maxdepth\s+1\b)[^|;&\n]*"), "find over a folder tree"),
    (re.compile(r"(?<![\w-])tree(?:\s|$)"), "tree"),
    (re.compile(r"\bgit\s+status\b"), "git status"),
    (re.compile(rf"\bgit\s+log\b{SEG}\s(?:-p\b|--patch\b)"), "git log -p"),
    (re.compile(r"\bgit\s+diff\b(?![^|;&\n]*(?:\s--\s|\s--stat\b|\s--name-only\b|\s--name-status\b))"),
     "a git diff with no path (`git diff --stat`, or `git diff -- <path>`)"),
]
SEARCH_VERB = r"(?<![\w-])(?:rg|grep|egrep|findstr|Select-String|sls)\b"
SEARCHES = [
    (re.compile(rf"{SEARCH_VERB}{SEG}assets[/\\]decompiled", re.I),
     "searching the decompiled Java: use `make java-find DIR=<decompiled> TEXT=<name>` "
     "(or java-outline GREP=<name>)"),
    (re.compile(rf"{SEARCH_VERB}{SEG}planning[/\\]mods[/\\][\w-]+\.md", re.I),
     "searching the spec: use `make spec-toc MOD=<id>`, then `make spec-show MOD=<id> SECTION=\"<heading>\"`"),
]
WIDE_CONTEXT = re.compile(rf"{SEARCH_VERB}{SEG}(?:\s-[CAB]\s*(\d+)|\s--context[= ](\d+)|\s-Context\s+(\d+))", re.I)
TOOLCHAIN_PATH = r"(?:scripts[/\\]|Makefile\b|\.agents[/\\](?:fleet-plan|reference)\.md)"
TOOLCHAIN_SHELL = re.compile(
    rf"(?<![\w-])(?:cat|type|Get-Content|gc|head|tail|less|more|sed|awk|rg|grep|Select-String|sls|findstr)\b{SEG}"
    rf"{TOOLCHAIN_PATH}|\bgit\s+(?:log|show|blame|diff)\b{SEG}{TOOLCHAIN_PATH}", re.I)
# Read-only commands whose output does not change during a run: running one twice only pays for it twice.
ONCE = re.compile(r"\bmake\s+(?:java-outline|java-show|java-find|spec-toc|spec-show|skill-toc|skill-show|"
                  r"conductor-step|fleet-context)\b[^|;&\n]*")
HOW_TO = ("`make help` lists every command and `<script> --help` explains one; the formats are in "
          "`make fleet-tickets SCHEMA=1` and `make contract-check SCHEMA=1`")


def waste(text: str, session: "Session") -> str | None:
    """Why this command wastes tokens (fix-plan A1-A3), or None."""
    for pattern, what in SWEEPS:
        if what.startswith(("git status", "a git diff")) and session.role == "release-clerk":
            continue  # it commits: what changed is its job
        if pattern.search(text):
            return (f"Refused ({what}): its output is paid on every later turn. `make fleet-task` (your brief) and "
                    "`make fleet-context MOD=<id>` say where things are; list one folder with `ls <dir>`.")
    for pattern, what in SEARCHES:
        if pattern.search(text):
            return f"Refused ({what})."
    if (m := WIDE_CONTEXT.search(text)) and int(next(g for g in m.groups() if g)) > 2:
        return "At most 2 lines of search context (-C 2): wider context fills your context for the whole run."
    if TOOLCHAIN_SHELL.search(text):
        return f"Do not read the toolchain to learn how it works: {HOW_TO}."
    for m in ONCE.finditer(text):
        cmd = " ".join(m.group(0).split())
        if cmd in session.runs:
            return (f"You already ran `{cmd}` in this session and its output has not changed: it is still in your "
                    "context above.")
    return None


def remember_runs(text: str, session: "Session") -> None:
    found = [" ".join(m.group(0).split()) for m in ONCE.finditer(text)]
    if found:
        session.runs += [c for c in found if c not in session.runs]
        session.save()


SPEC_FILE = re.compile(r"^planning/mods/[\w-]+\.md$")
SKILL_FILE = re.compile(r"^\.(?:agents|claude)/skills/[\w-]+/SKILL\.md$")
TOOLCHAIN_FILE = re.compile(r"^(?:scripts/|Makefile$|\.agents/(?:fleet-plan|reference)\.md$)")
WHOLE_READ_LINES = 60  # a spec or skill longer than this is read by section


def read_range(args: object) -> tuple[int | None, int | None]:
    if not isinstance(args, dict):
        return None, None
    start = args.get("StartLine") or args.get("offset")
    end = args.get("EndLine") or ((start or 0) + args["limit"] if args.get("limit") else None)
    return start, end


def tool_name(payload: dict) -> str:
    call = payload.get("toolCall") if isinstance(payload.get("toolCall"), dict) else {}
    return str(payload.get("tool_name") or call.get("name") or call.get("toolName") or "")


def read_waste(payload: dict, paths: list[str], session: "Session") -> str | None:
    """Why this read or search wastes tokens (fix-plan A2-A3), or None."""
    tool = tool_name(payload).lower()
    args = tool_input(payload)
    searching = any(t in tool for t in ("grep", "search"))
    globbing = any(t in tool for t in ("glob", "find_by_name"))
    for rel in paths:
        if TOOLCHAIN_FILE.match(rel):
            return f"Do not read {rel} to learn how a command works: {HOW_TO}."
        if searching and rel.startswith(DECOMPILED):
            return "Search the decompiled Java with `make java-find DIR=<decompiled> TEXT=<name>`, not the search tool."
        if searching and SPEC_FILE.match(rel):
            return "Find a spec section with `make spec-toc MOD=<id>`, then `make spec-show MOD=<id> SECTION=\"...\"`."
    if globbing and (not paths or paths == [""]) and "**" in json.dumps(args):
        return ("No glob over the whole checkout: your brief (`make fleet-task`) and `make fleet-context MOD=<id>` "
                "say where things are; glob inside one folder.")
    if searching or globbing:
        return None
    start, end = read_range(args)
    for rel in paths:
        full = ROOT / rel
        if not full.is_file():
            continue
        whole = not start and not end
        if whole and (SPEC_FILE.match(rel) or SKILL_FILE.match(rel)):
            n = full.read_text(encoding="utf-8", errors="replace").count("\n")
            if n > WHOLE_READ_LINES:
                if SPEC_FILE.match(rel):
                    mod = Path(rel).stem
                    return (f"{rel} has {n} lines: read the sections you need, `make spec-toc MOD={mod}` then "
                            f"`make spec-show MOD={mod} SECTION=\"<heading>\"`.")
                skill = Path(rel).parent.name
                return (f"{rel} has {n} lines: your brief has the sections your ticket needs; for another, "
                        f"`make skill-toc SKILL={skill}` then `make skill-show SKILL={skill} SECTION=\"<heading>\"`.")
        key = f"{rel}|{start or ''}|{end or ''}"
        mtime = full.stat().st_mtime
        if session.reads.get(key) == mtime:
            return (f"You already read {rel}{f' lines {start}-{end}' if start or end else ''} and it has not changed "
                    "since: it is still in your context above.")
    return None


def remember_reads(payload: dict, paths: list[str], session: "Session") -> None:
    tool = tool_name(payload).lower()
    if any(t in tool for t in ("grep", "search", "glob", "find_by_name", "list")):
        return
    start, end = read_range(tool_input(payload))
    changed = False
    for rel in paths:
        full = ROOT / rel
        if full.is_file():
            session.reads[f"{rel}|{start or ''}|{end or ''}"] = full.stat().st_mtime
            changed = True
    if changed:
        session.save()


WARNINGS: list[str] = []  # for the conductor: the same advice, not a refusal (it may need to look around)


def bd_text_removed(text: str) -> str:
    """The command with the quoted arguments of `bd` calls emptied: a comment, title or reason that
    quotes "--force" or "git push" is text for Beads, not a command. Quotes that run something
    ($(...) or backticks) stay."""
    out, i, words, word = [], 0, [], ""
    while i < len(text):
        c = text[i]
        if c in "\"'":
            end = i + 1
            while end < len(text) and text[end] != c:
                end += 2 if c == '"' and text[end] == "\\" else 1
            quoted = text[i:end + 1]
            runs = c == '"' and ("$(" in quoted or "`" in quoted)
            command = (words + [word])[0] if (words or word) else ""
            out.append(c + c if command == "bd" and not runs else quoted)
            word += "q"
            i = end + 1
            continue
        if c in ";&|\n":
            words, word = [], ""
        elif c.isspace():
            if word:
                words.append(word)
            word = ""
        else:
            word += c
        out.append(c)
        i += 1
    return "".join(out)


def check_shell(payload: dict) -> str | None:
    commands = strings_named(tool_input(payload), lambda k: k in ("command", "commandline", "cmd"))
    # A heredoc's body is text being written, not commands being run.
    text = HEREDOC.sub("", "\n".join(commands)) if commands else json.dumps(payload)
    if BLOCKED_SHELL.search(bd_text_removed(text) if commands else text):
        return ("Blocked by the Rekkit fleet guard: pushing, releasing, signing, submodule changes, "
                "history rewrites and recursive deletes are for the human. Ask them to run it.")
    session = Session(payload)
    if m := FLEET_START.search(text):
        if session.fleet != m.group(1):
            session.fleet, session.idle_stops, session.progress = m.group(1), 0, ""
        # A conductor is never a role subagent: drop a bead it took by looking at a brief.
        session.tasks, session.role = [], None
        session.save()
    elif (m := FLEET_TASK.search(text)) and not session.fleet and not flat_mode():
        # A conductor that runs `make fleet-task` is only reading the brief; a subagent takes the bead.
        if m.group(1) not in session.tasks:
            session.tasks.append(m.group(1))
        session.role = m.group(2) or session.role
        session.save()
    if budget := over_budget(payload, session, text):
        return budget
    # A session that conducts keeps the conductor's rights even if a subagent shares its id.
    if session.tasks and not session.fleet and BEADS_CMD.search(text):
        return ("Role subagents do not use Beads (not even to read or for --help): your bead and what it needs "
                f"are in your prompt. Put anything for Beads in your report (follow_ups, notes): {INBOX_HINT}")
    if session.tasks or session.fleet:
        plain = bd_text_removed(text) if commands else text
        if why := waste(plain, session):
            if session.tasks and not session.fleet:
                return why
            WARNINGS.append(why)
        remember_runs(plain, session)
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


def flat_mode() -> bool:
    """Flat mode: the session does every role itself, so a `make fleet-task` makes it no role subagent."""
    import flat
    return flat.is_flat()


def check_spawn(payload: dict) -> str | None:
    """A subagent is being started (denied in flat mode, scripts/fleet/flat.py): keep its prompt under each bead it names, for a restart after a crash."""
    import flat
    if flat.is_flat():
        return flat.HOW
    args = tool_input(payload)
    if not isinstance(args, dict):
        return None
    prompt = args.get("Prompt") or args.get("prompt") or ""
    role = args.get("TypeName") or args.get("subagent_type") or args.get("Role") or "subagent"
    session = Session(payload)
    found = sorted(set(BEAD_ID.findall(prompt)))
    busy = [b for b in running_tasks() if b not in found]
    if found and len(busy) >= MAX_SUBAGENTS:
        return (f"{len(busy)} subagents are already working ({', '.join(busy)}); at most {MAX_SUBAGENTS} at once. "
                "End your turn: the stop hook waits for a report. File it (`make fleet-inbox`), then start the next.")
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
    if budget := over_budget(payload, Session(payload), " ".join(repo_paths(payload))):
        return budget
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
    for rel in repo_paths(payload):
        journal(session, "write", rel)
    return None


BIG_READ_LINES = 400  # lines of one file a single read may take (a 1,700-line file is ~25k tokens of context)


def big_read(payload: dict, paths: list[str]) -> str | None:
    """A read that pulls hundreds of lines of one file into the context, where it stays for every later
    turn (and a local model runs out of GPU memory): refused for every agent, with how to find the part."""
    args = tool_input(payload)
    if not isinstance(args, dict):
        return None
    start, end = read_range(args)
    candidates = list(paths)
    for raw in strings_named(args, lambda k: k in ("path", "file_path", "filepath", "file", "absolutepath", "targetfile")):
        m = re.match(r"(.+?):(\d+)(?:-(\d+))?$", raw)  # omp's "file:1-1887"
        if m:
            rel = os.path.relpath(m.group(1), ROOT) if os.path.isabs(m.group(1)) else m.group(1)
            candidates.append(rel)
            start, end = int(m.group(2)), int(m.group(3)) if m.group(3) else None
    for rel in candidates:
        full = ROOT / rel
        if not full.is_file() or SPEC_FILE.match(rel) or SKILL_FILE.match(rel) or rel.startswith(DECOMPILED):
            continue
        try:
            n = full.read_text(encoding="utf-8", errors="replace").count("\n") + 1
        except OSError:
            continue
        first, last = int(start or 1), int(end or n)
        if min(last, n) - first + 1 > BIG_READ_LINES:
            hint = (f"`grep -n \"^pub \\|^    pub fn\\|^impl\\|^mod \" {rel}` for its map" if rel.endswith(".rs")
                    else f"`grep -n \"<name>\" {rel}`")
            return (f"{rel} has {n} lines: do not read {min(last, n) - first + 1} of them at once (they stay in your "
                    f"context for every later turn). Find what you need first ({hint}, or grep the name you are "
                    f"after), then read at most {BIG_READ_LINES} lines around it (offset/limit, or {rel}:<from>-<to>).")
    return None


def check_read(payload: dict) -> str | None:
    session = Session(payload)
    paths = repo_paths(payload)
    if budget := over_budget(payload, session, " ".join(paths)):
        return budget
    if why := whole_java_read(payload, paths):
        return why
    if why := big_read(payload, paths):
        return why
    if session.tasks or session.fleet:
        if why := read_waste(payload, paths, session):
            if session.tasks and not session.fleet:
                return why
            WARNINGS.append(why)
        remember_reads(payload, paths, session)
    return None


def whole_java_read(payload: dict, paths: list[str]) -> str | None:
    """Whole decompiled classes fill an agent's context for the rest of its run: read them in pieces."""
    args = tool_input(payload)
    if not isinstance(args, dict):
        return None
    start, end = read_range(args)
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


def contract_differences(mod: str) -> int:
    """How the mod differs from its contract: the static check, plus the last harness run's live screen,
    shape and scenario failures for it."""
    import contract_check
    count = len(contract_check.check(mod))
    try:
        summary = json.loads((MAIN / "target" / "e2e-report" / "summary.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return count
    for session in summary.get("phases", {}).get("isolated", []):
        if mod in session.get("mods", []):
            count += sum(str(e.get("stage", "")).startswith(beads.LIVE_CONTRACT_STAGES) for e in session.get("errors", []))
    return count


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
            elif st["epic"]["status"] != "blocked" and (differs := contract_differences(session.fleet)):
                # Closed in Beads is not done: the port must be the original, block by block.
                filed = beads.contract(session.fleet)
                problems.append(
                    f"{session.fleet} is closed in Beads but differs from the original in {differs} way(s) "
                    f"(make contract-check MOD={session.fleet}, and the last harness run's screen, shape and "
                    f"scenario checks). {filed} Work those beads; do not stop.")
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
    # A mod in progress: this is a new, cleared or compacted conductor session. Hand it the resume page
    # so it carries on at once, without a person.
    try:
        mod = beads.active_mod() if beads.available() else None
        if mod:
            import resume  # lazy: resume imports this module
            text += (f"\n\nA mod is in progress: {mod}. You are its conductor; carry on without waiting to be told. "
                     f"`make fleet-resume MOD={mod}` (below) shows what to file and what to restart.\n\n"
                     + resume.digest(mod))
    except Exception:  # noqa: BLE001 (best effort: Beads may be down)
        pass
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
        import flat
        if flat.is_flat():  # one linear session: its context is the cost, so keep it small and go on
            print(f"Flat mode: you work {args.target}{' as ' + args.role if args.role else ''} in this session, "
                  f"no subagent. Read your role file once. Append a line to {PROGRESS.relative_to(MAIN).as_posix()}/"
                  f"{args.target}.md per piece finished, write {beads.INBOX / (args.target + '.json')} as "
                  f"{REPORT_SHAPE} when done, run `make fleet-inbox`, then take the next ticket without "
                  "stopping. The report goes to the inbox, not to bd by hand.")
        else:
          print(f"You work {args.target}{' as ' + args.role if args.role else ''}. After each piece of work you finish "
              f"(a file, a block, a test passing), append one line saying what is done to {rel}: if you are "
              "interrupted, the next agent continues from it. When you finish (or cannot go on), write "
              f"{beads.INBOX / (args.target + '.json')} as {REPORT_SHAPE}. Do not write to Beads yourself. This is "
              "one ticket: report everything when it is done and stop; a fresh agent takes the next. "
              f"Your budget: {TASK_TURNS} turns or {TASK_BILLED:,} tokens billed (each call's fresh input, cached reads at a tenth), "
              "whichever comes first; past it the hooks let you only write your notes and a partial report. "
              "Each turn re-sends everything before it, so do the work in few turns: everything your ticket "
              "needs is below. Read nothing else unless a check fails and names the file; never list folders "
              "or read the scripts.")
        import brief  # lazy: brief imports beads and the Java outline
        text = brief.brief(args.target, args.role)
        if text:
            print("\n" + text)
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
    warning = "Token note from the fleet guard: " + " ".join(WARNINGS) if WARNINGS and not reason else ""
    if args.format == "agy":
        if reason:
            print(json.dumps({"decision": "deny", "reason": reason}))
        else:
            env = agy_env(payload) if args.check == "shell" else {}
            print(json.dumps({"decision": "allow", **({"overwrite": env} if env else {}),
                              **({"reason": warning} if warning else {})}))
        return 0
    if reason:
        print(reason, file=sys.stderr)
        return 2 if args.format == "claude" else 1
    if warning:
        if args.format == "claude":  # extra context for the model; the call goes ahead as usual
            print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "additionalContext": warning}}))
        else:
            print(warning, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
