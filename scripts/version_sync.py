#!/usr/bin/env python3
"""Keep each mod's (and rekkit_core's) version one step ahead of its last commit while it has work.

For every mod under mods/ (but the example_mod template) and crates/rekkit_core:
  - files changed since the last commit (HEAD), version still the committed one: bump the patch
    version once (0.1.0 -> 0.1.1), in lockstep: Cargo.toml, content/horizon-mod.toml,
    native/horizon-native.json and native/mixins/<id>/horizon-mixin.toml (the core: Cargo.toml);
  - already bumped since the last commit: nothing; all uncommitted work ships under that one version;
  - nothing changed, or new since the last commit (it starts at its own version): nothing.
Lockstep files that disagree are set to Cargo.toml's version.

The stop gate, `make fleet-inbox` and `make agent-gate` run it; agents never bump versions by hand.

Usage:
  make version-sync [MOD=<id>]
  python3 scripts/version_sync.py [--mod <id>] [--check]
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP = {"example_mod"}
TOML_VERSION = re.compile(r'^(version\s*=\s*")([^"]+)(")', re.M)
JSON_VERSION = re.compile(r'("version"\s*:\s*")([^"]+)(")')


def git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True)


def units(only: str | None = None) -> list[Path]:
    found = [p.parent for p in sorted((ROOT / "mods").glob("*/Cargo.toml")) if p.parent.name not in SKIP]
    found.append(ROOT / "crates" / "rekkit_core")
    return [u for u in found if only in (None, u.name) and (u / "Cargo.toml").is_file()]


def files(unit: Path) -> list[Path]:
    if unit.parent.name == "crates":
        return [unit / "Cargo.toml"]
    name = unit.name
    return [unit / "Cargo.toml", unit / "content" / "horizon-mod.toml", unit / "native" / "horizon-native.json",
            unit / "native" / "mixins" / name / "horizon-mixin.toml"]


def key(version: str) -> tuple[int, ...]:
    return tuple(int(n) for n in re.findall(r"\d+", version)[:3])


def cargo_version(text: str) -> str:
    return str(tomllib.loads(text).get("package", {}).get("version", ""))


def changed(unit: Path) -> bool:
    rel = unit.relative_to(ROOT).as_posix()
    return bool(git("status", "--porcelain", "--", rel).stdout.strip())


def committed_version(unit: Path) -> str | None:
    r = git("show", f"HEAD:{(unit / 'Cargo.toml').relative_to(ROOT).as_posix()}")
    return cargo_version(r.stdout) if r.returncode == 0 else None


def set_version(path: Path, version: str) -> bool:
    if not path.is_file():
        return False
    text = path.read_text(encoding="utf-8")
    pattern = JSON_VERSION if path.suffix == ".json" else TOML_VERSION
    new = pattern.sub(lambda m: m.group(1) + version + m.group(3), text, count=1)
    if new != text:
        path.write_text(new, encoding="utf-8")
        return True
    return False


def file_version(path: Path) -> str | None:
    if not path.is_file():
        return None
    m = (JSON_VERSION if path.suffix == ".json" else TOML_VERSION).search(path.read_text(encoding="utf-8"))
    return m.group(2) if m else None


def sync(unit: Path, check: bool) -> str | None:
    """What was (or, with check, would be) done for the unit, or None."""
    cargo = unit / "Cargo.toml"
    current = cargo_version(cargo.read_text(encoding="utf-8"))
    base = committed_version(unit)
    target = current
    note = None
    if base and changed(unit) and key(current) <= key(base):
        parts = list(key(base)) + [0, 0, 0]
        target = f"{parts[0]}.{parts[1]}.{parts[2] + 1}"
        note = f"{unit.name}: {base} -> {target} (changed since the last commit)"
    off = [f for f in files(unit) if file_version(f) not in (None, target)]
    if off and not note:
        note = f"{unit.name}: lockstep {target} in " + ", ".join(f.relative_to(unit).as_posix() for f in off)
    if note and not check:
        for f in files(unit):
            set_version(f, target)
    return note


def sync_units(names: list[str] | None = None, check: bool = False) -> list[str]:
    """For the given mod ids / crate names (or every unit); used by the guard and tracker.py too."""
    picked = [u for u in units() if names is None or u.name in names]
    return [n for u in picked if (n := sync(u, check))]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mod", help="Only this mod (or rekkit_core)")
    parser.add_argument("--check", action="store_true", help="Report what would change; exit 1 if anything would")
    args = parser.parse_args()
    notes = sync_units([args.mod] if args.mod else None, args.check)
    for n in notes:
        print(("would bump " if args.check else "") + n)
    if not notes:
        print("Versions in step: every changed mod is one version ahead of its last commit.")
    return 1 if args.check and notes else 0


if __name__ == "__main__":
    raise SystemExit(main())
