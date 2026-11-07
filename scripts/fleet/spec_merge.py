#!/usr/bin/env python3
"""Merge the spec-writers' part files into the mod's spec and outline (step 1, fix-plan B4).

Each spec-writer reads its part of the decompiled code once and writes, in its own files (so parallel
writers never edit the same one):
  planning/mods/<id>.parts/<part>.md              the outline of what its part holds (### headings)
  planning/mods/<id>.parts/<part>.features.json   {"blocks": [...], "items": [...], "features": [...]}:
                                                  each entry's id and attributes (and any field it found)
  planning/mods/<id>.tickets/<part>.json          its tickets
This puts every part's outline under "## Part details" at the end of planning/mods/<id>.md (between
markers, replaced on each run) and merges the entries into planning/mods/<id>.features.json by id:
attributes are unioned, a new id is added, a field the outline lacks is filled in. `make fleet-tickets`
runs it first.

Usage:
  make spec-merge MOD=<id>
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BEGIN, END = "<!-- parts:begin (make spec-merge writes this; edit the part files) -->", "<!-- parts:end -->"


def merge(mod: str) -> str:
    plan = ROOT / "planning" / "mods"
    parts = plan / f"{mod}.parts"
    if not parts.is_dir():
        return f"No part files in {parts.relative_to(ROOT).as_posix()}/: nothing to merge."
    notes = []
    md_parts = sorted(parts.glob("*.md"))
    spec = plan / f"{mod}.md"
    if md_parts:
        text = spec.read_text(encoding="utf-8") if spec.is_file() else f"# {mod}\n"
        if BEGIN in text:
            text = text[:text.index(BEGIN)].rstrip() + "\n"
        body = "\n\n".join(f"### {p.stem}\n\n" + p.read_text(encoding="utf-8").strip() for p in md_parts)
        spec.write_text(f"{text.rstrip()}\n\n{BEGIN}\n## Part details\n\n{body}\n{END}\n", encoding="utf-8", newline="\n")
        notes.append(f"{len(md_parts)} part outline(s) into {spec.relative_to(ROOT).as_posix()}")
    feats = plan / f"{mod}.features.json"
    try:
        outline = json.loads(feats.read_text(encoding="utf-8")) if feats.is_file() else {}
    except ValueError as exc:
        return f"{feats.relative_to(ROOT).as_posix()} is not JSON ({exc}); fix it first."
    added = updated = 0
    for f in sorted(parts.glob("*.features.json")):
        try:
            part = json.loads(f.read_text(encoding="utf-8"))
        except ValueError as exc:
            notes.append(f"{f.name}: not JSON ({exc}), skipped")
            continue
        for kind in ("blocks", "items", "features"):
            have = {e.get("id"): e for e in outline.setdefault(kind, []) if isinstance(e, dict)}
            for e in part.get(kind, []):
                if not isinstance(e, dict) or not e.get("id"):
                    continue
                old = have.get(e["id"])
                if old is None:
                    outline[kind].append(e)
                    have[e["id"]] = e
                    added += 1
                    continue
                attrs = list(old.get("attributes", []))
                attrs += [a for a in e.get("attributes", []) if a not in attrs]
                if attrs != old.get("attributes", []):
                    old["attributes"] = attrs
                    updated += 1
                for k, v in e.items():
                    old.setdefault(k, v)
    if added or updated:
        feats.write_text(json.dumps(outline, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    notes.append(f"{added} new and {updated} updated entries in {feats.relative_to(ROOT).as_posix()}")
    return "Merged: " + "; ".join(notes) + "."


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__.strip().splitlines()[-1].strip(), file=sys.stderr)
        return 1
    print(merge(sys.argv[1]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
