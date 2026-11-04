#!/usr/bin/env python3
"""What agent hooks run on Windows (cmd or PowerShell, no bash): python ../scripts/fleet/hook.py <check> <format>.
Same job as hook.sh: finds a Python 3.11+ and hands the hook's stdin to guard.py."""
import os
import shutil
import subprocess
import sys

here = os.path.dirname(os.path.abspath(__file__))
guard = os.path.join(here, "guard.py")
args = sys.argv[1:3]
if len(args) < 2:
    sys.exit("usage: hook.py <check> <format>")
tail = [args[0], "--format", args[1]]


def ok(cmd: list[str]) -> bool:
    try:
        return subprocess.run(cmd + ["-c", "import sys; sys.exit(sys.version_info < (3, 11))"],
                              capture_output=True).returncode == 0
    except OSError:
        return False


if sys.version_info >= (3, 11):
    cmd = [sys.executable]
else:
    cands = [[os.environ["PYTHON"]]] if os.environ.get("PYTHON") else []
    cands += [["py", "-3.13"], ["py", "-3.12"], ["py", "-3.11"], ["python3"], ["python"]]
    cmd = next((c for c in cands if shutil.which(c[0]) and ok(c)), None)
    if cmd is None:
        sys.stderr.write("Rekkit hook: no Python 3.11+ found\n")
        sys.exit(0)

sys.exit(subprocess.run(cmd + [guard] + tail, stdin=sys.stdin).returncode)
