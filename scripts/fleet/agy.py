#!/usr/bin/env python3
"""Start agy on this repository with its terminal sandbox off.

agy runs agent commands in a sandbox that hides the network and the Minecraft install, which
the port needs (cargo downloads, the issue tracker, the harness), and asks a person to approve every command that
fails there. agy has no project setting for it, so this passes the session-only setting
toolPermission=always-proceed: commands run directly, and the fleet's hooks (.agents/hooks.json) still
block pushing, signing, releasing, submodule changes, force flags and recursive deletes.

Usage:
  make agy [ARGS="--agent conductor"]
  python3 scripts/fleet/agy.py [agy arguments...]

On Windows it first installs .agents/hooks.windows.json as hooks.json (no bash needed).
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

SETTINGS = {"toolPermission": "always-proceed"}
HOOKS = Path(__file__).resolve().parents[2] / ".agents"


def windows_hooks() -> bool:
    """On Windows, agy's hooks run in cmd/PowerShell, which cannot start `../scripts/fleet/hook.sh`:
    put hooks.windows.json (python ../scripts/fleet/hook.py) in place of hooks.json, and keep git from
    showing it as a change. Elsewhere does nothing. True when hooks.json is right for this OS."""
    if not sys.platform.startswith("win"):
        return True
    src, dst = HOOKS / "hooks.windows.json", HOOKS / "hooks.json"
    if not src.is_file():
        return False
    # agy does not run hooks from .agents/: `../scripts/...` fails (python exits 2) and agy dies on the
    # first shell command. The installed copy (never committed) names the scripts by absolute path.
    scripts = (HOOKS.parent / "scripts").as_posix()
    # (agy passes quotes through literally, so the path stays unquoted; a path with a space keeps `../`.)
    text = src.read_text(encoding="utf-8")
    if " " not in scripts:
        text = text.replace("python ../scripts/", f"python {scripts}/")
    if not dst.is_file() or dst.read_text(encoding="utf-8") != text:
        dst.write_text(text, encoding="utf-8", newline="\n")
    subprocess.run(["git", "update-index", "--skip-worktree", ".agents/hooks.json"],
                   cwd=HOOKS.parent, capture_output=True)
    return True


def command(args: list[str]) -> list[str]:
    return ["agy", "--settings", json.dumps(SETTINGS), *args]


if __name__ == "__main__":
    if sys.argv[1:] == ["--install-hooks"]:  # setup: only the hooks swap
        raise SystemExit(0 if windows_hooks() else 1)
    windows_hooks()
    if not shutil.which("agy"):
        sys.exit("agy is not installed or not on PATH.")
    raise SystemExit(subprocess.call(command(sys.argv[1:])))
