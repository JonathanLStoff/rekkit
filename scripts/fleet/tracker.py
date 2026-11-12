#!/usr/bin/env python3
"""The fleet's progress in the issue tracker (issuectl), the source of truth for the port.

Issues are markdown files under issues/ (one folder per issue, item.md with frontmatter), committed with
the code. Every write goes through the `issuectl` CLI, which holds the repo lock and validates the schema;
reads use `issuectl list --all --json`, cached until the next write.

  epic     one per mod (labels mod, mod:<id>, wave:<w>, tier:<t>, needs:<epic>), blocked by the epics it
           needs, except rekkit_core: the shared crate grows with the mods. Its epic holds core work as
           tasks labelled for:<mod>; it opens while any is pending and closes when none is (core_sync), and
           a mod's step 3 Code is not done until its for:<mod> core work is closed.
  story    one per step of the pipeline: a task labelled story and step:<n>, under the mod's epic
  feature  one per block, item, subsystem or missing asset, under the story it belongs to
  bug      a review finding or harness failure, under the story it blocks

Only the conductor writes (directly with issuectl, or through these commands); role subagents report to
target/fleet/inbox/<bead>.json and the conductor files the report with `inbox`. Hooks only read.

Commands (make targets in brackets):
  status [<mod>]            a mod's epic, stories and counts; every mod without one
  start <mod>               create the mod's stories, set its epic in progress        [fleet-start]
  features <mod>            features from planning/mods/<mod>.features.json             [fleet-features]
  gaps <mod>                New-assets features from the completeness check            [fleet-gaps]
  inbox                     file every subagent report in the tracker                   [fleet-inbox]
  board                     planning/fleet-board.md from the issues                     [board]
  core-need <mod> --title T [--why W]  core work the mod needs, under rekkit_core     [core-need]
  core-sync                 open rekkit_core while core work is pending, close it when none is
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
import tempfile
import time
from collections import Counter
from functools import lru_cache
from pathlib import Path
from typing import Any


def main_checkout() -> Path:
    """The main checkout, also from a worktree, so every session shares one tracker and inbox."""
    here = Path(__file__).resolve().parents[2]
    r = subprocess.run(["git", "-C", str(here), "rev-parse", "--path-format=absolute", "--git-common-dir"],
                       capture_output=True, text=True)
    common = r.stdout.strip()
    if r.returncode == 0 and common:
        if sys.platform.startswith("win"):
            common = re.sub(r"^/([A-Za-z])/", r"\1:/", common)  # Git Bash-style /c/... paths, from git on PowerShell
        return Path(common).parent
    return here


MAIN = main_checkout()
# This checkout: the planning files a spec-writer writes (outline, tickets) are read where it wrote them,
# also in a worktree (the tracker, the inbox and the board stay the main checkout's).
WORK = Path(__file__).resolve().parents[2]
ISSUES_DIR = MAIN / "issues"
STATE_DIR = MAIN / "target" / "fleet"  # the fleet's local state: inbox, journal, prompts, progress notes
INBOX = STATE_DIR / "inbox"
BOARD = MAIN / "planning" / "fleet-board.md"
STEPS = {1: "Spec", 2: "Scaffold", 3: "Code", 4: "Check", 5: "Extract assets", 6: "New assets",
         7: "Recheck", 8: "Harness", 9: "Package", 10: "Human", 11: "Own art"}
# Each story waits for these (Extract assets runs beside the code; Own art is the later pass).
STEP_DEPS = {2: [1], 3: [2], 4: [3], 5: [1], 6: [3, 5], 7: [4, 6], 8: [7], 9: [8], 10: [9], 11: [10]}
# The agent's part ends with Package; Human is the person's, Own art comes later.
AGENT_STEPS = range(1, 10)
MARK = {"closed": "✓", "in_progress": "▶", "blocked": "✗", "deferred": "—", "open": "·"}
CORE = "rekkit_core"
SETTLED = ("closed", "deferred")


class TrackerError(Exception):
    pass


# ---------------------------------------------------------------- the issuectl CLI

def _exe() -> str:
    return shutil.which("issuectl") or "issuectl"


def _run(*args: str) -> str:
    """Run issuectl at the main checkout. Warnings (stderr on success) are not errors."""
    r = subprocess.run([_exe(), "--root", str(MAIN), *args], cwd=MAIN, capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    if r.returncode:
        raise TrackerError(f"issuectl {' '.join(args[:2])}: {(r.stderr or r.stdout).strip()[-500:]}")
    return r.stdout


def _json(*args: str) -> Any:
    out = _run(*args, "--json").strip()
    return json.loads(out)["data"] if out else None


def _written() -> None:
    global _cache
    _cache = None


def _body_file(text: str) -> Path:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix="body-", suffix=".md", dir=STATE_DIR)
    with os.fdopen(fd, "w", encoding="utf-8") as f:
        f.write(text)
    return Path(name)


# issuectl statuses -> the statuses this module works with (open, in_progress, blocked, deferred, closed)
CLOSING = {"done", "fixed", "wontfix", "duplicate", "cannot-reproduce", "obsolete"}
STATUS_IN = {"open": "open", "untriaged": "open", "in-progress": "in_progress", "testing": "in_progress",
             "deferred": "deferred", "needs-info": "blocked"}
STATUS_OUT = {"open": "open", "in_progress": "in-progress", "blocked": "needs-info", "deferred": "deferred"}
PRIORITY_IN = {"high": 1, "normal": 2, "low": 3}
PRIORITY_OUT = {0: "high", 1: "high", 2: "normal", 3: "low", 4: "low"}
_cache: list[dict] | None = None


def _norm(i: dict) -> dict:
    """An issuectl issue in the shape the fleet works with: id is its slug, parent its epic. A story is a task
    labelled story (its type is task, its label marks it); children inherit the labels but not that one."""
    labels = list(i.get("labels") or [])
    status = "closed" if i["status"] in CLOSING else STATUS_IN.get(i["status"], i["status"])
    body = (i.get("body") or "").strip()
    if body.startswith("# "):  # the title heading issuectl writes first is not the description
        body = body.split("\n", 1)[1].strip() if "\n" in body else ""
    body = re.split(r"(?m)^## Beads import\b", body, maxsplit=1)[0].strip()  # the migration record stays in the file
    return {
        "id": i["slug"], "title": i["title"], "status": status,
        "issue_type": "story" if "story" in labels else i["type"],
        "priority": PRIORITY_IN.get(i.get("priority"), 2), "labels": labels,
        "parent": i.get("epic"), "blocked_by": [b.lstrip("@") for b in (i.get("blocked_by") or [])],
        "description": body, "created_at": i.get("created") or "",
    }


def _all() -> list[dict]:
    global _cache
    if _cache is None:
        _cache = [_norm(i) for i in (_json("list", "--all") or [])]
    return _cache


def available() -> bool:
    return bool(shutil.which("issuectl")) and ISSUES_DIR.is_dir()


def show(slug: str) -> dict:
    data = _json("show", slug)
    if not data:
        raise TrackerError(f"no issue {slug!r}")
    return _norm(data)


def issues(*filters: str) -> list[dict]:
    """Issues matching `--label X`, `--type X`, `--parent X` and `--status X` (all of them, closed included)."""
    labels: list[str] = []
    want: dict[str, str] = {}
    it = iter(filters)
    for f in it:
        value = next(it)
        if f == "--label":
            labels.append(value)
        elif f in ("--type", "--parent", "--status"):
            want[f[2:]] = value
        else:
            raise TrackerError(f"unsupported filter {f}")
    out = []
    for i in _all():
        if want.get("type") and i["issue_type"] != want["type"]:
            continue
        if want.get("parent") and i["parent"] != want["parent"]:
            continue
        if want.get("status") and i["status"] != want["status"]:
            continue
        if not all(l in i["labels"] for l in labels):
            continue
        out.append(i)
    return out


def ready_list(*labels: str) -> list[dict]:
    """Open issues whose every blocker is closed (a blocker that is gone counts as closed), by priority then age."""
    by_id = {i["id"]: i for i in _all()}
    out = [i for i in _all()
           if i["status"] == "open" and all(l in i["labels"] for l in labels)
           and all(by_id.get(b, {"status": "closed"})["status"] == "closed" for b in i["blocked_by"])]
    return sorted(out, key=lambda i: (i["priority"], i["created_at"], i["id"]))


def label_value(issue: dict, prefix: str) -> str | None:
    return next((l.split(":", 1)[1] for l in issue.get("labels", []) if l.startswith(prefix + ":")), None)


def epics() -> dict[str, dict]:
    """One epic per mod. Repeated seeding left some mods with copies: the canonical one is the one in progress,
    else the one with the most work under it, else the oldest."""
    kids = Counter(i["parent"] for i in _all() if i["parent"])
    found: dict[str, list[dict]] = {}
    for i in issues("--type", "epic", "--label", "mod"):
        mod = label_value(i, "mod")
        if mod:
            found.setdefault(mod, []).append(i)
    return {mod: min(cands, key=lambda e: (e["status"] != "in_progress", -kids[e["id"]], e["created_at"], e["id"]))
            for mod, cands in found.items()}


def epic(mod: str) -> dict:
    e = epics().get(mod)
    if not e:
        raise TrackerError(f"no epic for mod {mod!r} (issuectl ls -t epic --label mod:{mod} --all)")
    return e


def create(title: str, kind: str, *, parent: str | None = None, labels: list[str] | tuple = (),
           deps: list[str] | tuple = (), priority: int | None = None, description: str = "") -> str:
    """File an issue and return its slug. kind: story (a task labelled story), task, feature, bug or epic."""
    itype = "task" if kind == "story" else kind
    all_labels = list(dict.fromkeys(list(labels) + (["story"] if kind == "story" else [])))
    args = ["create", "--title", title, "-t", itype]
    if parent:
        args += ["--epic", parent]
    for l in all_labels:
        args += ["-l", l]
    if priority is not None:
        args += ["-p", PRIORITY_OUT.get(priority, "normal")]
    body = _body_file(description) if description else None
    try:
        if body:
            args += ["--body-file", str(body)]
        slug = _json(*args)["slug"]
    finally:
        if body:
            body.unlink(missing_ok=True)
    _written()
    for d in deps:
        dep_add(slug, d)
    return slug


def set_status(slug: str, status: str) -> None:
    """open, in_progress, blocked (needs a person) or deferred. Use close_issue for a closed status."""
    if status not in STATUS_OUT:
        raise TrackerError(f"no status {status!r}")
    _run("update", slug, "--status", STATUS_OUT[status])
    _written()


def close_issue(slug: str, reason: str = "") -> None:
    """Close with issuectl's status for the type (done, or fixed for a bug), the reason kept as a note."""
    args = ["close", slug, "--as", "conductor"]
    if reason:
        args += ["--comment", reason[:2000]]
    _run(*args)
    _written()


def reopen(slug: str, reason: str = "") -> None:
    _run("update", slug, "--status", "open")
    if reason:
        comment(slug, reason)
    _written()


def comment(slug: str, text: str, author: str = "conductor") -> None:
    _run("note", slug, "--as", re.sub(r"\s+", "-", author.strip()) or "conductor", "--message", text)
    _written()


def set_body(slug: str, text: str) -> None:
    body = _body_file(text)
    try:
        _run("update", slug, "--body-file", str(body))
    finally:
        body.unlink(missing_ok=True)
    _written()


def add_label(slug: str, label: str) -> None:
    _run("update", slug, "--add-label", label)
    _written()


def dep_add(slug: str, blocker: str) -> None:
    """`slug` is blocked by `blocker`."""
    _run("update", slug, "--add-blocked-by", blocker)
    _written()


# ---------------------------------------------------------------- status

def mod_status(mod: str) -> dict:
    """The mod's epic and stories, as the gate, runner and board need them."""
    e = epic(mod)
    family = issues("--label", f"mod:{mod}")
    stories = {}
    for i in family:
        step = label_value(i, "step")
        if i["issue_type"] == "story" and step and step.isdigit() and i["parent"] == e["id"]:
            stories[int(step)] = i
    counts = {n: [0, 0] for n in stories}
    unsettled = {n: 0 for n in stories}  # children neither closed nor deferred
    by_id = {s["id"]: n for n, s in stories.items()}
    for i in family:
        n = by_id.get(i["parent"])
        if n is not None and i["issue_type"] in ("feature", "bug", "task"):
            counts[n][1] += 1
            counts[n][0] += i["status"] == "closed"
            unsettled[n] += i["status"] not in ("closed", "deferred")
    # Core work this mod needs (or, for rekkit_core itself, all pending core work) is part of step 3 Code.
    core = core_work(None if mod == CORE else mod)
    if 3 in stories and core:
        unsettled[3] += len(core)
        counts[3][1] += len(core)
    # A step is done when its story is closed and nothing under it is still open (closing a story early
    # does not make the work under it done).
    pending = [n for n in AGENT_STEPS if stories.get(n, {}).get("status") != "closed" or unsettled.get(n)]
    if mod == CORE:  # the core is not ported through the steps any more: its work is its pending core work
        pending = [3] if core else []
    finished = e["status"] in ("closed", "blocked", "deferred") or (bool(stories) and not pending)
    return {"mod": mod, "epic": e, "stories": stories, "counts": counts, "finished": finished,
            "unsettled": unsettled, "core": core,
            "next": f"step {pending[0]} {STEPS[pending[0]]}" if pending else "the human's review"}


def core_work(mod: str | None) -> list[dict]:
    """Open core work (direct children of the rekkit_core epic) for one mod, or all of it."""
    try:
        e = epic(CORE)
    except TrackerError:
        return []
    kids = [i for i in issues("--parent", e["id"]) if i["issue_type"] != "story" and i["status"] not in SETTLED]
    return [i for i in kids if mod is None or f"for:{mod}" in i.get("labels", [])]


def core_sync() -> str:
    """rekkit_core's epic is open while core work is pending and closed when none is; never in progress
    unless someone runs a fleet on it, so it never stands in for the mod being ported."""
    e = epic(CORE)
    pending = core_work(None)
    if pending and e["status"] == "closed":
        reopen(e["id"], f"Reopened: {len(pending)} core task(s) pending ({', '.join(i['id'] for i in pending[:8])}).")
        return f"rekkit_core reopened: {len(pending)} core task(s) pending."
    if not pending and e["status"] == "open":
        close_issue(e["id"], "No core work pending; reopens when a mod needs more.")
        return "rekkit_core closed: no core work pending."
    return f"rekkit_core {e['status']}: {len(pending)} core task(s) pending."


def core_need(mod: str, title: str, why: str = "") -> str:
    """Core work a mod needs: a feature under rekkit_core labelled for:<mod>, done in this mod's run."""
    e = epic(CORE)
    made = create(title if title.startswith("rekkit_core") else f"rekkit_core: {title}", "feature",
                  parent=e["id"], labels=["core", f"for:{mod}"], description=why)
    return f"{made} filed under rekkit_core for {mod}. {core_sync()}"


def active_mod() -> str | None:
    active = [m for m, e in epics().items() if e["status"] == "in_progress"]
    return next((m for m in active if m != CORE), active[0] if active else None)


def next_mod() -> str | None:
    """The first ready epic: open, every epic it needs closed (priority, then age)."""
    core_sync()  # rekkit_core is ready only while core work is pending
    ready_ids = {i["id"] for i in ready_list() if i["issue_type"] == "epic"}
    canonical = sorted(epics().items(), key=lambda kv: (kv[1]["priority"], kv[1]["created_at"], kv[0]))
    return next((mod for mod, e in canonical if e["id"] in ready_ids and mod != CORE), None)


# ---------------------------------------------------------------- conductor commands

def start(mod: str) -> dict:
    """Create the mod's stories if missing and set its epic in progress."""
    e = epic(mod)
    if mod == CORE:  # no steps: the core's work is its pending core work (core_work)
        set_status(e["id"], "in_progress")
        return mod_status(mod)
    have = mod_status(mod)["stories"]
    ids = {n: s["id"] for n, s in have.items()}
    for n, name in STEPS.items():
        if n in ids:
            continue
        ids[n] = create(f"{mod}: {n} {name}", "story", parent=e["id"], labels=[f"mod:{mod}", f"step:{n}"],
                        deps=[ids[d] for d in STEP_DEPS.get(n, []) if d in ids])
        if n == 11:
            set_status(ids[n], "deferred")
    if e["status"] == "open":
        set_status(e["id"], "in_progress")
    return mod_status(mod)


def add_children(mod: str, step: int, entries: list[tuple]) -> int:
    """Features (title, type, labels[, description]) under a story, skipping titles it already has."""
    story = mod_status(mod)["stories"].get(step)
    if not story:
        raise TrackerError(f"{mod} has no step {step} story: run make fleet-start MOD={mod}")
    existing = {i["title"] for i in issues("--parent", story["id"])}
    todo = [(e[0], e[1], e[2], e[3] if len(e) > 3 else "") for e in entries if e[0] not in existing]
    # Children carry the story's labels (mod:<id>, step:<n>...): mod_status finds a mod's issues by them.
    inherited = [l for l in story.get("labels", []) if l != "story"]
    for title, kind, labels, description in todo:
        create(title, kind, parent=story["id"], labels=sorted(set(inherited) | set(labels)),
               description=description)
    return len(todo)


def features(mod: str) -> str:
    """Code, Extract assets and Harness features from the spec's feature list."""
    path = WORK / "planning" / "mods" / f"{mod}.features.json"
    if not path.is_file():
        raise TrackerError(f"{path.relative_to(MAIN)} is missing: the spec-writer writes it at the end of step 1")
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
    # Every item and block item must be findable in the creative menu, in one of the mod's tabs.
    block_dir = MAIN / "mods" / mod / "content" / "data" / mod / "horizon" / "block"

    def has_item(name: str) -> bool:
        try:
            return json.loads((block_dir / f"{name}.json").read_text(encoding="utf-8")).get("item", True)
        except (OSError, ValueError):
            return True
    made += add_children(mod, 8, [(f"{mod}: creative {n}", "feature", ["harness", "creative"])
                                  for n in [b for b in blocks if has_item(b)] + items])
    return f"{made} features added for {mod} ({len(blocks)} blocks, {len(items)} items, {len(subsystems)} subsystems)."


def gaps(mod: str) -> str:
    """New-assets features from scripts/pack_mod.py's completeness check."""
    r = subprocess.run([sys.executable, str(MAIN / "scripts" / "pack_mod.py"), "--gaps", "--mod", mod],
                       cwd=MAIN, capture_output=True, text=True)
    if r.returncode not in (0, 1):
        raise TrackerError(r.stderr.strip()[-800:])
    found = json.loads(r.stdout or "[]")
    made = add_children(mod, 6, [(f"{mod}: new asset {g['path']}", "feature", ["asset", "new-asset"]) for g in found])
    return f"{len(found)} missing asset(s), {made} new feature(s) under {mod} step 6."


# Where each kind of difference from the original is fixed: (step, role).
CONTRACT_OWNERS = {"spec-writer": 1, "asset-smith": 5, "content-author": 3, "domain-builder": 3, "hook-engineer": 3,
                   "test-smith": 3}
# Live failures that are the whole mod's, not one block's or item's (the game log, a crash, the launch).
MOD_WIDE_STAGES = ("0_", "8_", "harness", "build")
# Live-harness stages that count against a contract (the screen, shape and scenario checks of a block).
LIVE_CONTRACT_STAGES = ("3_", "4_", "5_", "10_", "11_", "12_")


def contract(mod: str) -> str:
    """One task per block or item that differs from the original (scripts/contract_check.py, and the
    live harness's screen, shape and scenario checks in the last target/e2e-report/summary.json),
    under the step that fixes it. A closed one whose difference is back is reopened; one whose
    difference is gone is closed."""
    sys.path.insert(0, str(MAIN / "scripts"))
    import contract_check
    found = contract_check.check(mod)
    summary = MAIN / "target" / "e2e-report" / "summary.json"
    try:
        sessions = json.loads(summary.read_text(encoding="utf-8")).get("phases", {}).get("isolated", [])
    except (OSError, ValueError):
        sessions = []
    # What the last live run of this mod tested: a block or item closes only when it was tested there
    # and nothing failed (no live run, or a scenario that never ran, is no evidence that it works).
    tested: set[str] = set()
    ran_scenarios: set[str] = set()
    for session in sessions:
        if mod not in session.get("mods", []):
            continue
        for group in ("blocks", "creative", "items"):
            tested |= {k.split(":", 1)[-1] for k in session.get(group) or {}}
        ran_scenarios |= {r["scenario"] for r in session.get("steps", []) if r.get("scenario") and "cmd" in r}
        for e in session.get("errors", []):
            stage = str(e.get("stage", ""))
            if stage.startswith(MOD_WIDE_STAGES):
                continue
            role = ("domain-builder" if stage.startswith(LIVE_CONTRACT_STAGES) else "content-author")
            subject = e.get("block") or e.get("item") or e.get("entity")
            for sub in [subject] if subject else (e.get("for") or []):
                found.append({"for": sub, "kind": "live " + "_".join(stage.split("_")[:2]), "owner": role,
                              "why": (f"scenario {e['scenario']!r}: " if e.get("scenario") and e.get("scenario") not in str(e.get("error")) else "")
                              + str(e.get("error"))[:400]})
    contract_all = contract_check.load_contract(mod) or {}
    untested: dict[str, str] = {}
    for sc in contract_all.get("scenarios", []):
        if sc.get("name") not in ran_scenarios:
            for sub in sc.get("for", []):
                untested.setdefault(sub, sc.get("name", "?"))
    grouped: dict[tuple[int, str, str], list[dict]] = {}
    for p in found:
        # The outline is one pass over the mod: one task for it, not one per block.
        subject = "the outline" if p["owner"] == "spec-writer" else p["for"].split(":", 1)[-1]
        grouped.setdefault((CONTRACT_OWNERS.get(p["owner"], 3), p["owner"], subject), []).append(p)
    st = mod_status(mod)
    made = updated = reopened = 0
    for step in sorted({k[0] for k in grouped}):
        story = st["stories"][step]
        kids = {i["title"]: i for i in issues("--parent", story["id"])}
        entries = []
        for (s_, owner, subject), ps in sorted(grouped.items()):
            if s_ != step:
                continue
            kinds = ", ".join(sorted({p["kind"] for p in ps}))
            text = (f"Differs from the original ({kinds}), for {owner}:\n"
                    + "\n".join(f"- {p['for']}: {p['why']}" if subject == "the outline" else f"- {p['why']}"
                                for p in ps[:300])
                    + f"\n\nRecheck: make contract-check MOD={mod}")
            # The block's or item's own feature carries it: one per block, small enough for one agent.
            prefix = "assets for " if step == 5 else ""
            existing = next((kids[t] for t in (f"{mod}: {prefix}block {subject}", f"{mod}: {prefix}item {subject}",
                                               f"{mod}: match the original: {subject} ({owner})") if t in kids), None)
            if existing:
                set_body(existing["id"], text)
                updated += 1
                if existing["status"] == "closed":
                    reopen(existing["id"], "Differs from the original: see its description")
                    reopened += 1
            else:
                entries.append((f"{mod}: match the original: {subject} ({owner})", "bug", ["contract", owner], text))
        made += add_children(mod, step, entries)
        if story["status"] == "closed":
            reopen(story["id"], "Differs from the original (make contract-check)")
    # Tasks this command filed earlier whose difference is gone: close them.
    fixed = [i for n, story in st["stories"].items() for i in issues("--parent", story["id"])
             if "contract" in i.get("labels", []) and i["status"] != "closed"
             and i["title"].startswith(f"{mod}: match the original: ")
             and not any(i["title"] == f"{mod}: match the original: {sub} ({own})" for (_, own, sub) in grouped)]
    for i in fixed:
        close_issue(i["id"], "No difference from the original any more (make contract-check)")
    # A block's or item's own feature closes once it matches the original (and every attribute's scenario passed).
    differs = {sub for (_, _, sub) in grouped}
    feature = re.compile(rf"{re.escape(mod)}: (?:assets for )?(?:block|item) (\S+)$")
    matching = [i for n in (3, 5) if n in st["stories"] for i in issues("--parent", st["stories"][n]["id"])
                if i["status"] != "closed" and (m := feature.match(i["title"])) and m.group(1) not in differs
                and m.group(1) in tested and m.group(1) not in untested
                and found is not None and not any(p["owner"] == "spec-writer" for p in found)]
    for i in matching:
        close_issue(i["id"], "Matches the original: make contract-check and the last harness run find no difference")
    fixed += matching
    if found and st["epic"]["status"] == "closed":
        reopen(st["epic"]["id"])
    return (f"{len(found)} difference(s) from the original in {len(grouped)} block(s)/item(s): {updated} task(s) "
            f"updated ({reopened} reopened), {made} new, {len(fixed)} closed as fixed." if found
            else f"{mod} matches its contract." + (f" {len(fixed)} task(s) closed as fixed." if fixed else ""))


TICKET_ROLES = {"content-author", "domain-builder", "domain-builder-pro", "hook-engineer", "test-smith",
                "asset-smith", "scaffolder", "scribe"}
TICKET_BUDGET_K = 200  # thousand tokens of work a ticket is aimed at: one agent, one context
# What makes a ticket's turns (planning/token_audit/fix-plan.md B2): every file written and every spec
# section or Java member read is about a turn, and each turn re-sends all the context before it.
TICKET_MAX_FILES = 6
TICKET_MAX_READS = 8
# Files many tickets add a line or entry to: sharing them does not need an order between tickets.
SHARED_FILES = re.compile(r"/lang/[^/]+\.json$|/creative_tab/|/data/minecraft/tags/|/data/c/tags/|Cargo\.lock$")
TICKET_SCHEMA = f"""planning/mods/<id>.tickets/<part>.json, one file per part (keys unique across parts):

{{"tickets": [{{
  "key": "part12-solar-panel",                 # <part>-<name>, unique
  "title": "Solar panel: model, generation, storage",
  "role": "domain-builder-pro",                # {', '.join(sorted(TICKET_ROLES))}
  "step": 3,                                   # the story it is under (3 Code; 5, 6 assets)
  "effort_k": 120,                             # aimed at <= {TICKET_BUDGET_K} (thousand tokens, reading included)
  "covers": ["solar_panel"],                   # outline ids (blocks, items, features) it makes work
  "files": ["mods/redpower_2/src/blutricity/solar.rs",
            "planning/mods/redpower_2.contract/solar_panel.json"],   # at most {TICKET_MAX_FILES}
  "spec": ["2.5 Blutricity Power Grid"],       # spec headings; spec + java at most {TICKET_MAX_READS}
  "java": ["TileSolarPanel#updateEntity", "BlockMachinePanel#getBlockTexture"],
  "after": ["part12-bluewire-grid"],           # keys that must be closed first
  "description": "What to build and how it behaves. Done: make contract-check clean for solar_panel; its level 1, 3 and 5 scenarios pass."
}}]}}

`make fleet-task` prints each ticket's spec sections and Java members in full for its agent, so name only
what it needs. Bigger work is several tickets in order (`after`). Two tickets editing one file are ordered
(lang, creative tab and tag files excepted)."""


def ticket_plan(mod: str) -> tuple[list[dict], list[str]]:
    """The spec-writers' tickets (planning/mods/<id>.tickets/*.json, one file per part of the decompiled
    code) and what is wrong with them: sizes, roles, unknown or circular `after`, two tickets that edit
    the same file with no order between them, and blocks, items or attributes no ticket covers."""
    folder = WORK / "planning" / "mods" / f"{mod}.tickets"
    tickets: list[dict] = []
    problems: list[str] = []
    for f in sorted(folder.glob("*.json")) if folder.is_dir() else []:
        try:
            part = json.loads(f.read_text(encoding="utf-8"))
        except ValueError as exc:
            problems.append(f"{f.name}: not JSON ({exc})")
            continue
        for t in part.get("tickets", []):
            t["_file"] = f.name
            tickets.append(t)
    if not tickets:
        return [], [f"no tickets in {folder.relative_to(WORK)}/ (the spec-writers write them in step 1)"]
    keys: dict[str, dict] = {}
    for t in tickets:
        where = f"{t['_file']}: {t.get('key', '?')}"
        for field in ("key", "title", "role", "effort_k", "description"):
            if not t.get(field):
                problems.append(f"{where}: no {field}")
        if t.get("key") in keys:
            problems.append(f"{where}: key also in {keys[t['key']]['_file']}")
        keys[t.get("key", "")] = t
        if t.get("role") and t["role"] not in TICKET_ROLES:
            problems.append(f"{where}: role {t['role']!r} is not one of {', '.join(sorted(TICKET_ROLES))}")
        if isinstance(t.get("effort_k"), (int, float)) and t["effort_k"] > TICKET_BUDGET_K:
            problems.append(f"{where}: {t['effort_k']}k tokens of work; split it into tickets of at most {TICKET_BUDGET_K}k")
        if len(t.get("files", [])) > TICKET_MAX_FILES:
            problems.append(f"{where}: {len(t['files'])} files; at most {TICKET_MAX_FILES} a ticket (split it)")
        reads = len(t.get("spec", [])) + len(t.get("java", []))
        if reads > TICKET_MAX_READS:
            problems.append(f"{where}: {reads} spec sections and Java members to read; at most {TICKET_MAX_READS} "
                            "(name only what it needs, or split it)")
        if int(t.get("step", 3)) not in STEPS:
            problems.append(f"{where}: no step {t.get('step')}")
    for t in tickets:
        for dep in t.get("after", []):
            if dep not in keys:
                problems.append(f"{t['_file']}: {t['key']} is after {dep!r}, which no ticket has as its key")

    @lru_cache(maxsize=None)
    def before(key: str, trail: tuple = ()) -> frozenset:
        """Every ticket that must be done before `key`."""
        if key in trail:
            cycle = f"circular order: {' -> '.join(trail[trail.index(key):] + (key,))}"
            if not any(p.startswith("circular order") and set(p.split(": ")[1].split(" -> ")) ==
                       set(cycle.split(": ")[1].split(" -> ")) for p in problems):
                problems.append(cycle)
            return frozenset()
        out: set[str] = set()
        for dep in keys.get(key, {}).get("after", []):
            if dep in keys:
                out |= {dep} | before(dep, trail + (key,))
        return frozenset(out)
    for t in tickets:
        before(t.get("key", ""))  # reports every circular order once it is reached
    for a in tickets:
        for b in tickets:
            if a["key"] >= b["key"]:
                continue
            same = {f for f in set(a.get("files", [])) & set(b.get("files", [])) if not SHARED_FILES.search(f)}
            if same and a["key"] not in before(b["key"]) and b["key"] not in before(a["key"]):
                problems.append(f"{a['key']} and {b['key']} both edit {', '.join(sorted(same))} with no order: "
                                f"put one after the other (or split the file)")
    outline_path = WORK / "planning" / "mods" / f"{mod}.features.json"
    try:
        outline = json.loads(outline_path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        outline = {}
    covered = {c for t in tickets for c in t.get("covers", [])}
    for kind in ("blocks", "items", "features"):
        for e in outline.get(kind, []):
            if e.get("id") not in covered:
                problems.append(f"no ticket covers {kind[:-1]} {e.get('id')} (list it in a ticket's covers)")
    return tickets, problems


def tickets(mod: str, check_only: bool = False) -> str:
    """File the spec-writers' tickets under their steps, with their order as dependencies (a ticket is ready
    when every ticket it is after is closed). Refuses a plan with problems; files only new keys.
    The spec-writers' part outlines and attributes are merged into the spec and outline first."""
    import spec_merge
    merged = spec_merge.merge(mod)
    plan, problems = ticket_plan(mod)
    if problems:
        raise TrackerError(f"{len(problems)} problem(s) in {mod}'s tickets (send them back to the spec-writer):\n"
                           + "\n".join(f"- {p}" for p in problems[:80]))
    if check_only:
        return f"{merged}\n{len(plan)} tickets for {mod}: sizes, roles, order, shared files and coverage are fine."
    st = mod_status(mod)
    filed = {label_value(i, "key"): i["id"] for i in issues("--label", "ticket", "--label", f"mod:{mod}")
             if label_value(i, "key")}
    new = [t for t in plan if t["key"] not in filed]
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    reopened_steps: set[int] = set()
    for t in new:
        step = int(t.get("step", 3))
        story = st["stories"][step]
        inherited = [l for l in story.get("labels", []) if l != "story"]
        body = (t["description"] + "\n\nSize: ~" + f"{t['effort_k']}k tokens."
                + (f"\nCovers: {', '.join(t.get('covers', []))}" if t.get("covers") else "")
                + (f"\nRead first: spec {', '.join(t['spec'])}" if t.get("spec") else "")
                + (f"\nJava: {', '.join(t['java'])}" if t.get("java") else "")
                + (f"\nFiles: {', '.join(t['files'])}" if t.get("files") else ""))
        create(f"{mod}: {t['title']}", "task", parent=story["id"], description=body,
               labels=sorted(set(inherited) | {"ticket", f"role:{t['role']}", f"key:{t['key']}"}))
        if story["status"] == "closed" and step not in reopened_steps:
            reopen(story["id"], "New tickets")
            reopened_steps.add(step)
    # Orders between tickets (within a step, across steps, or onto tickets filed before) go in once all exist.
    # Adding a blocker that is already there is a no-op.
    ids = {label_value(i, "key"): i["id"] for i in issues("--label", "ticket", "--label", f"mod:{mod}")
           if label_value(i, "key")}
    for t in new:
        for d in t.get("after", []):
            if d in ids:
                dep_add(ids[t["key"]], ids[d])
    return f"{len(new)} new ticket(s) filed for {mod} ({len(plan) - len(new)} already filed)."


SPLIT = "needs-split"  # a ticket two agents could not finish: split it, never start a third (fix-plan B3)


def ready(mod: str) -> list[dict]:
    """Open tickets of the mod whose every predecessor is closed: one subagent each. A ticket labelled
    needs-split waits for its spec-writer instead."""
    return [i for i in ready_list("ticket", f"mod:{mod}") if SPLIT not in (i.get("labels") or [])]


def partials_before(bead: str) -> int:
    """Partial reports already filed for a bead."""
    n = 0
    for f in (INBOX / "done").glob(f"{bead}-*.json"):
        try:
            n += json.loads(f.read_text(encoding="utf-8")).get("status") == "partial"
        except (OSError, ValueError):
            continue
    return n


def progressed(bead: str) -> bool:
    """Whether the bead's progress notes record any finished piece."""
    notes = STATE_DIR / "progress" / f"{bead}.md"
    return notes.is_file() and any(ln.strip() for ln in notes.read_text(encoding="utf-8", errors="replace").splitlines())


def inbox() -> str:
    """File every subagent report: a comment, the issue's new status, and follow-ups as new issues. One report
    that cannot be filed never holds up the others: a status the tracker refuses (a story still blocked by its
    step before, a report written too early) is noted on the issue, which keeps its status, and the report
    counts as filed; one that is not JSON or names no issue is moved to inbox/bad/."""
    done_dir = INBOX / "done"
    done_dir.mkdir(parents=True, exist_ok=True)
    # Each report's finished steps (comment, follow_up:<n>, status) are recorded here as they happen, so a
    # run that dies part way is finished by the next one without repeating a comment or a follow-up issue.
    filing = INBOX / "filing"
    filing.mkdir(parents=True, exist_ok=True)
    filed, notes = [], []
    stamp = time.strftime("%Y%m%dT%H%M%S")
    for path in sorted(INBOX.glob("*.json")):
        try:
            report = json.loads(path.read_text(encoding="utf-8"))
            bead = report.get("bead") or path.stem
            issue = show(bead)
        except (ValueError, TrackerError) as exc:
            bad = INBOX / "bad"
            bad.mkdir(exist_ok=True)
            path.rename(bad / f"{path.stem}-{stamp}.json")
            notes.append(f"{path.name}: not filed ({str(exc)[:160]}); moved to {bad.relative_to(MAIN)}/ for its role to rewrite")
            continue
        status = report.get("status", "closed")
        print(f"Processing {bead} with status {status}")
        lines = [f"[{report.get('role', 'subagent')}] {status}: {report.get('summary', '')}".strip()]
        print(f"  Generating lines for {bead}")
        if report.get("files"):
            lines.append("Files: " + ", ".join(report["files"]))
        if report.get("notes"):
            lines.append(report["notes"])
        progress_path = filing / f"{path.stem}.json"
        steps = json.loads(progress_path.read_text(encoding="utf-8")) if progress_path.exists() else []

        def mark(step: str) -> None:
            steps.append(step)
            progress_path.write_text(json.dumps(steps), encoding="utf-8")

        if "comment" not in steps:
            comment(bead, "\n".join(lines), author=str(report.get("role") or "subagent"))
            mark("comment")
        parent = issue.get("parent") if issue["issue_type"] not in ("story", "epic") else bead
        mod = label_value(issue, "mod")
        for i, f in enumerate(report.get("follow_ups", [])):
            if f"follow_up:{i}" in steps:
                continue
            kind = f.get("type", "bug") if f.get("type") in ("bug", "feature", "task") else "bug"
            title = f.get("title", "follow-up")
            if title.lower().startswith(("rekkit_core", "core:", "core-needs")) and mod and mod != CORE:
                core_need(mod, title.split(":", 1)[1].strip() if ":" in title else title, f.get("why", ""))
            else:
                create(title, kind, parent=parent, description=f.get("why", ""))
            mark(f"follow_up:{i}")
        if "status" not in steps:
            try:
                if status == "done":
                    close_issue(bead, (report.get("summary") or "done")[:200])
                elif status == "blocked":
                    set_status(bead, "blocked")
                elif status == "partial" and "ticket" in (issue.get("labels") or []) and \
                        (partials_before(bead) >= 1 or not progressed(bead)):
                    # A restart pays the floor and the brief again and stops where the last one did.
                    set_status(bead, "open")
                    add_label(bead, SPLIT)
                    comment(bead, "Two agents could not finish this ticket (or one finished nothing): split it "
                                  "into smaller tickets (a spec-writer in split mode), then close this one.")
                    notes.append(f"{bead}: partial again (or no progress); labelled {SPLIT}, not restarted")
                else:
                    set_status(bead, "open")
            except TrackerError as exc:
                reason = str(exc).strip().splitlines()[-1][:200]
                comment(bead, f"Report filed, status not changed to {status}: {reason}. Redo this work when "
                              "the issue is ready (make fleet-ready).")
                notes.append(f"{bead}: report filed, still {issue['status']} ({reason})")
            mark("status")
        path.rename(done_dir / f"{path.stem}-{stamp}.json")
        progress_path.unlink(missing_ok=True)
        filed.append(f"{bead} ({status}, {len(report.get('follow_ups', []))} follow-up(s))")
    sync = core_sync() if filed else ""
    if filed:  # the reported work is one version ahead of the last commit (bundled until committed)
        sys.path.insert(0, str(MAIN / "scripts"))
        import version_sync
        bumped = version_sync.sync_units()
        sync = "\n".join(x for x in [sync, *bumped] if x)
    out = ("Filed: " + ", ".join(filed) + (f"\n{sync}" if sync else "")) if filed else "Inbox empty."
    return out + "".join(f"\nNote: {n}" for n in notes)


# ---------------------------------------------------------------- board

def board() -> str:
    """planning/fleet-board.md, generated from the issues."""
    all_issues = _all()
    by_mod: dict[str, list[dict]] = {}
    for i in all_issues:
        mod = label_value(i, "mod")
        if mod:
            by_mod.setdefault(mod, []).append(i)
    rows = []
    order = sorted(epics().values(), key=lambda e: (e.get("priority", 2), e.get("created_at", "")))
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
        if mod == CORE:  # grows with the mods: open only while some mod's core work is pending
            pending = core_work(None)
            fors = sorted({l[4:] for i in pending for l in i.get("labels", []) if l.startswith("for:")})
            state = f"growing: {len(pending)} for {', '.join(fors)}" if pending else "done (grows as mods need it)"
        desc = (e.get("description") or "").strip().splitlines()
        rows.append([str(n), mod, f"`{label_value(e, 'path') or 'mods/' + mod}`", label_value(e, "wave") or "",
                     label_value(e, "tier") or "", ",".join(deps) or "—", state, *cells,
                     desc[0] if desc else "", e["id"]])
    head = ["#", "Mod", "Path", "Wave", "Tier", "Deps", "State", *(f"{n} {s}" for n, s in STEPS.items()), "Notes", "Issue"]
    lines = [
        "# Fleet board",
        "",
        "Generated from `issues/` by `make board`: do not edit. Change progress with `issuectl` (the conductor) and regenerate.",
        "One mod at a time, in this order (`issuectl dag` gives the next). Steps are the mod's stories; `▶ 12/42` is a",
        "story in progress with 12 of its 42 features or bugs closed.",
        "",
        "State: `queued` · `active` · `blocked` · `done`. Steps: `·` todo · `▶` in progress · `✓` done · `✗` blocked · `—` deferred.",
        "",
        "| " + " | ".join(head) + " |",
        "|" + "|".join("---:" if h == "#" else "---" for h in head) + "|",
        *("| " + " | ".join(r) + " |" for r in rows),
        "",
    ]
    BOARD.parent.mkdir(parents=True, exist_ok=True)
    BOARD.write_text("\n".join(lines), encoding="utf-8")
    return f"Wrote {BOARD.relative_to(MAIN)} ({len(rows)} mods)."


def seed(board_md: Path) -> str:
    """An epic per row of the old markdown board, in its order, with its dependencies (one-time)."""
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
                          deps=[ids[d] for d in needs if d != CORE], priority=priority.get(wave, 4),
                          description=cells[-2] if len(cells) > 18 else "")
        if state == "done":
            close_issue(ids[mod], "done before the move to the tracker")
    return f"{len(ids) - len(have)} epic(s) created, {len(have)} already there."


def prime_text() -> str:
    """The tracker's context for an agent: the commands it may use and where the port stands."""
    try:
        mod = active_mod()  # read only: next_mod would reopen or close rekkit_core
    except TrackerError as exc:
        mod = None
        note = f"(tracker: {exc})"
    else:
        note = ""
    where = f"Mod in progress: {mod}." if mod else "No mod in progress."
    return "\n".join([
        "Issue tracker: issuectl, issues under issues/ (committed with the code).",
        f"{where} {note}".rstrip(),
        "Read: `issuectl show <slug>`, `issuectl ls <query> --all`, `issuectl dag`, `make fleet-context MOD=<id>`.",
        "Write (the conductor only): `issuectl update|close|note|create`, or the make targets.",
    ])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["status", "start", "features", "gaps", "contract", "tickets",
                                            "tickets-check", "tickets-schema", "ready", "inbox", "board", "seed",
                                            "core-need", "core-sync"])
    parser.add_argument("mod", nargs="?")
    parser.add_argument("--title", help="core-need: what core needs")
    parser.add_argument("--why", default="", help="core-need: which mod feature needs it, and why it is shared")
    args = parser.parse_args()
    if args.command == "tickets-schema":  # needs no tracker
        print(TICKET_SCHEMA)
        return 0
    if not available():
        print("The issue tracker is not set up: install issuectl (brew install jarimustonen/issuectl/issuectl).",
              file=sys.stderr)
        return 1
    try:
        if args.command in ("start", "features", "gaps", "contract", "tickets", "tickets-check", "ready",
                            "core-need") and not args.mod:
            raise TrackerError(f"{args.command} needs a mod id (MOD=<id>)")
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
        elif args.command == "contract":
            print(contract(args.mod))
        elif args.command in ("tickets", "tickets-check"):
            print(tickets(args.mod, check_only=args.command == "tickets-check"))
        elif args.command == "ready":
            rows = ready(args.mod)
            for i in rows:
                print(f"{i['id']:14} {label_value(i, 'role') or '?':20} {i['title']}")
            print(f"{len(rows)} ticket(s) ready for {args.mod}: `make fleet-dispatch MOD={args.mod}` works them "
                  "(or one subagent each, its role, `make fleet-task BEAD=<id> ROLE=<role>` first)."
                  if rows else f"No ticket of {args.mod} is ready.")
        elif args.command == "inbox":
            print(inbox())
        elif args.command == "board":
            print(board())
        elif args.command == "core-need":
            if not args.title:
                raise TrackerError("core-need needs TITLE=\"<what core needs>\"")
            print(core_need(args.mod, args.title, args.why))
        elif args.command == "core-sync":
            print(core_sync())
        elif args.command == "seed":
            print(seed(Path(args.mod) if args.mod else BOARD))
    except TrackerError as exc:
        print(f"[tracker] {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
