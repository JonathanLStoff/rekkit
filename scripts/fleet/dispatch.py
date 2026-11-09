#!/usr/bin/env python3
"""Work a mod's ready tickets with headless agents, without an LLM in the loop.

The conductor's routine loop (fleet-ready, start one agent per ticket, wait, fleet-inbox, repeat) cost about
100k tokens per report as an agent and grew with every report (planning/token_audit/performance.md, C2).
This script does that loop itself and hands back only what needs judgement:

  - each ready ticket (`make fleet-ready`) gets a fresh headless agent of its role: `make fleet-task` first,
    its report in target/fleet/inbox/ last, on the model the tool is already on (no --model), with the
    role's thinking level (`effort:` in .agents/agents/<role>/agent.md, where the tool has one);
  - at most MAX at once (default 2), and the first ticket of a role that has none closed yet runs alone:
    the rest of that role waits until it is in, costs no more than a ticket may and reports done;
  - reports are filed as they come (`make fleet-inbox`), with each agent's tokens (target/fleet/usage.jsonl);
  - it stops starting agents when a report is blocked or failed, a ticket needs splitting (partial twice,
    labelled needs-split), a role's first ticket went wrong, the agents together billed BUDGET tokens, a
    model ran out of quota, or `make fleet-stop`; it waits for the agents in flight, then exits naming
    what needs the conductor (exit 2) or a person (exit 3). It exits 0 when no ticket is ready.
  - a quota or rate limit stops it (the person picked the model); FALLBACK=1 instead rests that model
    and goes on down .agents/models.json.
  - CONDUCTOR=1 (standalone, no conductor session watching): for what needs the conductor it starts one
    fresh headless conductor run on just that, then goes on.

On opencode it starts nothing (unless REKKIT_ALLOW_SUBAGENTS=1 on a cloud model), and never on a local
model (Ollama, LM Studio or llama.cpp): the one session does each role itself (AGENTS.md).

Usage:
  make fleet-dispatch MOD=<id> [AGENT=agy|claude|codex|opencode] [MAX=2] [BUDGET=<tokens>] [FALLBACK=1] [CONDUCTOR=1]
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

import agy
import beads
import models
from guard import STOP_FILE, TASK_BILLED, billed_tokens

ROOT = Path(__file__).resolve().parents[2]
ROLES = ROOT / ".agents" / "agents"
WORKERS = beads.STATE_DIR / "workers"
USAGE = beads.STATE_DIR / "usage.jsonl"
DEFAULT_BUDGET = 20_000_000  # tokens billed (all agents of this run together) before it asks a person
CLAUDE_ALLOWED = [
    "Read", "Grep", "Glob", "Edit", "Write",
    *(f"Bash({c}:*)" for c in ("cargo", "make", "python3", "python", "py", "scripts/fleet/python.sh", "rustfmt",
                               "ls", "mkdir", "git diff", "git log", "git show", "git add", "git commit",
                               "git rev-parse", "git ls-files"))]


@dataclass
class Worker:
    bead: str
    role: str
    title: str
    proc: subprocess.Popen
    log: Path
    started: float = field(default_factory=time.time)
    billed: int = 0
    turns: int = 0
    last_id: str | None = None
    tail: list[str] = field(default_factory=list)
    solo: bool = False


def role_meta(role: str) -> dict:
    path = ROLES / role / "agent.md"
    meta = {}
    if path.is_file():
        text = path.read_text(encoding="utf-8")
        if text.startswith("---\n"):
            for line in text[4:].split("\n---\n", 1)[0].splitlines():
                if ":" in line and not line.startswith(" "):
                    k, _, v = line.partition(":")
                    meta[k.strip()] = v.strip().strip('"')
    return meta


def pick_tool(wanted: str | None) -> str:
    order = [wanted] if wanted else models.config()["tools"]
    for tool in order:
        if tool and shutil.which(tool):
            return tool
    sys.exit(f"No agent CLI found ({', '.join(t for t in order if t)}); install one or give AGENT=<tool>.")


def local_opencode() -> bool:
    try:
        import local_capacity
        return bool(local_capacity.opencode_model()[1])
    except Exception:  # noqa: BLE001 (cannot tell: treat it as local, the safe side)
        return True


def prompt_for(mod: str, bead: str, role: str, title: str, tool: str) -> str:
    text = (f"You are the {role} subagent for the mod {mod}. Your bead: {bead} ({title}). Your first command is "
            f"`make fleet-task BEAD={bead} ROLE={role}`: it prints your whole brief. Do the ticket, then write your "
            f"report to target/fleet/inbox/{bead}.json and stop. Never run bd, git push or anything a person does.")
    if tool in ("codex",):  # no roles of its own: the role file is its instructions
        text += f" Your instructions are .agents/agents/{role}/agent.md (read it once, first)."
    return text


def command(tool: str, model: str | None, role: str, prompt: str, log: Path) -> list[str]:
    effort = role_meta(role).get("effort")
    mod_flag = ["--model", model] if model else []
    if tool == "agy":
        return [*agy.command(["-p", prompt]), "--agent", role, "--output-format", "stream-json", *mod_flag,
                *(["--effort", effort] if effort else []), "--log-file", str(log.with_suffix(".agy.log"))]
    if tool == "claude":
        return ["claude", "-p", prompt, "--agent", role, "--output-format", "stream-json", "--verbose", *mod_flag,
                *(["--effort", effort] if effort else []), "--permission-mode", "acceptEdits",
                "--allowedTools", *CLAUDE_ALLOWED]
    if tool == "codex":
        return ["codex", "exec", "--full-auto", *(["-m", model] if model else []), prompt]
    if tool == "opencode":
        return ["opencode", "run", "--agent", role, *(["-m", model] if model else []), prompt]
    raise ValueError(tool)


def read_usage(w: Worker, line: str) -> None:
    """Add one stream line's model call to the worker's billed tokens and turns (agy and Claude Code)."""
    try:
        ev = json.loads(line)
    except ValueError:
        return
    if not isinstance(ev, dict):
        return
    su = ev.get("step_update") or {}
    u = su.get("usage") if isinstance(su, dict) else None
    if isinstance(u, dict) and su.get("state") == "DONE":  # agy
        w.billed += billed_tokens(int(u.get("input_tokens") or 0), int(u.get("cache_read_tokens") or 0))
        w.turns += 1
        return
    msg = ev.get("message") if ev.get("type") == "assistant" else None
    if isinstance(msg, dict) and isinstance(msg.get("usage"), dict) and msg.get("id") != w.last_id:  # Claude Code
        u = msg["usage"]
        w.last_id = msg.get("id")
        w.billed += billed_tokens(int(u.get("input_tokens") or 0) + int(u.get("cache_creation_input_tokens") or 0),
                                  int(u.get("cache_read_input_tokens") or 0))
        w.turns += 1


def pump(w: Worker) -> bool:
    """Read what the worker printed so far (a reader thread feeds a queue: pipes do not select on
    Windows); True while it runs or has output left."""
    q, reader = w._q, w._t  # type: ignore[attr-defined]
    with w.log.open("a", encoding="utf-8") as f:
        while not q.empty():
            line = q.get_nowait()
            f.write(line)
            read_usage(w, line)
            w.tail = (w.tail + [line])[-models.TAIL:]
    return w.proc.poll() is None or reader.is_alive() or not q.empty()


def start(mod: str, tool: str, model: str | None, issue: dict, solo: bool) -> Worker:
    import queue
    import threading
    bead, role, title = issue["id"], beads.label_value(issue, "role") or "domain-builder", issue.get("title", "")
    WORKERS.mkdir(parents=True, exist_ok=True)
    log = WORKERS / f"{bead}.jsonl"
    log.unlink(missing_ok=True)
    cmd = command(tool, model, role, prompt_for(mod, bead, role, title, tool), log)
    proc = subprocess.Popen(cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                            encoding="utf-8", errors="replace")
    w = Worker(bead, role, title, proc, log, solo=solo)
    q: queue.Queue = queue.Queue()

    def reader() -> None:
        for line in proc.stdout:
            q.put(line)
    t = threading.Thread(target=reader, daemon=True)
    t.start()
    w._q, w._t = q, t  # type: ignore[attr-defined]
    print(f"[dispatch] {bead} -> {role}{' (alone: the first of its role)' if solo else ''}: {title}", flush=True)
    return w


def report_status(bead: str) -> str | None:
    """The status of the bead's last report, waiting or filed."""
    waiting = beads.INBOX / f"{bead}.json"
    files = [waiting] if waiting.is_file() else sorted((beads.INBOX / "done").glob(f"{bead}-*.json"))
    for f in reversed(files):
        try:
            return json.loads(f.read_text(encoding="utf-8")).get("status") or "done"
        except (OSError, ValueError):
            continue
    return None


def record(mod: str, tool: str, w: Worker, status: str | None) -> None:
    USAGE.parent.mkdir(parents=True, exist_ok=True)
    row = {"t": time.strftime("%Y-%m-%dT%H:%M:%S"), "mod": mod, "bead": w.bead, "role": w.role, "tool": tool,
           "billed": w.billed, "turns": w.turns, "status": status, "seconds": round(time.time() - w.started)}
    with USAGE.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row) + "\n")


def proven_roles(mod: str) -> set[str]:
    """Roles with a closed ticket in this mod: their first ticket went well."""
    try:
        done = beads.issues("--label", "ticket", "--label", f"mod:{mod}", "--status", "closed")
    except beads.BeadsError:
        return set()
    return {r for i in done if (r := beads.label_value(i, "role"))}


def conductor_run(mod: str, tool: str, model: str | None, needs: list[str]) -> None:
    """One fresh headless conductor run on just what the dispatcher could not decide."""
    import run_next
    prompt = (f"Follow AGENTS.md. Act as the conductor (.agents/agents/conductor/agent.md) for '{mod}'. "
              f"`make fleet-dispatch` stopped for these, and only these need you now:\n- " + "\n- ".join(needs)
              + "\nHandle each (a needs-split ticket: a spec-writer in split mode splits it, then close it; blocked "
                "or failed: fix the cause, defer it with a comment, or block the epic for a person). File reports "
                "with `make fleet-inbox`. Do not start ticket subagents yourself and do not run fleet-dispatch: "
                "stop when these are handled. Never push.")
    cand = models.Candidate(tool, model)
    log = beads.STATE_DIR / "agent-tool.log"
    cmd = run_next.agent_command(cand, [cand], prompt, log)
    print(f"[dispatch] conductor run for {len(needs)} item(s)", flush=True)
    with (ROOT / "target" / f"fleet-{mod}.log").open("a", encoding="utf-8") as f:
        run_next.run_agent(cmd, f)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("mod")
    parser.add_argument("--agent", default=os.environ.get("AGENT") or None)
    parser.add_argument("--max", type=int, default=int(os.environ.get("MAX") or 2))
    parser.add_argument("--budget", type=int, default=int(os.environ.get("BUDGET") or DEFAULT_BUDGET))
    parser.add_argument("--fallback", action="store_true", default=os.environ.get("FALLBACK") == "1")
    parser.add_argument("--conductor", action="store_true", default=os.environ.get("CONDUCTOR") == "1")
    args = parser.parse_args()

    import flat
    if flat.is_flat():
        print(flat.HOW)
        return 3
    tool = pick_tool(args.agent)
    if tool == "opencode" and (os.environ.get("REKKIT_ALLOW_SUBAGENTS") != "1" or local_opencode()):
        print("opencode runs no subagents (on a local model never; on a cloud model only with "
              "REKKIT_ALLOW_SUBAGENTS=1). Do each ticket yourself (`make fleet-ready`, then "
              "`make fleet-task BEAD=<id> ROLE=<role>`), as AGENTS.md says.")
        return 3
    chain = [c for c in models.candidates(tool) if c.tool == tool] if args.fallback else [models.Candidate(tool, None)]
    current = chain[0]
    STOP_FILE.unlink(missing_ok=True)
    proven = proven_roles(args.mod)
    running: dict[str, Worker] = {}
    started: set[str] = set()
    needs: list[str] = []  # for the conductor
    person: list[str] = []  # for a person
    billed_total = 0
    print(f"[dispatch] {args.mod} on {current.key}, at most {args.max} at once, budget {args.budget:,} tokens", flush=True)

    while True:
        for bead, w in list(running.items()):
            if pump(w):
                continue
            del running[bead]
            status = report_status(bead)
            record(args.mod, tool, w, status)
            billed_total += w.billed
            print(f"[dispatch] {bead} ({w.role}) ended: {status or 'no report'}, {w.turns} turns, "
                  f"{w.billed:,} tokens billed (run total {billed_total:,})", flush=True)
            hit = models.limit_in("".join(w.tail))
            if hit:
                if args.fallback:
                    models.rest(current, *hit)
                    nxt = models.first_ready(chain)
                    if nxt:
                        print(f"[dispatch] {current.key} hit a limit; going on with {nxt.key}", flush=True)
                        current = nxt
                        started.discard(bead)  # its ticket is still open: it is started again
                        continue
                until = models.rest(current, *hit)
                person.append(f"{current.key} ran out ({hit[0][:120]}); it resets about {models.when(until)}. "
                              "Run fleet-dispatch again then (or FALLBACK=1 to go on with another model).")
                continue
            if w.solo:
                if status == "done" and w.billed <= TASK_BILLED * 1.2:
                    proven.add(w.role)
                else:
                    needs.append(f"{bead}: the first {w.role} ticket of this run ended {status or 'with no report'} "
                                 f"after {w.billed:,} tokens: check its report and the role's brief before more")
            if status in ("blocked", "failed") or status is None:
                needs.append(f"{bead} ({w.role}): {status or 'no report written'}")
            elif status == "partial":
                started.discard(bead)  # a fresh agent goes on from its notes (fleet-inbox labels it needs-split
                # instead when two could not finish it, and then it is not ready)
        if beads.INBOX.is_dir() and any(beads.INBOX.glob("*.json")):
            filed = beads.inbox()
            print("[dispatch] " + filed.replace("\n", "\n[dispatch] "), flush=True)
            needs += [ln.split("Note: ", 1)[1] for ln in filed.splitlines() if "Note: " in ln and beads.SPLIT in ln]
        stopping = bool(needs or person) or STOP_FILE.is_file() or billed_total >= args.budget
        if billed_total >= args.budget and not any("budget" in p for p in person):
            person.append(f"the agents of this run billed {billed_total:,} tokens, past BUDGET={args.budget:,}")
        if stopping:
            if running:
                time.sleep(5)
                continue
            if needs and args.conductor and not person and not STOP_FILE.is_file():
                conductor_run(args.mod, tool, current.model, needs)
                needs = []
                proven = proven_roles(args.mod) | proven
                continue
            break
        try:
            ready = [i for i in beads.ready(args.mod) if i["id"] not in started and i["id"] not in running]
        except beads.BeadsError as exc:
            person.append(f"Beads: {exc}")
            break
        if not ready and not running:
            print(f"[dispatch] No ticket of {args.mod} is ready; the conductor's next step.", flush=True)
            return 0
        solo_running = any(w.solo for w in running.values())
        for issue in ready:
            if solo_running or len(running) >= args.max:
                break
            role = beads.label_value(issue, "role") or "domain-builder"
            if role not in proven:
                if running:
                    continue  # its role's first ticket runs alone: wait for the others to finish
                running[issue["id"]] = start(args.mod, tool, current.model, issue, solo=True)
                started.add(issue["id"])
                break
            running[issue["id"]] = start(args.mod, tool, current.model, issue, solo=False)
            started.add(issue["id"])
        time.sleep(5)

    if person:
        print("[dispatch] Stopped; a person is needed:\n  - " + "\n  - ".join(person), flush=True)
        return 3
    if needs:
        print("[dispatch] Stopped; the conductor is needed for:\n  - " + "\n  - ".join(needs)
              + "\nHandle these, then run `make fleet-dispatch MOD=" + args.mod + "` again.", flush=True)
        return 2
    print("[dispatch] Stopped by `make fleet-stop`." if STOP_FILE.is_file() else "[dispatch] Done.", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
