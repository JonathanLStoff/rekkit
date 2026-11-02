#!/usr/bin/env python3
"""Force the horizon submodule to its remote's newest commit, whatever state it is in.

For a person (agents are blocked from it): fetches origin, then hard-resets the submodule to
origin/<default branch> (main), throwing away local commits, changes and untracked files it tracks
nothing of. Ignored files stay, so Horizon's prepared game data (.local/) and build output survive.
Every step's error is shown and skipped: a failed fetch still resets to what was fetched before, a
missing git-lfs (filter.lfs.required in a global config) is switched off for these commands.

The parent repository then shows horizon as changed: commit it when you want mods to see the new pin
(`git add horizon && git commit`). Mods keep their own "horizon" pins; the harness builds each on the
newest revision any of them pins.

Usage:
  make horizon-pull [BRANCH=<branch>]
  python3 scripts/horizon_pull.py [--branch <branch>]
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HORIZON = ROOT / "horizon"
# No git-lfs needed: Horizon has no LFS files, but a global filter.lfs.required=true fails a checkout
# when git-lfs is not installed.
NO_LFS = ["-c", "filter.lfs.required=false", "-c", "filter.lfs.smudge=", "-c", "filter.lfs.process=",
          "-c", "filter.lfs.clean="]
ENV = dict(os.environ, GIT_LFS_SKIP_SMUDGE="1", GIT_TERMINAL_PROMPT="0")


def git(*args: str, cwd: Path | None = None) -> tuple[int, str]:
    r = subprocess.run(["git", *NO_LFS, *args], cwd=cwd or HORIZON, env=ENV, capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    return r.returncode, (r.stdout + r.stderr).strip()


def step(what: str, *args: str, cwd: Path | None = None) -> str:
    code, out = git(*args, cwd=cwd)
    print(f"{'ok  ' if code == 0 else 'FAIL'} {what}" + (f"\n     {out.splitlines()[-1]}" if code and out else ""))
    return out if code == 0 else ""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--branch", help="Remote branch to take (default: origin's default branch, else main)")
    parser.add_argument("--path", type=Path, help=argparse.SUPPRESS)  # another checkout, for testing
    args = parser.parse_args()
    global HORIZON
    HORIZON = (args.path or HORIZON).resolve()

    if not (HORIZON / ".git").exists():
        step("initialise the submodule", "submodule", "update", "--init", "--force", "horizon", cwd=ROOT)
    if not (HORIZON / ".git").exists():
        print("horizon/ is still not a checkout: run `git submodule update --init horizon` by hand.", file=sys.stderr)
        return 1
    before = git("rev-parse", "HEAD")[1][:12] or "none"

    url = git("config", "--file", str(ROOT / ".gitmodules"), "submodule.horizon.url", cwd=ROOT)[1]
    if url:
        step("point origin at " + url, "remote", "set-url", "origin", url)
    for lock in (HORIZON / ".git").glob("*.lock") if (HORIZON / ".git").is_dir() else []:
        lock.unlink(missing_ok=True)  # left by a git that was killed mid-operation
    for op in ("merge", "rebase", "cherry-pick"):  # whichever is in progress (the others just say none is)
        git(op, "--abort")
    step("fetch origin", "fetch", "origin", "--prune", "--force", "--tags")

    branch = args.branch
    if not branch:
        head = git("symbolic-ref", "--short", "refs/remotes/origin/HEAD")[1]
        branch = head.split("/", 1)[1] if head.startswith("origin/") else "main"
    target = f"origin/{branch}"
    if git("rev-parse", "--verify", "--quiet", target)[0]:
        print(f"FAIL there is no {target}; nothing to reset to", file=sys.stderr)
        return 1
    step(f"check out {branch} at {target}", "checkout", "--force", "-B", branch, target)
    step(f"reset to {target}", "reset", "--hard", target)
    step("remove untracked files (ignored ones, like .local/, stay)", "clean", "-fd")
    step("track " + target, "branch", f"--set-upstream-to={target}", branch)

    after = git("rev-parse", "HEAD")[1][:12]
    behind = git("rev-list", "--count", f"HEAD..{target}")[1]
    print(f"\nhorizon: {before} -> {after} ({branch}{', ' + behind + ' behind' if behind not in ('', '0') else ', up to date'}).")
    if before != after:
        print("The parent repository now shows horizon as changed; `git add horizon && git commit` records it.")
    return 0 if after and behind in ("", "0") else 1


if __name__ == "__main__":
    raise SystemExit(main())
