#!/usr/bin/env python3
"""Split a mod's decompiled code into parts one spec-writer can read in a ~200k-token context.

Step 1 runs one spec-writer per part, all at once. Each reads its part's Java (in pieces: java-outline,
java-show) and writes that part's tickets, planning/mods/<id>.tickets/<part>.json. A part is files of
one decompiled archive, in path order (a package stays together where it fits), packed up to
PART_LINES lines of Java: reading costs about 12 tokens a line, and the writer needs room left to
think and write. Each part's file list is saved in target/fleet/parts/<id>/<part>.txt.

Usage:
  make fleet-parts MOD=<id>
  python3 scripts/fleet/parts.py <id>
"""

from __future__ import annotations

import sys
from pathlib import Path

import context

ROOT = Path(__file__).resolve().parents[2]
PART_LINES = 9000  # ~110k tokens of Java read in full; most parts are skimmed, so this is conservative


def java_lines(path: Path) -> int:
    try:
        return path.read_text(encoding="utf-8", errors="replace").count("\n") + 1
    except OSError:
        return 0


def parts(mod: str) -> list[dict]:
    """Consecutive files (by path, so a package stays together where it fits) of each decompiled
    archive, packed up to PART_LINES lines."""
    spec_path = ROOT / "planning" / "mods" / f"{mod}.md"
    spec = spec_path.read_text(encoding="utf-8", errors="replace") if spec_path.is_file() else ""
    out: list[dict] = []
    for _, _, _, archive in context.archives(mod, spec):
        dec = ROOT / "assets" / "decompiled" / Path(archive).stem
        if not dec.is_dir():
            continue
        current: dict | None = None
        for java in sorted(dec.rglob("*.java")):
            size = java_lines(java)
            if current is None or (current["lines"] + size > PART_LINES and current["files"]):
                current = {"archive": dec.relative_to(ROOT).as_posix(), "files": [], "lines": 0}
                out.append(current)
            current["files"].append(java.relative_to(dec).as_posix())
            current["lines"] += size
    listing = ROOT / "target" / "fleet" / "parts" / mod
    listing.mkdir(parents=True, exist_ok=True)
    for n, part in enumerate(out, 1):
        part["name"] = f"part{n:02d}"
        part["list"] = listing / f"{part['name']}.txt"
        part["list"].write_text("\n".join(f"{part['archive']}/{f}" for f in part["files"]) + "\n", encoding="utf-8")
        part["packages"] = sorted({str(Path(f).parent) for f in part["files"]})
    return out


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: make fleet-parts MOD=<id>", file=sys.stderr)
        return 1
    mod = sys.argv[1]
    found = parts(mod)
    if not found:
        print(f"No decompiled code for {mod}: make decompile JAR=<archive> for each archive "
              f"(make fleet-context MOD={mod} lists them).")
        return 1
    for p in found:
        print(f"{p['name']}: {p['archive']} ({p['lines']:,} lines, {len(p['files'])} files in "
              f"{', '.join(p['packages'])}): {p['list'].relative_to(ROOT).as_posix()}")
    print(f"\n{len(found)} part(s): start one spec-writer per part, all at once. Each writes "
          f"planning/mods/{mod}.tickets/<part>.json; then `make fleet-tickets MOD={mod}` checks and files them.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
