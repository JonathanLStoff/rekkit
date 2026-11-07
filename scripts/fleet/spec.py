#!/usr/bin/env python3
"""A mod's spec, a skill or the conductor's steps a section at a time: read the sections you need,
not the whole file (every line read is paid again on every later turn).

  toc         every heading of planning/mods/<id>.md with its line range and size
  show        the sections whose heading contains the text given (each with its subsections)
  skill-toc   the same for .agents/skills/<name>/SKILL.md
  skill-show
  step        the conductor's instructions for one step (.agents/agents/conductor/steps.md)

Usage:
  make spec-toc MOD=<id>
  make spec-show MOD=<id> SECTION="Alloy Furnace"
  make skill-toc SKILL=mod-spec
  make skill-show SKILL=mod-spec SECTION="Test levels"
  make conductor-step STEP=3
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


def target(kind: str, name: str) -> Path:
    if kind in ("skill-toc", "skill-show"):
        return ROOT / ".agents" / "skills" / name / "SKILL.md"
    if kind == "step":
        return ROOT / ".agents" / "agents" / "conductor" / "steps.md"
    return ROOT / "planning" / "mods" / f"{name}.md"


def main() -> int:
    kinds = ("toc", "show", "skill-toc", "skill-show", "step")
    if len(sys.argv) < 3 or sys.argv[1] not in kinds or (sys.argv[1] in ("show", "skill-show") and len(sys.argv) < 4):
        print("\n".join(ln.strip() for ln in __doc__.strip().splitlines() if ln.strip().startswith("make ")),
              file=sys.stderr)
        return 1
    kind, name = sys.argv[1], sys.argv[2]
    path = target(kind, name)
    if not path.is_file():
        print(f"No file at {path.relative_to(ROOT)}.", file=sys.stderr)
        return 1
    lines = path.read_text(encoding="utf-8").splitlines()
    found = sections(lines)
    if kind in ("toc", "skill-toc"):
        for level, title, start, end in found:
            print(f"{'  ' * (level - 1)}{title}  [{start}-{end}, {end - start + 1} lines]")
        return 0
    want = (f"step {name} " if kind == "step" else sys.argv[3]).lower()
    hits = [s for s in found if want in (s[1].lower() + " ")]
    # A hit inside another hit is already printed with its parent.
    hits = [h for h in hits if not any(o is not h and o[2] <= h[2] and h[3] <= o[3] for o in hits)]
    if not hits:
        toc = {"show": f"make spec-toc MOD={name}", "skill-show": f"make skill-toc SKILL={name}",
               "step": "the steps are 0 to 10"}[kind]
        print(f"No heading contains {want.strip()!r}; {toc}.", file=sys.stderr)
        return 1
    for _, _, start, end in hits:
        print(f"<!-- {path.relative_to(ROOT).as_posix()}:{start}-{end} -->")
        print("\n".join(lines[start - 1:end]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
