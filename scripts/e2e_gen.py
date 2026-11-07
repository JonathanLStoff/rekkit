#!/usr/bin/env python3
"""Draft a mod's tests/e2e_actions.json from its contract, so the test-smith only checks and adds to it.

From the merged contract (planning/mods/<id>.features.json + planning/mods/<id>.contract/*.json):
  trigger_items         every item with a `use` or a durability
  custom_recipe_types   every "<id>:..." recipe type under content/data/<id>/recipe/
  stage_3_interactions  each scenario step that uses an item on a block: the item and the blocks it is used on
  stage_5_neighbors     each block with a level-6 (automation & sides) attribute: a hopper above, a chest below

An existing file keeps everything in it; the draft only adds what it lacks (marked "draft": true on new
interactions and neighbours, for the test-smith to check).

Usage:
  make e2e-gen MOD=<id> [ARGS=--check]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from contract_check import attribute, load_contract  # noqa: E402


def draft(mod: str) -> dict:
    contract = load_contract(mod) or {}
    items = [e for e in contract.get("items", []) if isinstance(e, dict)]
    blocks = [e for e in contract.get("blocks", []) if isinstance(e, dict)]
    triggers = [f"{mod}:{e['id']}" for e in items if e.get("use") or isinstance(e.get("durability"), int)]
    types = set()
    for f in (ROOT / "mods" / mod / "content" / "data" / mod / "recipe").glob("*.json"):
        try:
            kind = json.loads(f.read_text(encoding="utf-8")).get("type", "")
        except ValueError:
            continue
        if kind.startswith(f"{mod}:"):
            types.add(kind)
    uses: dict[str, set[str]] = {}
    for sc in contract.get("scenarios", []):
        placed = [s["setblock"][3] for s in sc.get("steps", []) if isinstance(s.get("setblock"), list) and len(s["setblock"]) > 3]
        for s in sc.get("steps", []):
            if s.get("use") and placed:
                uses.setdefault(s["use"], set()).update(placed)
    sided = [f"{mod}:{e['id']}" for e in blocks
             if any(attribute(a)[1] == 6 for a in e.get("attributes", []))]
    return {"mod": mod, "trigger_items": triggers, "custom_recipe_types": sorted(types),
            "stage_3_interactions": [{"item": i, "target_blocks": sorted(b), "action": "use", "draft": True}
                                     for i, b in sorted(uses.items())],
            "stage_5_neighbors": [{"center": c, "top": "minecraft:hopper", "bottom": "minecraft:chest",
                                   "purpose": "sided_automation", "draft": True} for c in sided]}


def merge(old: dict, new: dict) -> dict:
    out = dict(old) if old else {"mod": new["mod"]}
    for key in ("trigger_items", "custom_recipe_types"):
        have = list(out.get(key, []))
        out[key] = have + [x for x in new[key] if x not in have]
    have_items = {i.get("item") for i in out.get("stage_3_interactions", [])}
    out["stage_3_interactions"] = list(out.get("stage_3_interactions", [])) + [
        i for i in new["stage_3_interactions"] if i["item"] not in have_items]
    have_centers = {n.get("center") for n in out.get("stage_5_neighbors", [])}
    out["stage_5_neighbors"] = list(out.get("stage_5_neighbors", [])) + [
        n for n in new["stage_5_neighbors"] if n["center"] not in have_centers]
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mod", required=True)
    parser.add_argument("--check", action="store_true", help="Say what it would add; write nothing")
    args = parser.parse_args()
    path = ROOT / "mods" / args.mod / "tests" / "e2e_actions.json"
    try:
        old = json.loads(path.read_text(encoding="utf-8")) if path.is_file() else {}
    except ValueError:
        print(f"{path.relative_to(ROOT)} is not JSON; fix it first.", file=sys.stderr)
        return 1
    merged = merge(old, draft(args.mod))
    added = {k: len(merged.get(k, [])) - len(old.get(k, [])) for k in
             ("trigger_items", "custom_recipe_types", "stage_3_interactions", "stage_5_neighbors")}
    summary = ", ".join(f"{n} {k}" for k, n in added.items() if n) or "nothing new"
    if merged != old and not args.check:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(merged, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"{'Would add' if args.check else 'Added'} to {path.relative_to(ROOT).as_posix()}: {summary}."
          + (" Check every entry marked \"draft\": true, then remove the mark." if any(added.values()) else ""))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
