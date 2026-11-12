#!/usr/bin/env python3
"""Which mods need a release, and whether each one's version was bumped.

Each mod is released as the GitHub release (and tag) `<id>-<version>`, the version being the one in
mods/<id>/content/horizon-mod.toml (pack_mod.py checks the other three files carry the same one).
For every mod under mods/ (but --exclude):

  - no tag for its version, and the version is newer than every earlier release: release it;
  - a tag for its version, and nothing it ships changed since: nothing to do;
  - a tag for its version, but what it ships (content/, native/, src/) changed since: error, the
    version must be bumped;
  - no tag for its version, but an earlier release has a newer or equal version: error.

A mod is considered only once the port process has packaged it: its step 9 Package story is closed
in the tracker (read from the committed issues/). A mod still in progress is neither
released nor held to the version check.

A mod that still ships any of the original mod's art or sound (asset-map.json origin "original", or
a PNG/OGG the map does not account for) is released as a GitHub pre-release, which Horizon's registry
skips; only a mod whose every asset is its own gets a full release (.agents/release.md).

Prints a summary; in GitHub Actions also writes `matrix` (JSON list of {id, version, tag,
previous_tag, prerelease, originals}) and `count` to $GITHUB_OUTPUT. Exit 1 on any error.

Usage:
  python3 scripts/release_plan.py --exclude example_mod
  python3 scripts/release_plan.py --mod buildcraft
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODS_ROOT = ROOT / "mods"
# What a mod's .hmod and .hnmod are built from (pack_mod.py); other files (tests, README, spec)
# change without a release.
SHIPPED = ("content", "native", "src")
VERSION_RE = re.compile(r"^(\d+(?:\.\d+)*)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$")


def version_key(version: str) -> tuple | None:
    """A sortable key for a SemVer-like version (1.2.0 > 1.2.0-rc.1 > 1.1.9); None if it isn't one."""
    m = VERSION_RE.match(version)
    if not m:
        return None
    core = tuple(int(p) for p in m.group(1).split("."))
    core += (0,) * (4 - len(core))
    pre = m.group(2)
    if pre is None:
        return core, 1, ()
    return core, 0, tuple((0, int(p), "") if p.isdigit() else (1, 0, p) for p in pre.split("."))


def git(*args: str) -> str:
    return subprocess.run(["git", "-C", str(ROOT), *args], check=True, capture_output=True, text=True).stdout


def released_versions(mod_id: str, tags: list[str]) -> dict[str, str]:
    """{version: tag} for this mod's release tags (`<id>-<version>`)."""
    out = {}
    for tag in tags:
        if tag.startswith(f"{mod_id}-"):
            version = tag[len(mod_id) + 1:]
            if version_key(version):
                out[version] = tag
    return out


def original_assets(mod_dir: Path) -> list[str]:
    """The mod's PNG/OGG files that are still the original mod's (or of unknown origin)."""
    try:
        amap = json.loads((mod_dir / "asset-map.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        amap = {}
    own = {e["out"] for key in ("slices", "copies", "generated") for e in amap.get(key, [])
           if e.get("out") and e.get("origin", "original" if key != "generated" else "generated") != "original"}
    assets = mod_dir / "content" / "assets"
    files = sorted(p for p in assets.rglob("*") if p.suffix in (".png", ".ogg", ".wav")) if assets.is_dir() else []
    return [p.relative_to(mod_dir).as_posix() for p in files if p.relative_to(mod_dir).as_posix() not in own]


def shipped_changed(mod_id: str, tag: str) -> bool:
    paths = [f"mods/{mod_id}/{p}" for p in SHIPPED]
    return subprocess.run(["git", "-C", str(ROOT), "diff", "--quiet", tag, "HEAD", "--", *paths]).returncode != 0


CLOSING_STATUSES = {"done", "fixed", "wontfix", "duplicate", "cannot-reproduce", "obsolete"}


def frontmatter(path: Path) -> dict:
    """The frontmatter of an issues/<slug>/item.md: `key: value` lines and `- item` lists. Read directly (no
    issuectl needed), so CI can run this."""
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---"):
        return {}
    out: dict = {}
    key = None
    for line in text.split("\n---", 1)[0].split("\n")[1:]:
        if line.startswith("- ") and key is not None and isinstance(out.get(key), list):
            out[key].append(line[2:].strip().strip("\"'"))
        elif ":" in line and not line.startswith(" "):
            k, _, v = line.partition(":")
            key, v = k.strip(), v.strip()
            out[key] = [] if v == "" else v.strip("\"'")
    return out


def packaged_mods() -> set[str] | None:
    """Mods whose step 9 Package story is closed in the tracker (issues/); None without an issues/ folder."""
    root = ROOT / "issues"
    if not root.is_dir():
        return None
    done = set()
    for item in root.rglob("item.md"):
        fm = frontmatter(item)
        labels = fm.get("labels") or []
        if "story" in labels and "step:9" in labels and fm.get("status") in CLOSING_STATUSES:
            done |= {l.split(":", 1)[1] for l in labels if l.startswith("mod:")}
    return done


def plan(mod_dirs: list[Path], tags: list[str], packaged: set[str] | None = None) -> tuple[list[dict], list[str]]:
    releases, errors = [], []
    for mod_dir in mod_dirs:
        mod_id = mod_dir.name
        if packaged is not None and mod_id not in packaged:
            print(f"  ~ {mod_id}: in progress (step 9 Package not closed in the tracker); not released")
            continue
        manifest = mod_dir / "content" / "horizon-mod.toml"
        if not manifest.is_file():
            errors.append(f"{mod_id}: no content/horizon-mod.toml")
            continue
        version = str(tomllib.loads(manifest.read_text(encoding="utf-8")).get("version", ""))
        key = version_key(version)
        if not key:
            errors.append(f"{mod_id}: version {version!r} in content/horizon-mod.toml is not like 1.2.3")
            continue
        done = released_versions(mod_id, tags)
        newest = max(done, key=version_key) if done else None
        tag = f"{mod_id}-{version}"
        if version in done:
            if shipped_changed(mod_id, tag):
                errors.append(
                    f"{mod_id}: content/, native/ or src/ changed since release {tag}, but the version is still "
                    f"{version}. Bump it in Cargo.toml, content/horizon-mod.toml, native/horizon-native.json and "
                    f"native/mixins/{mod_id}/horizon-mixin.toml.")
            else:
                print(f"  = {mod_id} {version}: already released, unchanged")
            continue
        if newest and version_key(newest) >= key:
            errors.append(f"{mod_id}: version {version} is not newer than the released {newest} ({done[newest]})")
            continue
        originals = original_assets(mod_dir)
        print(f"  + {mod_id} {version}: to " + ("pre-release" if originals else "release")
              + (f" (after {newest})" if newest else " (first release)")
              + (f"; {len(originals)} original asset(s)" if originals else ""))
        releases.append({"id": mod_id, "version": version, "tag": tag, "previous_tag": done[newest] if newest else "",
                         "prerelease": bool(originals), "originals": len(originals)})
    return releases, errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--exclude", action="append", default=[], help="Mod id to skip (repeatable)")
    parser.add_argument("--mod", action="append", help="Only these mod ids (repeatable; default: every mod)")
    args = parser.parse_args()

    mod_dirs = sorted(p for p in MODS_ROOT.iterdir()
                      if (p / "Cargo.toml").is_file() and p.name not in args.exclude
                      and (not args.mod or p.name in args.mod))
    tags = git("tag", "--list").split()
    print(f"Release plan for {len(mod_dirs)} mod(s):")
    releases, errors = plan(mod_dirs, tags, packaged_mods())
    for e in errors:
        print(f"::error::{e}" if os.environ.get("GITHUB_ACTIONS") else f"  ✗ {e}")
    if out := os.environ.get("GITHUB_OUTPUT"):
        with open(out, "a", encoding="utf-8") as fh:
            fh.write(f"matrix={json.dumps(releases)}\ncount={len(releases)}\n")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
