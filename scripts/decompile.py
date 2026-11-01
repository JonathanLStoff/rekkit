#!/usr/bin/env python3
"""Decompile an original mod archive to Java source for the spec-writer and the logic review.

Uses a pinned Vineflower (fetched once into .cache/tools/, checked against its SHA-256) and the JDK
on PATH. Output goes to assets/decompiled/<archive name>/, which is gitignored: decompiled code is
read to write the spec, never committed and never copied into a mod (builders may not read it).

Mods from 1.2.5/1.4.7 reference Minecraft's obfuscated names (aan, xd...); their own classes keep
readable names. Decompile the client build: assets/tekkit/client/mods/ for Tekkit (its server ports
in assets/tekkit/mods/ are MCPC/Bukkit rewrites) and assets/direwolf20/mods/. A mod that ships as an
unpacked folder (ComputerCraft) is decompiled from that folder.

Usage:
  make decompile JAR=assets/tekkit/client/mods/RedPowerCore-2.0pr5b2.zip
  python3 scripts/decompile.py --jar <archive> [--jar <archive> ...] [--out <dir>] [--force]
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VINEFLOWER_VERSION = "1.12.0"
VINEFLOWER_SHA256 = "1dfcfe974395734fa467ce620661c7623d05ba83670de0529b1fbd63ff548b9d"
VINEFLOWER_URL = (f"https://repo1.maven.org/maven2/org/vineflower/vineflower/{VINEFLOWER_VERSION}/"
                  f"vineflower-{VINEFLOWER_VERSION}.jar")
TOOL = ROOT / ".cache" / "tools" / f"vineflower-{VINEFLOWER_VERSION}.jar"
OUT = ROOT / "assets" / "decompiled"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def vineflower() -> Path:
    """The pinned Vineflower jar, fetched once."""
    if TOOL.is_file() and sha256(TOOL) == VINEFLOWER_SHA256:
        return TOOL
    TOOL.parent.mkdir(parents=True, exist_ok=True)
    print(f"Fetching Vineflower {VINEFLOWER_VERSION} into {TOOL.relative_to(ROOT)}", flush=True)
    partial = TOOL.with_suffix(".part")
    with urllib.request.urlopen(VINEFLOWER_URL, timeout=120) as response, partial.open("wb") as out:
        shutil.copyfileobj(response, out)
    if sha256(partial) != VINEFLOWER_SHA256:
        partial.unlink()
        sys.exit(f"Vineflower download does not match the pinned SHA-256 ({VINEFLOWER_SHA256}); not using it.")
    partial.replace(TOOL)
    return TOOL


def java_works(java: str) -> bool:
    try:
        return subprocess.run([java, "-version"], capture_output=True, timeout=60).returncode == 0
    except (OSError, subprocess.TimeoutExpired):
        return False


def find_java() -> str | None:
    """A working java: JAVA_HOME's, PATH's, else the newest installed JDK. The JDK's own binary is used
    because macOS's /usr/bin/java stub cannot find a JDK inside agy's terminal sandbox."""
    exe = "java.exe" if os.name == "nt" else "java"
    found = [Path(os.environ["JAVA_HOME"]) / "bin" / exe] if os.environ.get("JAVA_HOME") else []
    if on_path := shutil.which("java"):
        found.append(Path(on_path))
    installed = [*Path("/Library/Java/JavaVirtualMachines").glob(f"*/Contents/Home/bin/{exe}"),
                 *Path("/opt/homebrew/opt").glob(f"openjdk*/bin/{exe}"), *Path("/usr/local/opt").glob(f"openjdk*/bin/{exe}"),
                 *Path("/usr/lib/jvm").glob(f"*/bin/{exe}")]
    # Newest first: the version is the numbers in the JDK's folder name (temurin-25.jdk, openjdk@21).
    installed.sort(key=lambda p: tuple(int(n) for n in re.findall(r"\d+", p.as_posix())), reverse=True)
    for java in found + installed:
        if java.is_file() and java_works(str(java)):
            return str(java)
    return None


def shown(path: Path) -> str:
    return path.relative_to(ROOT).as_posix() if path.is_relative_to(ROOT) else str(path)


def decompile(archive: Path, out_root: Path, force: bool) -> Path:
    dest = out_root / archive.stem
    if dest.is_dir() and any(dest.rglob("*.java")) and not force:
        print(f"Already decompiled: {shown(dest)} (--force to redo)")
        return dest
    java = find_java()
    if not java:
        sys.exit("No Java 17+ found (JAVA_HOME, PATH or the usual JDK folders); ./setup.sh says how to install JDK 25.")
    shutil.rmtree(dest, ignore_errors=True)
    dest.mkdir(parents=True)
    print(f"Decompiling {archive.name} into {shown(dest)}", flush=True)
    r = subprocess.run([java, "-jar", str(vineflower()), "--silent", "--folder", str(archive), str(dest)],
                       capture_output=True, text=True)
    sources = list(dest.rglob("*.java"))
    if r.returncode or not sources:
        sys.exit(f"Vineflower failed on {archive.name} (exit {r.returncode}):\n{(r.stderr or r.stdout)[-2000:]}")
    print(f"  {len(sources)} source files")
    return dest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--jar", action="append", required=True, help="Mod archive (.jar or .zip) or unpacked mod folder; repeatable")
    parser.add_argument("--out", type=Path, default=OUT, help=f"Output root (default: {OUT.relative_to(ROOT)})")
    parser.add_argument("--force", action="store_true", help="Decompile again even if the output exists")
    args = parser.parse_args()
    for jar in args.jar:
        path = Path(jar).expanduser()
        if not path.exists():
            sys.exit(f"No such archive or folder: {jar}")
        decompile(path.resolve(), args.out.resolve(), args.force)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
