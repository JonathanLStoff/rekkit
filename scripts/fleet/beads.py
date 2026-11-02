#!/usr/bin/env python3
"""The fleet's progress in Beads (bd), the source of truth for the port.

  epic     one per mod (labels mod, mod:<id>, wave:<w>, tier:<t>, needs:<epic>), blocked by the epics it
           needs, except rekkit_core: the shared crate grows with the mods. Its epic holds core work as
           beads labelled for:<mod>; it opens while any is pending and closes when none is
           (core_sync), and a mod's step 3 Code is not done until its for:<mod> core beads are closed.
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
  core-need <mod> --title T [--why W]  core work the mod needs, under rekkit_core     [core-need]
  core-sync                 open rekkit_core while core work is pending, close it when none is
  seed                      one-time: an epic per row of the old markdown board
"""

from __future__ import annotations

import argparse
import json
from functools import lru_cache
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
    unsettled = {n: 0 for n in stories}  # children neither closed nor deferred
    by_id = {s["id"]: n for n, s in stories.items()}
    for i in family:
        n = by_id.get(i.get("parent") or i["id"].rsplit(".", 1)[0])
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
    if mod == CORE:  # the core is not ported through the steps any more: its work is its pending core beads
        pending = [3] if core else []
    finished = e["status"] in ("closed", "blocked", "deferred") or (bool(stories) and not pending)
    return {"mod": mod, "epic": e, "stories": stories, "counts": counts, "finished": finished,
            "unsettled": unsettled, "core": core,
            "next": f"step {pending[0]} {STEPS[pending[0]]}" if pending else "the human's review"}


CORE = "rekkit_core"
SETTLED = ("closed", "deferred")


def core_work(mod: str | None) -> list[dict]:
    """Open core beads (direct children of the rekkit_core epic) for one mod, or all of them."""
    try:
        e = epic(CORE)
    except BeadsError:
        return []
    kids = [i for i in issues("--parent", e["id"]) if i["issue_type"] != "story" and i["status"] not in SETTLED]
    return [i for i in kids if mod is None or f"for:{mod}" in i.get("labels", [])]


def core_sync() -> str:
    """rekkit_core's epic is open while core work is pending and closed when none is; never in progress
    unless someone runs a fleet on it, so it never stands in for the mod being ported."""
    e = epic(CORE)
    pending = core_work(None)
    if pending and e["status"] == "closed":
        bd("reopen", e["id"])
        bd("comment", e["id"], f"Reopened: {len(pending)} core bead(s) pending ({', '.join(i['id'] for i in pending[:8])}).")
        return f"rekkit_core reopened: {len(pending)} core bead(s) pending."
    if not pending and e["status"] == "open":
        bd("close", e["id"], "--reason", "No core work pending; reopens when a mod needs more.")
        return "rekkit_core closed: no core work pending."
    return f"rekkit_core {e['status']}: {len(pending)} core bead(s) pending."


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
    """The first ready epic: open, every epic it needs closed (bd ready order: priority, then age)."""
    core_sync()  # rekkit_core is ready only while core work is pending
    ready = bd("ready", "--limit", "0")
    return next((label_value(i, "mod") for i in ready if i["issue_type"] == "epic" and label_value(i, "mod")), None)


# ---------------------------------------------------------------- conductor commands

def start(mod: str) -> dict:
    """Create the mod's stories if missing and set its epic in progress."""
    e = epic(mod)
    if mod == CORE:  # no steps: the core's work is its pending core beads (core_work)
        bd("update", e["id"], "--status", "in_progress")
        return mod_status(mod)
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


def add_children(mod: str, step: int, entries: list[tuple]) -> int:
    """Features (title, type, labels[, description]) under a story, skipping titles it already has."""
    story = mod_status(mod)["stories"].get(step)
    if not story:
        raise BeadsError(f"{mod} has no step {step} story: run make fleet-start MOD={mod}")
    existing = {i["title"] for i in issues("--parent", story["id"])}
    todo = [(e[0], e[1], e[2], e[3] if len(e) > 3 else "") for e in entries if e[0] not in existing]
    # One graph plan per 200 beads: a single bd call and transaction each, instead of one per bead, so a
    # run cut short (quota, network) loses at most one chunk and a rerun adds only what is missing.
    for start in range(0, len(todo), 200):
        chunk = todo[start:start + 200]
        # The story's labels too (mod:<id>, step:<n>...): mod_status finds a mod's beads by them.
        inherited = [l for l in story.get("labels", []) if l != "story"]
        plan = {"nodes": [{"key": f"n{n}", "title": t, "type": k, "parent_id": story["id"],
                           "labels": sorted(set(inherited) | set(ls)),
                           **({"description": d} if d else {})}
                          for n, (t, k, ls, d) in enumerate(chunk)]}
        STATE_DIR.mkdir(parents=True, exist_ok=True)
        plan_file = STATE_DIR / f"plan-{story['id']}.json"
        plan_file.write_text(json.dumps(plan), encoding="utf-8")
        bd("create", "--graph", str(plan_file))
        plan_file.unlink(missing_ok=True)
    return len(todo)


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
        raise BeadsError(r.stderr.strip()[-800:])
    found = json.loads(r.stdout or "[]")
    made = add_children(mod, 6, [(f"{mod}: new asset {g['path']}", "feature", ["asset", "new-asset"]) for g in found])
    return f"{len(found)} missing asset(s), {made} new feature(s) under {mod} step 6."


# Where each kind of difference from the original is fixed: (step, role).
CONTRACT_OWNERS = {"spec-writer": 1, "asset-smith": 5, "content-author": 3, "domain-builder": 3, "hook-engineer": 3,
                   "test-smith": 3}
LIVE_CONTRACT_STAGES = ("2_shape", "4_gui", "4_menu_opens", "10_scenario")


def contract(mod: str) -> str:
    """One bead per block or item that differs from the original (scripts/contract_check.py, and the
    live harness's screen, shape and scenario checks in the last target/e2e-report/summary.json),
    under the step that fixes it. A closed bead whose difference is back is reopened; one whose
    difference is gone is left for its role to close."""
    sys.path.insert(0, str(MAIN / "scripts"))
    import contract_check
    found = contract_check.check(mod)
    summary = MAIN / "target" / "e2e-report" / "summary.json"
    try:
        sessions = json.loads(summary.read_text(encoding="utf-8")).get("phases", {}).get("isolated", [])
    except (OSError, ValueError):
        sessions = []
    for session in sessions:
        if mod not in session.get("mods", []):
            continue
        for e in session.get("errors", []):
            if str(e.get("stage", "")).startswith(LIVE_CONTRACT_STAGES):
                role = "domain-builder" if e["stage"].startswith(("4_", "10_")) else "content-author"
                for subject in [e["block"]] if e.get("block") else (e.get("for") or [mod]):
                    found.append({"for": subject, "kind": "live " + e["stage"], "owner": role,
                                  "why": str(e.get("error"))})
    grouped: dict[tuple[int, str, str], list[dict]] = {}
    for p in found:
        # The outline is one pass over the mod: one bead for it, not one per block.
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
            # The block's or item's own feature bead carries it: one bead per block, small enough for
            # one agent, many in parallel.
            prefix = "assets for " if step == 5 else ""
            existing = next((kids[t] for t in (f"{mod}: {prefix}block {subject}", f"{mod}: {prefix}item {subject}",
                                               f"{mod}: match the original: {subject} ({owner})") if t in kids), None)
            if existing:
                bd("update", existing["id"], "--description", text, parse=False)
                updated += 1
                if existing["status"] == "closed":
                    bd("reopen", existing["id"], "--reason", "Differs from the original: see its description", parse=False)
                    reopened += 1
            else:
                entries.append((f"{mod}: match the original: {subject} ({owner})", "bug", ["contract", owner], text))
        made += add_children(mod, step, entries)
        if story["status"] == "closed":
            bd("reopen", story["id"], "--reason", "Differs from the original (make contract-check)", parse=False)
    # Beads this command filed earlier whose difference is gone: close them.
    fixed = [i for n, story in st["stories"].items() for i in issues("--parent", story["id"])
             if "contract" in i.get("labels", []) and i["status"] != "closed"
             and i["title"].startswith(f"{mod}: match the original: ")
             and not any(i["title"] == f"{mod}: match the original: {sub} ({own})" for (_, own, sub) in grouped)]
    for i in fixed:
        bd("close", i["id"], "--reason", "No difference from the original any more (make contract-check)", parse=False)
    # A block's or item's own beads close once it matches the original (and every attribute's scenario passed).
    differs = {sub for (_, _, sub) in grouped}
    feature = re.compile(rf"{re.escape(mod)}: (?:assets for )?(?:block|item) (\S+)$")
    matching = [i for n in (3, 5) if n in st["stories"] for i in issues("--parent", st["stories"][n]["id"])
                if i["status"] != "closed" and (m := feature.match(i["title"])) and m.group(1) not in differs
                and found is not None and not any(p["owner"] == "spec-writer" for p in found)]
    for c in range(0, len(matching), 100):
        bd("close", *[i["id"] for i in matching[c:c + 100]], "--reason",
           "Matches the original: make contract-check and the last harness run find no difference", parse=False)
    fixed += matching
    if found and st["epic"]["status"] == "closed":
        bd("reopen", st["epic"]["id"])
    return (f"{len(found)} difference(s) from the original in {len(grouped)} block(s)/item(s): {updated} bead(s) "
            f"updated ({reopened} reopened), {made} new, {len(fixed)} closed as fixed." if found
            else f"{mod} matches its contract." + (f" {len(fixed)} bead(s) closed as fixed." if fixed else ""))


TICKET_ROLES = {"content-author", "domain-builder", "domain-builder-pro", "hook-engineer", "test-smith",
                "asset-smith", "scaffolder", "scribe"}
TICKET_BUDGET_K = 200  # thousand tokens of work a ticket may need: one agent, one context
# Files many tickets add a line or entry to: sharing them does not need an order between tickets.
SHARED_FILES = re.compile(r"/lang/[^/]+\.json$|/creative_tab/|/data/minecraft/tags/|/data/c/tags/|Cargo\.lock$")


def ticket_plan(mod: str) -> tuple[list[dict], list[str]]:
    """The spec-writers' tickets (planning/mods/<id>.tickets/*.json, one file per part of the decompiled
    code) and what is wrong with them: sizes, roles, unknown or circular `after`, two tickets that edit
    the same file with no order between them, and blocks, items or attributes no ticket covers."""
    folder = MAIN / "planning" / "mods" / f"{mod}.tickets"
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
        return [], [f"no tickets in {folder.relative_to(MAIN)}/ (the spec-writers write them in step 1)"]
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
    outline_path = MAIN / "planning" / "mods" / f"{mod}.features.json"
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
    """File the spec-writers' tickets under their steps, with their order as Beads dependencies (a ticket is
    ready when every ticket it is after is closed). Refuses a plan with problems; files only new keys."""
    plan, problems = ticket_plan(mod)
    if problems:
        raise BeadsError(f"{len(problems)} problem(s) in {mod}'s tickets (send them back to the spec-writer):\n"
                         + "\n".join(f"- {p}" for p in problems[:80]))
    if check_only:
        return f"{len(plan)} tickets for {mod}: sizes, roles, order, shared files and coverage are fine."
    st = mod_status(mod)
    filed = {label_value(i, "key"): i["id"] for i in issues("--label", "ticket", "--label", f"mod:{mod}")
             if label_value(i, "key")}
    new = [t for t in plan if t["key"] not in filed]
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    for step in sorted({int(t.get("step", 3)) for t in new}):
        story = st["stories"][step]
        inherited = [l for l in story.get("labels", []) if l != "story"]
        batch = [t for t in new if int(t.get("step", 3)) == step]
        nodes = []
        for t in batch:
            body = (t["description"] + "\n\nSize: ~" + f"{t['effort_k']}k tokens."
                    + (f"\nCovers: {', '.join(t.get('covers', []))}" if t.get("covers") else "")
                    + (f"\nRead first: spec {', '.join(t['spec'])}" if t.get("spec") else "")
                    + (f"\nJava: {', '.join(t['java'])}" if t.get("java") else "")
                    + (f"\nFiles: {', '.join(t['files'])}" if t.get("files") else ""))
            nodes.append({"key": t["key"], "title": f"{mod}: {t['title']}", "type": "task", "parent_id": story["id"],
                          "description": body,
                          "labels": sorted(set(inherited) | {"ticket", f"role:{t['role']}", f"key:{t['key']}"}),
                          "deps": [{"target": d, "type": "blocks"} for d in t.get("after", [])
                                   if d in {b["key"] for b in batch}]})
        plan_file = STATE_DIR / f"tickets-{mod}-{step}.json"
        plan_file.write_text(json.dumps({"nodes": nodes}), encoding="utf-8")
        bd("create", "--graph", str(plan_file))
        plan_file.unlink(missing_ok=True)
        if story["status"] == "closed":
            bd("reopen", story["id"], "--reason", "New tickets", parse=False)
    # Orders across steps or onto tickets filed before go in one by one.
    ids = {label_value(i, "key"): i["id"] for i in issues("--label", "ticket", "--label", f"mod:{mod}")
           if label_value(i, "key")}
    steps_of = {t["key"]: int(t.get("step", 3)) for t in plan}
    for t in new:
        for d in t.get("after", []):
            if d in filed or steps_of.get(d) != steps_of[t["key"]]:
                bd("dep", "add", ids[t["key"]], "--blocked-by", ids[d], parse=False)
    return f"{len(new)} new ticket(s) filed for {mod} ({len(plan) - len(new)} already filed)."


def ready(mod: str) -> list[dict]:
    """Open tickets of the mod whose every predecessor is closed: one subagent each."""
    out = bd("ready", "--json", "--limit", "0", "--label", "ticket", "--label", f"mod:{mod}")
    return [i for i in out if i.get("status") == "open"] if isinstance(out, list) else []


def inbox() -> str:
    """File every subagent report: a comment, the bead's new status, and follow-ups as new beads. One
    report that cannot be filed never holds up the others: a status Beads refuses (a story still blocked
    by its step before, a report written too early) is noted on the bead, which keeps its status, and
    the report counts as filed; one that is not JSON or names no bead is moved to inbox/bad/."""
    done_dir = INBOX / "done"
    done_dir.mkdir(parents=True, exist_ok=True)
    filed, notes = [], []
    stamp = time.strftime("%Y%m%dT%H%M%S")
    for path in sorted(INBOX.glob("*.json")):
        try:
            report = json.loads(path.read_text(encoding="utf-8"))
            bead = report.get("bead") or path.stem
            show = bd("show", bead)
            issue = show[0] if isinstance(show, list) else show
        except (ValueError, BeadsError) as exc:
            bad = INBOX / "bad"
            bad.mkdir(exist_ok=True)
            path.rename(bad / f"{path.stem}-{stamp}.json")
            notes.append(f"{path.name}: not filed ({str(exc)[:160]}); moved to {bad.relative_to(MAIN)}/ for its role to rewrite")
            continue
        status = report.get("status", "done")
        lines = [f"[{report.get('role', 'subagent')}] {status}: {report.get('summary', '')}".strip()]
        if report.get("files"):
            lines.append("Files: " + ", ".join(report["files"]))
        if report.get("notes"):
            lines.append(report["notes"])
        bd("comment", bead, "\n".join(lines), parse=False)
        parent = issue.get("parent") if issue["issue_type"] not in ("story", "epic") else bead
        mod = label_value(issue, "mod")
        for f in report.get("follow_ups", []):
            kind = f.get("type", "bug") if f.get("type") in ("bug", "feature", "task") else "bug"
            title = f.get("title", "follow-up")
            if title.lower().startswith(("rekkit_core", "core:", "core-needs")) and mod and mod != CORE:
                core_need(mod, title.split(":", 1)[1].strip() if ":" in title else title, f.get("why", ""))
            else:
                create(title, kind, parent=parent, description=f.get("why", ""))
        try:
            if status == "done":
                bd("close", bead, "--reason", (report.get("summary") or "done")[:200])
            elif status == "blocked":
                bd("update", bead, "--status", "blocked")
            else:
                bd("update", bead, "--status", "open")
        except BeadsError as exc:
            reason = str(exc).strip().splitlines()[-1][:200]
            bd("comment", bead, f"Report filed, status not changed to {status}: {reason}. Redo this work when "
                                "the bead is ready (make fleet-ready).", parse=False)
            notes.append(f"{bead}: report filed, still {issue['status']} ({reason})")
        path.rename(done_dir / f"{path.stem}-{stamp}.json")
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
        if mod == CORE:  # grows with the mods: open only while some mod's core work is pending
            pending = core_work(None)
            fors = sorted({l[4:] for i in pending for l in i.get("labels", []) if l.startswith("for:")})
            state = f"growing: {len(pending)} for {', '.join(fors)}" if pending else "done (grows as mods need it)"
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
                          deps=[ids[d] for d in needs if d != CORE], priority=priority.get(wave, 4), description=cells[-2] if len(cells) > 18 else "")
        if state == "done":
            bd("close", ids[mod], "--reason", "done before the move to Beads")
    return f"{len(ids) - len(have)} epic(s) created, {len(have)} already there."


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["status", "start", "features", "gaps", "contract", "tickets",
                                            "tickets-check", "ready", "inbox", "board", "seed", "core-need",
                                            "core-sync"])
    parser.add_argument("mod", nargs="?")
    parser.add_argument("--title", help="core-need: what core needs")
    parser.add_argument("--why", default="", help="core-need: which mod feature needs it, and why it is shared")
    args = parser.parse_args()
    if not available():
        print("Beads is not set up: run ./setup.sh (it installs bd and runs bd init).", file=sys.stderr)
        return 1
    try:
        if args.command in ("start", "features", "gaps", "contract", "tickets", "tickets-check", "ready",
                            "core-need") and not args.mod:
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
        elif args.command == "contract":
            print(contract(args.mod))
        elif args.command in ("tickets", "tickets-check"):
            print(tickets(args.mod, check_only=args.command == "tickets-check"))
        elif args.command == "ready":
            rows = ready(args.mod)
            for i in rows:
                print(f"{i['id']:14} {label_value(i, 'role') or '?':20} {i['title']}")
            print(f"{len(rows)} ticket(s) ready for {args.mod}: start one subagent for each (its role), "
                  "`make fleet-task BEAD=<id> ROLE=<role>` first." if rows else f"No ticket of {args.mod} is ready.")
        elif args.command == "inbox":
            print(inbox())
        elif args.command == "board":
            print(board())
        elif args.command == "core-need":
            if not args.title:
                raise BeadsError("core-need needs TITLE=\"<what core needs>\"")
            print(core_need(args.mod, args.title, args.why))
        elif args.command == "core-sync":
            print(core_sync())
        elif args.command == "seed":
            print(seed(Path(args.mod) if args.mod else BOARD))
    except BeadsError as exc:
        print(f"[beads] {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
