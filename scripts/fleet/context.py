#!/usr/bin/env python3
"""Everything an agent needs to find for one mod, in one short page, so it need not explore with
ls, find or by reading the big docs: its state in Beads, its spec, every original archive of it in
every pack (the port merges them, newest version wins), what is decompiled, and what the mod folder
already holds.

Usage:
  make fleet-context MOD=<id>
  python3 scripts/fleet/context.py <id>
"""

from __future__ import annotations

import json
import re
import sys
import zipfile
from pathlib import Path

import beads

ROOT = beads.main_checkout()
PACKS = ("tekkit", "direwolf20", "agrarian")
# Archive names that do not contain the mod id: regexes on the name lowercased without punctuation.
ALIASES = {
    "industrialcraft_2": ["^ic2(?!nuclear)", "industrialcraft"], "ic2_nuclear_control": ["nuclearcontrol"],
    "equivalent_exchange_2": ["^ee2"], "equivalent_exchange_3": ["^ee3"], "gravitation_suite": ["gravisuite"],
    "wireless_redstone_cbe": ["wrcbe", "wirelessredstone"], "obsidian_pressure_plates": ["obsidiplates"],
    "xenos_reliquary": ["xreliquary"], "applied_energistics": ["appeng"], "balkons_weaponmod": ["weaponmod"],
    "dimensional_anchors": ["dimanchor", "dimensionalanchor"], "iron_chests": ["ironchest"],
    "compact_solars": ["compactsolar"], "mffs": ["mffs", "modularforcefield"], "steves_carts_2": ["stevescarts"],
    "thaumcraft_3": ["thaumcraft"], "extrabiomes_xl": ["extrabiomes"], "charge_pads": ["chargepad"],
    "nether_ores": ["netherores"], "soul_shards": ["soulshards"], "misc_peripherals": ["miscperipherals"],
}


def norm(text: str) -> str:
    return re.sub(r"[^a-z0-9]", "", text.lower())


def keys(mod: str, spec: str) -> tuple[set[str], set[str]]:
    """Patterns that identify the mod's archive names, and archive names the spec mentions."""
    k = norm(mod)
    # An alias list replaces the guesses from the id (ic2 is short, but exact).
    frags = set(ALIASES[mod]) if mod in ALIASES else {re.escape(f) for f in (k, k.rstrip("0123456789")) if len(f) >= 4}
    named = {norm(Path(n).name) for n in re.findall(r"[\w.\-\[\] @]+\.(?:jar|zip)", spec)}
    return frags, named


def archives(mod: str, spec: str) -> list[tuple[str, str, int, str]]:
    """(pack, side, art files, archive path) for each archive of this mod."""
    frags, named = keys(mod, spec)
    out = []
    for pack in PACKS:
        info = ROOT / "assets" / pack / "pack-info.json"
        if not info.is_file():
            continue
        for rec in json.loads(info.read_text(encoding="utf-8")).get("mods", []):
            name = norm(Path(rec["archive"]).name)
            if any(re.search(f, name) for f in frags) or name in named:
                side = rec.get("side") or ("server" if pack == "tekkit" else "client")
                path = ROOT / "assets" / pack / rec["archive"]
                art = rec["art_files"] if "art_files" in rec else art_files(path)
                out.append((pack, side, art, f"assets/{pack}/{rec['archive']}"))
    return out


def art_files(path: Path) -> int:
    """Textures and sounds in an archive (for a pack-info.json older than its art_files count)."""
    try:
        with zipfile.ZipFile(path) as z:
            return sum(n.lower().endswith((".png", ".ogg", ".wav")) for n in z.namelist())
    except (OSError, zipfile.BadZipFile):
        return 0


def count(path: Path, pattern: str) -> int:
    return sum(1 for _ in path.rglob(pattern)) if path.is_dir() else 0


def lines(path: Path) -> str:
    return f"{len(path.read_text(encoding='utf-8', errors='replace').splitlines())} lines" if path.is_file() else "missing"


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__.strip().splitlines()[-2].strip(), file=sys.stderr)
        return 1
    mod = sys.argv[1]
    spec_path = ROOT / "planning" / "mods" / f"{mod}.md"
    spec = spec_path.read_text(encoding="utf-8", errors="replace") if spec_path.is_file() else ""

    try:
        st = beads.mod_status(mod)
        e = st["epic"]
        print(f"{mod}: epic {e['id']} {e['status']}; next: {st['next']}")
        open_stories = [f"{s['id']} {n} {beads.STEPS[n]} ({s['status']}, {st['counts'][n][0]}/{st['counts'][n][1]})"
                        for n, s in sorted(st["stories"].items()) if s["status"] != "closed"]
        if open_stories:
            print("  open steps: " + "; ".join(open_stories))
    except beads.BeadsError as exc:
        print(f"{mod}: not in Beads ({exc})")

    feats = ROOT / "planning" / "mods" / f"{mod}.features.json"
    print(f"Spec: {spec_path.relative_to(ROOT)} ({lines(spec_path)}); features: {feats.relative_to(ROOT)} "
          f"({'%d entries' % len(json.loads(feats.read_text(encoding='utf-8'))) if feats.is_file() else 'missing'})")

    found = archives(mod, spec)
    print("Original archives (merge every version; where versions share a feature the newest wins):"
          if found else "Original archives: none matched by name; look up the spec's Overview, or ask the scout.")
    for pack, side, art, path in found:
        dec = ROOT / "assets" / "decompiled" / Path(path).stem  # make decompile's output folder
        java = count(dec, "*.java")
        print(f"  {pack:10} {side:6} {art:4} art  {path}"
              + (f"\n{'':25}decompiled: {dec.relative_to(ROOT)} ({java} classes)" if java else ""))

    mod_dir = ROOT / ("crates" if mod == "rekkit_core" else "mods") / mod
    if mod_dir.is_dir():
        content = mod_dir / "content" / "data" / mod / "horizon"
        textures = mod_dir / "content" / "assets" / mod / "textures"
        amap = mod_dir / "asset-map.json"
        print(f"Mod folder: {mod_dir.relative_to(ROOT)}: src {count(mod_dir / 'src', '*.rs')} .rs, "
              f"tests {count(mod_dir / 'tests', '*.rs')}, blocks {count(content / 'block', '*.json')}, "
              f"items {count(content / 'item', '*.json')}, recipes {count(mod_dir / 'content' / 'data' / mod / 'recipe', '*.json')}, "
              f"textures {count(textures, '*.png')}, mixin .rs {count(mod_dir / 'native', '*.rs')}, "
              f"asset-map {'yes' if amap.is_file() else 'missing'}, art/ {count(mod_dir / 'art', '*.*')} files")
    else:
        print(f"Mod folder: {mod_dir.relative_to(ROOT)} does not exist yet (step 2 Scaffold).")
    print("Read Java in pieces: make java-outline DIR=<decompiled> [GREP=<name>], then make java-show "
          "DIR=<decompiled> MEMBER=<Class#method>.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
