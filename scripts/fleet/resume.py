#!/usr/bin/env python3
"""Where a mod's fleet run stands, rebuilt from what survives any crash: Beads, the hooks' session
states and journal (target/fleet/journal.jsonl), each subagent's saved prompt (target/fleet/prompts/)
and progress notes (target/fleet/progress/). Nothing depends on the agent saving anything, so it
holds when an agent dies, runs out of credits or loses the network mid-step.

A conductor taking over a mod reads this first (make fleet-start prints it): it shows which steps are
done, which subagents were interrupted and how to restart them without redoing their work, which
reports are waiting, and what the last conductor did.

Usage:
  make fleet-resume MOD=<id>
  python3 scripts/fleet/resume.py <id>
"""

from __future__ import annotations

import json
import sys
import time

import beads
import guard


def sessions() -> list[dict]:
    out = []
    for f in sorted(guard.STATE.glob("*.json")) if guard.STATE.is_dir() else []:
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        data["id"] = f.stem
        data["last_seen"] = data.get("last_seen") or f.stat().st_mtime
        out.append(data)
    return out


def journal() -> list[dict]:
    if not guard.JOURNAL.is_file():
        return []
    out = []
    for line in guard.JOURNAL.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            out.append(json.loads(line))
        except ValueError:
            continue
    return out


def clock(ts: float) -> str:
    return time.strftime("%a %H:%M", time.localtime(ts))


def digest(mod: str) -> str:
    st = beads.mod_status(mod)
    epic = st["epic"]
    prefix = epic["id"] + "."
    out = [f"RESUME {mod}: epic {epic['id']} {epic['status']}; next: {st['next']}"]
    steps = []
    for n, story in sorted(st["stories"].items()):
        done, total = st["counts"][n]
        mark = "done" if story["status"] == "closed" else (f"{done}/{total}" if total else story["status"])
        steps.append(f"{n} {beads.STEPS[n]} {mark}")
    out.append("Steps: " + "; ".join(steps))

    now, entries = time.time(), journal()
    running, interrupted = [], []
    for s in sessions():
        for bead in s.get("tasks") or []:
            if not bead.startswith(prefix) or guard.reported(bead):
                continue
            wrote = sorted({e["text"] for e in entries if e.get("s") == s["id"] and e.get("kind") == "write"})
            row = (bead, s.get("role") or "subagent", s["last_seen"], wrote)
            (running if now - s["last_seen"] < guard.LIVE_WINDOW else interrupted).append(row)
    if running:
        out.append("Running now (a tool call in the last 15 min): "
                   + ", ".join(f"{b} {r}" for b, r, _, _ in running) + ". Wait for their reports.")
    if interrupted:
        out.append("Interrupted (took a bead, no report, silent since): restart each on the same bead with "
                   "the same role; its first command `make fleet-task` shows it the earlier work.")
        for bead, role, seen, wrote in interrupted:
            prompt = guard.PROMPTS / f"{bead}.md"
            notes = guard.PROGRESS / f"{bead}.md"
            out.append(f"  {bead} {role}, last active {clock(seen)}; wrote {len(wrote)} file(s)"
                       + (f"; prompt: {prompt.relative_to(beads.MAIN).as_posix()}" if prompt.is_file() else "")
                       + (f"; notes: {notes.relative_to(beads.MAIN).as_posix()}" if notes.is_file() else ""))
    waiting = sorted(p.stem for p in beads.INBOX.glob("*.json")) if beads.INBOX.is_dir() else []
    if waiting:
        out.append(f"Unfiled reports: {', '.join(waiting)}. Run `make fleet-inbox` first.")

    mine = [e for e in entries if e.get("fleet") == mod and e.get("who") == "conductor"]
    if mine:
        out.append("The last conductor's final actions:")
        out += [f"  {e['t'][5:16].replace('T', ' ')} {e['kind']:5} {e['text'][:140]}" for e in mine[-8:]]
    if not (running or interrupted or waiting):
        out.append(f"Nothing in flight: continue with {st['next']}.")
    return "\n".join(out)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: make fleet-resume MOD=<id>", file=sys.stderr)
        return 1
    try:
        print(digest(sys.argv[1]))
    except beads.BeadsError as exc:
        print(f"[beads] {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
