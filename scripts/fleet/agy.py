#!/usr/bin/env python3
"""Start agy on this repository with its terminal sandbox off.

agy runs agent commands in a sandbox that hides the network, ~/.dolt and the Minecraft install, which
the port needs (cargo downloads, Beads, the harness), and asks a person to approve every command that
fails there. agy has no project setting for it, so this passes the session-only setting
toolPermission=always-proceed: commands run directly, and the fleet's hooks (.agents/hooks.json) still
block pushing, signing, releasing, submodule changes, force flags and recursive deletes.

Usage:
  make agy [ARGS="--agent conductor"]
  python3 scripts/fleet/agy.py [agy arguments...]
"""

import json
import shutil
import subprocess
import sys

SETTINGS = {"toolPermission": "always-proceed"}


def command(args: list[str]) -> list[str]:
    return ["agy", "--settings", json.dumps(SETTINGS), *args]


if __name__ == "__main__":
    if not shutil.which("agy"):
        sys.exit("agy is not installed or not on PATH.")
    raise SystemExit(subprocess.call(command(sys.argv[1:])))
