#!/usr/bin/env python3
"""A mod's spec a section at a time: builders read the sections their bead needs, not the whole spec.

  toc   every heading of planning/mods/<id>.md with its line range and size
  show  the sections whose heading contains the text given (each with its subsections)

Usage:
  make spec-toc MOD=<id>
  make spec-show MOD=<id> SECTION="Alloy Furnace"
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HEADING = re.compile(r"^(#{1,6})\s+(.*)")


def sections(lines: list[str]) -> list[tuple[int, str, int, int]]:
    """(level, title, first line, last line) for every heading, 1-based, fenced code skipped."""
    heads, fenced = [], False
    for no, line in enumerate(lines, 1):
        if line.lstrip().startswith("```"):
            fenced = not fenced
        elif not fenced and (m := HEADING.match(line)):
            heads.append((len(m.group(1)), m.group(2).strip(), no))
    out = []
    for i, (level, title, start) in enumerate(heads):
        end = next((s - 1 for lv, _, s in heads[i + 1:] if lv <= level), len(lines))
        out.append((level, title, start, end))
    return out


def main() -> int:
    if len(sys.argv) < 3 or sys.argv[1] not in ("toc", "show") or (sys.argv[1] == "show" and len(sys.argv) < 4):
        print(__doc__.strip().splitlines()[-2].strip() + "\n" + __doc__.strip().splitlines()[-1].strip(), file=sys.stderr)
        return 1
    path = ROOT / "planning" / "mods" / f"{sys.argv[2]}.md"
    if not path.is_file():
        print(f"No spec at {path.relative_to(ROOT)}.", file=sys.stderr)
        return 1
    lines = path.read_text(encoding="utf-8").splitlines()
    found = sections(lines)
    if sys.argv[1] == "toc":
        for level, title, start, end in found:
            print(f"{'  ' * (level - 1)}{title}  [{start}-{end}, {end - start + 1} lines]")
        return 0
    want = sys.argv[3].lower()
    hits = [s for s in found if want in s[1].lower()]
    # A hit inside another hit is already printed with its parent.
    hits = [h for h in hits if not any(o is not h and o[2] <= h[2] and h[3] <= o[3] for o in hits)]
    if not hits:
        print(f"No heading contains {sys.argv[3]!r}; `make spec-toc MOD={sys.argv[2]}` lists them.", file=sys.stderr)
        return 1
    for _, _, start, end in hits:
        print(f"<!-- {path.relative_to(ROOT).as_posix()}:{start}-{end} -->")
        print("\n".join(lines[start - 1:end]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
