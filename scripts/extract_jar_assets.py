#!/usr/bin/env python3
"""Extract asset files (textures, audio, models, language, scripts) from Minecraft mod JARs/ZIPs.

Handles both legacy (pre-1.6 Minecraft like 1.2.5 / 1.4.7) and modern (1.6+) mod archives.
Pre-1.6 mods often store textures, sounds, and ROM files in root/subfolders without an assets/ prefix,
and extensionless files like ComputerCraft's lua/rom/apis/*.
Standard library only.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import sys
import zipfile
from collections import defaultdict
from pathlib import Path, PurePosixPath
from typing import Any

MEDIA_EXTENSIONS = {
    "png", "jpg", "jpeg", "tga", "bmp", "gif", "ico",
    "ogg", "wav", "mus", "mp3", "mid", "midi"
}

ASSET_EXTENSIONS = MEDIA_EXTENSIONS | {
    "lang", "properties", "mcmeta", "info", "cfg", "conf",
    "json", "txt", "csv", "xml", "lua", "obj", "mtl",
    "bbmodel", "vsh", "fsh", "glsl", "bin", "img"
}

CODE_AND_META_EXTENSIONS = {
    "class", "java", "sf", "dsa", "rsa", "mf"
}


def sanitize_filename(name: str) -> str:
    """Create a safe folder name from a JAR filename."""
    stem = Path(name).stem
    return re.sub(r"[^A-Za-z0-9._-]+", "_", stem).strip("._-") or "mod"


def categorize_file(path_str: str) -> str:
    """Categorize an asset file for reporting."""
    posix = PurePosixPath(path_str.replace("\\", "/"))
    ext = posix.suffix.lower().lstrip(".")

    if ext in {"png", "jpg", "jpeg", "tga", "bmp", "gif", "ico"}:
        return "Textures"
    if ext in {"ogg", "wav", "mus", "mp3", "mid", "midi"}:
        return "Audio"
    if ext in {"obj", "mtl", "bbmodel"} or (ext == "json" and "models" in posix.parts):
        return "Models"
    if ext in {"lang", "properties"} or (ext == "json" and "lang" in posix.parts):
        return "Localization"
    if ext == "lua" or "rom" in posix.parts or "lua" in posix.parts:
        return "Scripts/ROM"
    if ext in {"cfg", "conf", "txt", "info", "mcmeta"}:
        return "Configs/Docs"
    return "Other Assets"


def is_asset(path_str: str, *, filter_exts: set[str] | None = None, only_media: bool = False) -> bool:
    """Determine if a zip entry represents an asset/resource file."""
    norm = path_str.replace("\\", "/")
    if norm.endswith("/"):
        return False

    parts = PurePosixPath(norm).parts
    if not parts:
        return False

    # Skip JAR metadata and OS artifacts
    if parts[0].upper() == "META-INF" or parts[-1] in {".DS_Store", "Thumbs.db"}:
        return False

    ext = PurePosixPath(norm).suffix.lower().lstrip(".")
    if ext in CODE_AND_META_EXTENSIONS:
        return False

    if filter_exts:
        return ext in filter_exts

    if only_media:
        return ext in MEDIA_EXTENSIONS

    # Any file inside assets/ or data/ is an asset (modern 1.6+)
    if parts[0].lower() in {"assets", "data"}:
        return True

    # Recognized asset extension
    if ext in ASSET_EXTENSIONS:
        return True

    # Extensionless or custom files in known asset/script directories (e.g. ComputerCraft lua/rom/apis/*)
    top = parts[0].lower()
    if top in {"lua", "rom", "gfx", "textures", "sound", "sounds", "armor", "gui", "terrain", "item", "mob", "font", "environment", "art", "misc", "lang"}:
        return True

    # If it has no extension but is not code/META, it is typically a ROM/help file or asset
    if not ext:
        return True

    return False


def is_safe_relative_path(base_dir: Path, target_path: Path) -> bool:
    """Prevent ZipSlip path traversal vulnerabilities."""
    try:
        resolved_base = base_dir.resolve()
        resolved_target = target_path.resolve()
        return str(resolved_target).startswith(str(resolved_base))
    except (ValueError, RuntimeError):
        return False


def extract_jar(
    jar_path: Path,
    output_dir: Path,
    *,
    filter_exts: set[str] | None = None,
    only_media: bool = False,
    list_only: bool = False,
    force: bool = False,
    verbose: bool = False,
) -> dict[str, Any]:
    """Extract assets from a single mod JAR or ZIP."""
    stats: dict[str, Any] = {
        "jar": jar_path.name,
        "total": 0,
        "categories": defaultdict(int),
        "files": [],
        "errors": [],
    }

    if not jar_path.is_file():
        stats["errors"].append(f"Not a valid file: {jar_path}")
        return stats

    try:
        with zipfile.ZipFile(jar_path) as zf:
            for member in zf.infolist():
                if not is_asset(member.filename, filter_exts=filter_exts, only_media=only_media):
                    continue

                category = categorize_file(member.filename)
                stats["total"] += 1
                stats["categories"][category] += 1
                stats["files"].append((member.filename, category, member.file_size))

                if list_only:
                    if verbose:
                        print(f"  [{category:12}] {member.filename} ({member.file_size:,} bytes)")
                    continue

                target = output_dir / member.filename
                if not is_safe_relative_path(output_dir, target):
                    stats["errors"].append(f"Unsafe path skipped: {member.filename}")
                    continue

                target.parent.mkdir(parents=True, exist_ok=True)
                if not target.exists() or force:
                    with zf.open(member) as src, target.open("wb") as dst:
                        shutil.copyfileobj(src, dst)
                    if verbose:
                        print(f"  Extracted: {member.filename} -> {target.relative_to(output_dir)}")
                elif verbose:
                    print(f"  Skipped (exists): {member.filename}")

    except zipfile.BadZipFile:
        stats["errors"].append(f"Corrupt or non-zip archive: {jar_path.name}")
    except Exception as exc:
        stats["errors"].append(f"Error processing {jar_path.name}: {exc}")

    return stats


def process_targets(
    targets: list[Path],
    base_output_dir: Path,
    *,
    filter_exts: set[str] | None = None,
    only_media: bool = False,
    list_only: bool = False,
    force: bool = False,
    verbose: bool = False,
) -> int:
    """Process files and directories of JARs."""
    jar_files: list[Path] = []
    for target in targets:
        if target.is_file() and target.suffix.lower() in {".jar", ".zip"}:
            jar_files.append(target)
        elif target.is_dir():
            for root, _, files in os.walk(target):
                for f in files:
                    if f.lower().endswith((".jar", ".zip")):
                        jar_files.append(Path(root) / f)

    if not jar_files:
        print("No .jar or .zip mod archives found to process.", file=sys.stderr)
        return 1

    total_extracted = 0
    total_categories: dict[str, int] = defaultdict(int)
    single_jar = len(jar_files) == 1

    print(f"Found {len(jar_files)} archive(s) to process. Output directory: {base_output_dir.resolve()}\n")

    for jar_path in sorted(jar_files):
        mod_name = sanitize_filename(jar_path.name)
        mod_output = base_output_dir if single_jar else base_output_dir / mod_name

        action = "Listing" if list_only else "Extracting"
        print(f"=== {action}: {jar_path.name} ===")

        stats = extract_jar(
            jar_path,
            mod_output,
            filter_exts=filter_exts,
            only_media=only_media,
            list_only=list_only,
            force=force,
            verbose=verbose,
        )

        for err in stats["errors"]:
            print(f"  ERROR: {err}", file=sys.stderr)

        cat_summary = ", ".join(f"{k}: {v}" for k, v in sorted(stats["categories"].items())) or "No assets"
        print(f"  -> Total: {stats['total']} asset file(s) ({cat_summary})\n")

        total_extracted += stats["total"]
        for k, v in stats["categories"].items():
            total_categories[k] += v

    print("================ Summary ================")
    print(f"Archives processed: {len(jar_files)}")
    print(f"Total asset files:  {total_extracted}")
    for k, v in sorted(total_categories.items()):
        print(f"  - {k:15}: {v:,}")
    print("=========================================")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Extract asset files (textures, audio, models, lang, scripts) from Minecraft mod JARs/ZIPs.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""Examples:
  # Extract assets from a single mod JAR
  python scripts/extract_jar_assets.py --jar assets/direwolf20/mods/forestry.jar

  # Extract assets from all mods in a directory
  python scripts/extract_jar_assets.py --dir assets/direwolf20/mods -o assets/extracted/direwolf20

  # List assets in an archive without extracting
  python scripts/extract_jar_assets.py --jar assets/direwolf20/mods/computercraft.zip --list --verbose

  # Extract only audio files
  python scripts/extract_jar_assets.py --jar assets/direwolf20/mods/railcraft.jar --ext ogg,wav
"""
    )
    parser.add_argument("targets", nargs="*", type=Path, help="Mod JAR/ZIP files or directories containing them")
    parser.add_argument("--jar", type=Path, help="Path to a single mod JAR or ZIP")
    parser.add_argument("--dir", type=Path, help="Path to a directory containing mod JAR/ZIPs")
    parser.add_argument("-o", "--output", type=Path, default=Path("./assets/extracted"), help="Destination directory (default: ./assets/extracted)")
    parser.add_argument("--ext", type=str, help="Comma-separated extensions to include (e.g. png,ogg,lang)")
    parser.add_argument("--only-media", action="store_true", help="Extract only images and audio")
    parser.add_argument("-l", "--list", action="store_true", help="List matching asset files without extracting")
    parser.add_argument("-f", "--force", action="store_true", help="Overwrite existing extracted files")
    parser.add_argument("-v", "--verbose", action="store_true", help="Print details of each asset file")

    args = parser.parse_args()

    collected: list[Path] = list(args.targets)
    if args.jar:
        collected.append(args.jar)
    if args.dir:
        collected.append(args.dir)

    if not collected:
        parser.print_help()
        print("\nError: Please provide a JAR/ZIP path via positional argument or --jar/--dir.", file=sys.stderr)
        return 1

    filter_exts = set(x.strip().lower().lstrip(".") for x in args.ext.split(",")) if args.ext else None

    return process_targets(
        collected,
        args.output,
        filter_exts=filter_exts,
        only_media=args.only_media,
        list_only=args.list,
        force=args.force,
        verbose=args.verbose,
    )


if __name__ == "__main__":
    raise SystemExit(main())
