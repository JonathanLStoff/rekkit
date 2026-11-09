#!/usr/bin/env python3
"""Play Horizon with Rekkit mods: the human's look at a finished port.

Builds what is needed (the same build the harness uses), then opens Horizon in a window with the
chosen mods (default: every mod) in a game folder that is kept between runs, target/play/<mods>/.
A new world, "Rekkit test" (creative), gets a showcase plot first: every block of every mod placed
in a row per mod at y=100, from x=0. Close the game window when done; run it again to come back.

Usage:
  make play MOD=redpower_2         one mod
  make play                         every mod
  python3 scripts/play.py [--mod a,b] [--fresh] [--offscreen]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import e2e_harness as h  # noqa: E402
import pack_mod  # noqa: E402

PLAY = h.ROOT / "target" / "play"
WORLD = "Rekkit test"
SHOWCASE_STAGES = {"2_clear_plot", "2_floor_plot", "2_plot_ready", "2_place_mod_block", "2_block_lands"}


def harness_args() -> argparse.Namespace:
    return argparse.Namespace(bin_dir=None, horizon_rev="submodule", rebuild=False, resources=None, prepare=False,
                              minecraft_dir=None, server_jar=None)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mod", action="append", help="Mod id(s), comma-separated (default: every mod)")
    parser.add_argument("--fresh", action="store_true", help="Start over with a new game folder and world")
    parser.add_argument("--offscreen", action="store_true", help="No window (for checking this command itself)")
    args = parser.parse_args()

    all_dirs = sorted(p for p in h.MODS_ROOT.iterdir() if p.is_dir() and (p / "Cargo.toml").is_file())
    try:
        dirs = h.selected_mods(args.mod, all_dirs)
        for d in dirs:
            pack_mod.process_mod(d, pack=True)
        specs = [h.discover_mod_spec(d) for d in dirs]
        hargs = harness_args()
        sub = h.ensure_submodule(fetch=False)
        bins, rev = h.prepare_build(specs, hargs, sub)
        resources = h.resources_for(rev, hargs)
    except (h.HarnessError, pack_mod.ValidationError) as exc:
        print(f"[play] {exc}", file=sys.stderr)
        return 1

    names = [s["id"] for s in specs]
    key = names[0] if len(names) == 1 else "all" if len(names) == len(all_dirs) else "-".join(names)[:60]
    home = PLAY / key
    if args.fresh:
        shutil.rmtree(home, ignore_errors=True)
    (home / ".local" / "horizon-client").mkdir(parents=True, exist_ok=True)
    link = home / ".local" / "horizon-client" / h.GAME_VERSION
    if link.is_symlink() or link.exists():
        if link.resolve() != resources.resolve():
            link.unlink() if link.is_symlink() else os.rmdir(link)
    if not link.exists():
        h.link_dir(resources, link)
    if not (home / "options.txt").is_file():
        (home / "options.txt").write_text("onboardAccessibility:false\n", encoding="utf-8")
    (home / "horizon-extras.txt").write_text("agentControl:true\n", encoding="utf-8")
    mods_dir = home / "horizon-mods"
    shutil.rmtree(mods_dir, ignore_errors=True)
    mods_dir.mkdir()
    for spec in specs:
        hmod = h.MODS_ROOT / spec["id"] / "target" / "horizon-mod" / f"{spec['id']}.hmod"
        shutil.copy2(hmod, mods_dir / hmod.name)
    (home / "horizon-mods.txt").write_text("".join(f"mod.{n}:true\n" for n in names), encoding="utf-8")

    env = os.environ.copy()
    for var in ("HORIZON_LAUNCHER_HOME", "XDG_DATA_HOME", "XDG_CONFIG_HOME"):
        (home / var.lower()).mkdir(exist_ok=True)
        env[var] = str(home / var.lower())
    env.update({"HORIZON_HOME": str(home), "HORIZON_TELEMETRY_URL": "off", "HORIZON_LAUNCHER_NO_SELF_UPDATE": "1",
                "HORIZON_NO_DISCORD": "1"})
    # A short socket path: Unix sockets are capped at about 104 bytes.
    sock_dir = Path("/tmp" if os.name == "posix" else h.tempfile.gettempdir()) / f"rk-play-{hashlib.sha1(str(home).encode()).hexdigest()[:8]}"
    sock_dir.mkdir(exist_ok=True)
    ctl = h.Ctl(bins / f"horizon-ctl{h.EXE}", sock_dir / ("c.json" if os.name == "nt" else "c.sock"), home, env)

    print(f"[play] Opening Horizon {rev[:12]} with {', '.join(names)} (game folder {home.relative_to(h.ROOT)})")
    launch = ["launch", "--game", bins / f"horizon-client{h.EXE}", "--world", WORLD, "--creative", "--wait",
              "--resources", resources] + (["--offscreen"] if args.offscreen else [])
    code, reply = ctl(*launch, timeout=900)
    if code or "error" in reply:
        print(f"[play] The game did not start: {json.dumps(reply)}", file=sys.stderr)
        return 1

    marker = home / ".showcase.json"
    if not marker.is_file() or json.loads(marker.read_text(encoding="utf-8")).get("mods") != names:
        print("[play] Placing every block on the showcase plot")
        for plot, spec in enumerate(specs):
            for step in h.build_action_plan(spec, plot_index=plot):
                if step.get("stage") in SHOWCASE_STAGES:
                    ctl(*h.step_words(step, home), timeout=120)
        marker.write_text(json.dumps({"mods": names}), encoding="utf-8")
    ctl("tp", 0.5, 100, -3.5)

    if args.offscreen:
        ctl("quit", timeout=60)
        print("[play] Checked offscreen and closed.")
        return 0
    print(f"[play] Horizon is open in world \"{WORLD}\": every block is on the showcase plot in front of you "
          f"(one row per mod, from x=0, y=100). Close the window when you are done; `make play"
          f"{' MOD=' + ','.join(names) if key != 'all' else ''}` brings you back.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
