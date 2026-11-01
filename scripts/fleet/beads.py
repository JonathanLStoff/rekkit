#!/usr/bin/env python3
"""The fleet's progress in Beads (bd), the source of truth for the port.

  epic     one per mod (labels mod, mod:<id>, wave:<w>, tier:<t>), blocked by the epics it needs
  story    one per step of the pipeline (label step:<n>), created when the mod starts
  feature  one per block, item, subsystem or missing asset, under the story it belongs to
  bug      a review finding or harness failure, under the story it blocks

Only the conductor writes (directly with bd, or through these commands); role subagents report to
target/fleet/inbox/<bead>.json and the conductor files the report with `inbox`. Hooks only read.

Commands (make targets in brackets):
  status [<mod>]            a mod's epic, stories and counts; every mod without one
  start <mod>               create the mod's stories, set its epic in progress        [fleet-start]
  features <mod>            features from planning/mods/<mod>.features.json             [fleet-features]
  gaps <mod>                New-assets features from the completeness check            [fleet-gaps]
  inbox                     file every subagent report in Beads                         [fleet-inbox]
  board                     planning/fleet-board.md and .beads/issues.jsonl from Beads  [board]
  seed                      one-time: an epic per row of the old markdown board
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path


def main_checkout() -> Path:
    """The main checkout, also from a worktree, so every session shares one database and inbox."""
    here = Path(__file__).resolve().parents[2]
    r = subprocess.run(["git", "-C", str(here), "rev-parse", "--path-format=absolute", "--git-common-dir"],
                       capture_output=True, text=True)
    return Path(r.stdout.strip()).parent if r.returncode == 0 and r.stdout.strip() else here


MAIN = main_checkout()
BEADS_DIR = MAIN / ".beads"
INBOX = MAIN / "target" / "fleet" / "inbox"
BOARD = MAIN / "planning" / "fleet-board.md"
STEPS = {1: "Spec", 2: "Scaffold", 3: "Code", 4: "Check", 5: "Extract assets", 6: "New assets",
         7: "Recheck", 8: "Harness", 9: "Package", 10: "Human", 11: "Own art"}
# Each story waits for these (Extract assets runs beside the code; Own art is the later pass).
STEP_DEPS = {2: [1], 3: [2], 4: [3], 5: [1], 6: [3, 5], 7: [4, 6], 8: [7], 9: [8], 10: [9], 11: [10]}
# The agent's part ends with Package; Human is the person's, Own art comes later.
AGENT_STEPS = range(1, 10)
MARK = {"closed": "✓", "in_progress": "▶", "blocked": "✗", "deferred": "—", "open": "·"}


class BeadsError(Exception):
    pass


def bd(*args: str, parse: bool = True):
    env = dict(os.environ, BEADS_DIR=str(BEADS_DIR), BD_NON_INTERACTIVE="1")
    r = subprocess.run(["bd", *args, *(["--json"] if parse else [])], cwd=MAIN, env=env,
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r.returncode:
        raise BeadsError(f"bd {' '.join(args)}: {(r.stderr or r.stdout).strip()[-500:]}")
    if not parse:
        return r.stdout
    out = r.stdout.strip()
    return json.loads(out) if out else []


def available() -> bool:
    return bool(shutil.which("bd")) and (BEADS_DIR / "metadata.json").is_file() and (
        (BEADS_DIR / "embeddeddolt").is_dir() or (BEADS_DIR / "dolt").is_dir())


def issues(*filters: str) -> list[dict]:
    return bd("list", "--all", "--limit", "0", "--flat", *filters)


def label_value(issue: dict, prefix: str) -> str | None:
    return next((l.split(":", 1)[1] for l in issue.get("labels", []) if l.startswith(prefix + ":")), None)


def epics() -> dict[str, dict]:
    return {label_value(i, "mod"): i for i in issues("--type", "epic", "--label", "mod") if label_value(i, "mod")}


def epic(mod: str) -> dict:
    e = epics().get(mod)
    if not e:
        raise BeadsError(f"no epic for mod {mod!r} (bd list --type epic --label mod:{mod})")
    return e


def create(title: str, kind: str, *, parent: str | None = None, labels: list[str] = (), deps: list[str] = (),
           priority: int | None = None, description: str = "") -> str:
    args = ["create", title, "-t", kind]
    if parent:
        args += ["--parent", parent]
    if labels:
        args += ["-l", ",".join(labels)]
    if deps:
        args += ["--deps", ",".join(deps)]
    if priority is not None:
        args += ["-p", str(priority)]
    if description:
        args += ["-d", description]
    made = bd(*args)
    return (made[0] if isinstance(made, list) else made)["id"]


# ---------------------------------------------------------------- status

def mod_status(mod: str) -> dict:
    """The mod's epic and stories, as the gate, runner and board need them."""
    e = epic(mod)
    family = issues("--label", f"mod:{mod}")
    stories = {}
    for i in family:
        step = label_value(i, "step")
        if i["issue_type"] == "story" and step and step.isdigit():
            stories[int(step)] = i
    counts = {n: [0, 0] for n in stories}
    by_id = {s["id"]: n for n, s in stories.items()}
    for i in family:
        n = by_id.get(i.get("parent", ""))
        if n is not None and i["issue_type"] in ("feature", "bug", "task"):
            counts[n][1] += 1
            counts[n][0] += i["status"] == "closed"
    pending = [n for n in AGENT_STEPS if stories.get(n, {}).get("status") != "closed"]
    finished = e["status"] in ("closed", "blocked", "deferred") or (bool(stories) and not pending)
    return {"mod": mod, "epic": e, "stories": stories, "counts": counts, "finished": finished,
            "next": f"step {pending[0]} {STEPS[pending[0]]}" if pending else "the human's review"}


def active_mod() -> str | None:
    return next((m for m, e in epics().items() if e["status"] == "in_progress"), None)


def next_mod() -> str | None:
    """The first ready epic: open, every epic it needs closed (bd ready order: priority, then age)."""
    ready = bd("ready", "--limit", "0")
    return next((label_value(i, "mod") for i in ready if i["issue_type"] == "epic" and label_value(i, "mod")), None)


# ---------------------------------------------------------------- conductor commands

def start(mod: str) -> dict:
    """Create the mod's stories if missing and set its epic in progress."""
    e = epic(mod)
    have = mod_status(mod)["stories"]
    ids = {n: s["id"] for n, s in have.items()}
    for n, name in STEPS.items():
        if n in ids:
            continue
        ids[n] = create(f"{mod}: {n} {name}", "story", parent=e["id"], labels=[f"step:{n}"],
                        deps=[ids[d] for d in STEP_DEPS.get(n, []) if d in ids])
        if n == 11:
            bd("defer", ids[n])
    if e["status"] == "open":
        bd("update", e["id"], "--status", "in_progress")
    return mod_status(mod)


def add_children(mod: str, step: int, entries: list[tuple[str, str, list[str]]]) -> int:
    """Features (title, type, labels) under a story, skipping titles it already has."""
    story = mod_status(mod)["stories"].get(step)
    if not story:
        raise BeadsError(f"{mod} has no step {step} story: run make fleet-start MOD={mod}")
    existing = {i["title"] for i in issues("--parent", story["id"])}
    made = 0
    for title, kind, labels in entries:
        if title not in existing:
            create(title, kind, parent=story["id"], labels=labels)
            made += 1
    return made


def features(mod: str) -> str:
    """Code, Extract assets and Harness features from the spec's feature list."""
    path = MAIN / "planning" / "mods" / f"{mod}.features.json"
    if not path.is_file():
        raise BeadsError(f"{path.relative_to(MAIN)} is missing: the spec-writer writes it at the end of step 1")
    spec = json.loads(path.read_text(encoding="utf-8"))
    blocks = [b["id"] for b in spec.get("blocks", [])]
    items = [i["id"] for i in spec.get("items", [])]
    subsystems = [f["id"] for f in spec.get("features", [])]
    code = ([(f"{mod}: block {b}", "feature", ["block"]) for b in blocks]
            + [(f"{mod}: item {i}", "feature", ["item"]) for i in items]
            + [(f"{mod}: feature {f}", "feature", ["subsystem"]) for f in subsystems])
    made = add_children(mod, 3, code)
    made += add_children(mod, 5, [(f"{mod}: assets for {k} {n}", "feature", ["asset"])
                                  for k, names in (("block", blocks), ("item", items)) for n in names])
    made += add_children(mod, 8, [(f"{mod}: harness {b}", "feature", ["harness"]) for b in blocks])
    return f"{made} features added for {mod} ({len(blocks)} blocks, {len(items)} items, {len(subsystems)} subsystems)."


def gaps(mod: str) -> str:
    """New-assets features from scripts/pack_mod.py's completeness check."""
    r = subprocess.run([sys.executable, str(MAIN / "scripts" / "pack_mod.py"), "--gaps", "--mod", mod],
                       cwd=MAIN, capture_output=True, text=True)
    if r.returncode not in (0, 1):
        raise BeadsError(r.stderr.strip()[-800:])
    found = json.loads(r.stdout or "[]")
    made = add_children(mod, 6, [(f"{mod}: new asset {g['path']}", "feature", ["asset", "new-asset"]) for g in found])
    return f"{len(found)} missing asset(s), {made} new feature(s) under {mod} step 6."


def inbox() -> str:
    """File every subagent report: a comment, the bead's new status, and follow-ups as new beads."""
    done_dir = INBOX / "done"
    done_dir.mkdir(parents=True, exist_ok=True)
    filed = []
    for path in sorted(INBOX.glob("*.json")):
        try:
            report = json.loads(path.read_text(encoding="utf-8"))
        except ValueError as exc:
            raise BeadsError(f"{path.name} is not JSON ({exc}); ask its role to rewrite it") from exc
        bead = report.get("bead") or path.stem
        show = bd("show", bead)
        issue = show[0] if isinstance(show, list) else show
        status = report.get("status", "done")
        lines = [f"[{report.get('role', 'subagent')}] {status}: {report.get('summary', '')}".strip()]
        if report.get("files"):
            lines.append("Files: " + ", ".join(report["files"]))
        if report.get("notes"):
            lines.append(report["notes"])
        bd("comment", bead, "\n".join(lines), parse=False)
        parent = issue.get("parent") if issue["issue_type"] not in ("story", "epic") else bead
        for f in report.get("follow_ups", []):
            kind = f.get("type", "bug") if f.get("type") in ("bug", "feature", "task") else "bug"
            create(f.get("title", "follow-up"), kind, parent=parent, description=f.get("why", ""))
        if status == "done":
            bd("close", bead, "--reason", (report.get("summary") or "done")[:200])
        elif status == "blocked":
            bd("update", bead, "--status", "blocked")
        else:
            bd("update", bead, "--status", "open")
        path.rename(done_dir / f"{path.stem}-{time.strftime('%Y%m%dT%H%M%S')}.json")
        filed.append(f"{bead} ({status}, {len(report.get('follow_ups', []))} follow-up(s))")
    return "Filed: " + ", ".join(filed) if filed else "Inbox empty."


# ---------------------------------------------------------------- board

def board() -> str:
    """planning/fleet-board.md and .beads/issues.jsonl, both generated from Beads."""
    all_issues = issues()
    by_mod: dict[str, list[dict]] = {}
    for i in all_issues:
        mod = label_value(i, "mod")
        if mod:
            by_mod.setdefault(mod, []).append(i)
    rows = []
    order = sorted((i for i in all_issues if i["issue_type"] == "epic" and label_value(i, "mod")),
                   key=lambda e: (e.get("priority", 2), e.get("created_at", "")))
    for n, e in enumerate(order, start=1):
        mod = label_value(e, "mod")
        family = by_mod.get(mod, [])
        stories = {int(label_value(i, "step")): i for i in family
                   if i["issue_type"] == "story" and (label_value(i, "step") or "").isdigit()}
        cells = []
        for step in STEPS:
            s = stories.get(step)
            if not s:
                cells.append("✓" if e["status"] == "closed" else "·")
                continue
            kids = [i for i in family if i.get("parent") == s["id"]]
            done = sum(i["status"] == "closed" for i in kids)
            cells.append(MARK.get(s["status"], "·") + (f" {done}/{len(kids)}" if kids else ""))
        deps = [l.split(":", 1)[1] for l in e.get("labels", []) if l.startswith("needs:")]
        state = {"in_progress": "active", "closed": "done", "open": "queued"}.get(e["status"], e["status"])
        desc = (e.get("description") or "").strip().splitlines()
        rows.append([str(n), mod, f"`{label_value(e, 'path') or 'mods/' + mod}`", label_value(e, "wave") or "",
                     label_value(e, "tier") or "", ",".join(deps) or "—", state, *cells,
                     desc[0] if desc else "", e["id"]])
    head = ["#", "Mod", "Path", "Wave", "Tier", "Deps", "State", *(f"{n} {s}" for n, s in STEPS.items()), "Notes", "Bead"]
    lines = [
        "# Fleet board",
        "",
        "Generated from Beads by `make board`: do not edit. Change progress with `bd` (the conductor) and regenerate.",
        "One mod at a time, in this order (`bd ready` gives the next). Steps are the mod's stories; `▶ 12/42` is a",
        "story in progress with 12 of its 42 features or bugs closed.",
        "",
        "State: `queued` · `active` · `blocked` · `done`. Steps: `·` todo · `▶` in progress · `✓` done · `✗` blocked · `—` deferred.",
        "",
        "| " + " | ".join(head) + " |",
        "|" + "|".join("---:" if h == "#" else "---" for h in head) + "|",
        *("| " + " | ".join(r) + " |" for r in rows),
        "",
    ]
    BOARD.write_text("\n".join(lines), encoding="utf-8")
    bd("export", "-o", str(BEADS_DIR / "issues.jsonl"), parse=False)
    return f"Wrote {BOARD.relative_to(MAIN)} ({len(rows)} mods) and .beads/issues.jsonl."


def seed(board_md: Path) -> str:
    """An epic per row of the old markdown board, in its order, with its dependencies."""
    have = epics()
    old_rows = []
    for line in board_md.read_text(encoding="utf-8").splitlines():
        cells = [c.strip() for c in line.split("|")]
        if len(cells) > 8 and cells[1].isdigit():
            old_rows.append(cells)
    ids = {m: e["id"] for m, e in have.items()}
    priority = {"core": 0, "1": 1, "2": 2, "3": 3, "4": 4}
    for cells in old_rows:
        mod, path, wave, tier, deps, state = cells[2], cells[3].strip("`"), cells[4], cells[5], cells[6], cells[7]
        if mod in ids:
            continue
        needs = [d for d in deps.replace(" ", "").split(",") if d in ids]
        ids[mod] = create(mod, "epic", labels=["mod", f"mod:{mod}", f"wave:{wave}", f"tier:{tier}", f"path:{path}",
                                               *(f"needs:{d}" for d in needs)],
                          deps=[ids[d] for d in needs], priority=priority.get(wave, 4), description=cells[-2] if len(cells) > 18 else "")
        if state == "done":
            bd("close", ids[mod], "--reason", "done before the move to Beads")
    return f"{len(ids) - len(have)} epic(s) created, {len(have)} already there."


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["status", "start", "features", "gaps", "inbox", "board", "seed"])
    parser.add_argument("mod", nargs="?")
    args = parser.parse_args()
    if not available():
        print("Beads is not set up: run ./setup.sh (it installs bd and runs bd init).", file=sys.stderr)
        return 1
    try:
        if args.command in ("start", "features", "gaps") and not args.mod:
            raise BeadsError(f"{args.command} needs a mod id (MOD=<id>)")
        if args.command == "status":
            if not args.mod:
                for mod, e in epics().items():
                    print(f"{e['id']:12} {e['status']:12} {mod}")
                return 0
            st = mod_status(args.mod)
            print(f"{st['epic']['id']} {args.mod}: {st['epic']['status']}, next {st['next']}"
                  + (" (finished)" if st["finished"] else ""))
            for n, s in sorted(st["stories"].items()):
                done, total = st["counts"].get(n, [0, 0])
                print(f"  {s['id']:16} {n:>2} {STEPS[n]:15} {s['status']:12}" + (f" {done}/{total}" if total else ""))
        elif args.command == "start":
            st = start(args.mod)
            print(f"{args.mod}: {len(st['stories'])} stories, next {st['next']}.")
        elif args.command == "features":
            print(features(args.mod))
        elif args.command == "gaps":
            print(gaps(args.mod))
        elif args.command == "inbox":
            print(inbox())
        elif args.command == "board":
            print(board())
        elif args.command == "seed":
            print(seed(Path(args.mod) if args.mod else BOARD))
    except BeadsError as exc:
        print(f"[beads] {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
