#!/usr/bin/env python3
"""Set up a Rekkit checkout from a fresh clone, or report what is missing (--check).

Every step a machine can do runs here; the one step only a person can do (owning and playing
Minecraft: Java Edition 26.3 once) is named with exact instructions. Safe to run again: finished
steps are skipped.

Exit status: 0 everything is ready, 1 a step failed, 2 ready except steps that need a person,
3 (--check only) steps remain that ./setup.sh would do.

Usage (`make setup ARGS=...` and, on Windows, `setup.cmd` take the same flags):
  ./setup.sh                 # everything
  ./setup.sh --check         # report only, change nothing
  ./setup.sh --skip-packs    # no modpack downloads (they are only needed to port a new mod)
  ./setup.sh --skip-build    # no Horizon build (only Rust domain work and content JSON)
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WINDOWS = sys.platform.startswith("win")
RERUN = "setup.cmd" if WINDOWS else "./setup.sh"
# Status marks and download logs are UTF-8; a Windows console or redirect may default to cp1252.
for _stream in (sys.stdout, sys.stderr):
    _stream.reconfigure(encoding="utf-8", errors="replace")
if sys.version_info < (3, 11):
    sys.exit(f"Python 3.11 or newer is required (this is {sys.version.split()[0]}): "
             + ("winget install Python.Python.3.13" if WINDOWS else "brew install python@3.13"
                if sys.platform == "darwin" else "sudo apt install python3.12"))
sys.path.insert(0, str(ROOT / "scripts"))

import e2e_harness as harness  # noqa: E402

PACKS = ("tekkit", "direwolf20", "agrarian")
MIN_RUST = (1, 90)
OK, NEEDS_PERSON, FAILED, SKIPPED, TODO = "ok", "needs a person", "failed", "skipped", "to do"
MARK = {OK: "✓", NEEDS_PERSON: "!", FAILED: "✗", SKIPPED: "–", TODO: "·"}


class Report:
    def __init__(self) -> None:
        self.rows: list[tuple[str, str, str]] = []

    def add(self, step: str, state: str, detail: str = "") -> str:
        self.rows.append((step, state, detail))
        print(f"  {MARK[state]} {step}: {detail or state}", flush=True)
        return state

    def print(self) -> int:
        print("\n" + "=" * 72 + "\nRekkit setup summary\n" + "=" * 72)
        for step, state, detail in self.rows:
            print(f"{MARK[state]} {step:<22} {state:<15} {detail}")
        states = {state for _, state, _ in self.rows}
        if FAILED in states:
            print(f"\nA step failed. Fix the ✗ line above and run {RERUN} again.")
            return 1
        if NEEDS_PERSON in states:
            print("\nThe ! lines need a person (agents: tell the user exactly what they say, then keep "
                  "working on what does not need them)." + (f" Run {RERUN} for the · lines." if TODO in states
                                                         else " Everything a machine can do is done."))
            return 2
        if TODO in states:
            print(f"\nNot finished: run {RERUN} to do the · lines.")
            return 3
        print("\nAll set. Next: read CONTRIBUTING.md, then tell your agent \"Follow AGENTS.md\" (or run "
              "`make fleet-next AGENT=agy|claude|codex`).")
        return 0


def sh(cmd: list, **kw) -> subprocess.CompletedProcess:
    args = [str(c) for c in cmd]
    args[0] = shutil.which(args[0]) or args[0]  # resolves Windows .cmd/.bat shims such as npm's bd
    return subprocess.run(args, cwd=kw.pop("cwd", ROOT), text=True, encoding="utf-8",
                          errors="replace", **kw)


def tail_of(res: subprocess.CompletedProcess, lines: int) -> str:
    return "\n".join((res.stdout + res.stderr).strip().splitlines()[-lines:])


def dotenv() -> dict:
    """The environment plus .env (as the Makefile's `-include .env` gives make targets)."""
    env = dict(os.environ)
    path = ROOT / ".env"
    if path.is_file():
        for line in path.read_text(encoding="utf-8").splitlines():
            key, eq, value = line.partition("=")
            if eq and not key.strip().startswith("#") and value.strip():
                env.setdefault(key.strip(), value.strip().strip('"\''))
    return env


def version_of(cmd: list[str]) -> str | None:
    if not shutil.which(cmd[0]):
        return None
    r = subprocess.run(cmd, capture_output=True, text=True)
    return (r.stdout + r.stderr).strip()


def install_hint(tool: str) -> str:
    if WINDOWS:
        return {
            "git": "winget install Git.Git",
            "make": "winget install ezwinports.make",
            "cc": "winget install Microsoft.VisualStudio.2022.BuildTools --override "
                  '"--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive"',
            "cargo": "winget install Rustlang.Rustup   (then open a new terminal)",
            "java": "winget install EclipseAdoptium.Temurin.25.JDK",
            "issuectl": "brew install jarimustonen/issuectl/issuectl   (Homebrew; any install that puts issuectl on PATH)",
        }.get(tool, "")
    mac = sys.platform == "darwin"
    hints = {
        "git": "brew install git" if mac else "sudo apt install git",
        "make": "xcode-select --install" if mac else "sudo apt install build-essential",
        "cc": "xcode-select --install" if mac else "sudo apt install build-essential",
        "cargo": "curl https://sh.rustup.rs -sSf | sh   (then open a new terminal)",
        "java": "brew install openjdk@25" if mac else "sudo apt install openjdk-25-jdk",
        "linux-libs": "sudo apt install pkg-config libasound2-dev libudev-dev",
        "issuectl": "brew install jarimustonen/issuectl/issuectl" if mac else "brew install jarimustonen/issuectl/issuectl   (Homebrew; any install that puts issuectl on PATH)",
        "rsvg": "brew install librsvg" if mac else "sudo apt install librsvg2-bin",
    }
    return hints.get(tool, "")


# ---------------------------------------------------------------- steps

def have_linker() -> bool:
    """A C linker for Rust: `cc` on Unix; on Windows the MSVC build tools (found by vswhere, as
    rustc does) unless the toolchain is the GNU one."""
    if not WINDOWS:
        return bool(shutil.which("cc"))
    if "windows-gnu" in (version_of(["rustc", "-vV"]) or ""):
        return bool(shutil.which("gcc"))
    vswhere = Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"),
                   "Microsoft Visual Studio", "Installer", "vswhere.exe")
    if shutil.which("link") or not vswhere.is_file():
        return bool(shutil.which("link"))
    found = subprocess.run([vswhere, "-latest", "-products", "*", "-requires",
                            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"],
                           capture_output=True, text=True)
    return bool(found.stdout.strip())


def long_paths_enabled() -> bool:
    """Windows limits paths to 260 characters unless LongPathsEnabled is set; Horizon's unpacked
    source under target/e2e/source/<40-hex rev>/ goes past that."""
    import winreg
    try:
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\FileSystem") as key:
            return winreg.QueryValueEx(key, "LongPathsEnabled")[0] == 1
    except OSError:
        return False


def check_tools(r: Report) -> bool:
    good = True
    if shutil.which("git"):
        r.add("tool: git", OK)
    else:
        good = False
        r.add("tool: git", FAILED, f"missing; install with: {install_hint('git')}")
    # Every step here runs without make; it only provides the `make <target>` shortcuts.
    if shutil.which("make"):
        r.add("tool: make", OK)
    else:
        r.add("tool: make", SKIPPED, f"optional (the `make` shortcuts); install with: {install_hint('make')}")
    if not have_linker():
        good = False
        r.add("tool: C linker", FAILED, f"missing; install with: {install_hint('cc')}")

    rust = version_of(["rustc", "--version"])
    m = re.search(r"(\d+)\.(\d+)", rust or "")
    if not rust or not shutil.which("cargo"):
        good = False
        r.add("tool: Rust", FAILED, f"missing; install with: {install_hint('cargo')}")
    elif (int(m.group(1)), int(m.group(2))) < MIN_RUST:
        good = False
        r.add("tool: Rust", FAILED, f"{rust}; Horizon needs {MIN_RUST[0]}.{MIN_RUST[1]}+ (rustup update)")
    else:
        r.add("tool: Rust", OK, rust)

    r.add("tool: Python", OK, f"{sys.version.split()[0]} ({sys.executable})")

    java = version_of(["java", "-version"])
    jm = re.search(r'version "(\d+)', java or "")
    if jm and int(jm.group(1)) >= 25:
        r.add("tool: Java", OK, java.splitlines()[0])
    else:
        # Only horizon-prepare needs it, so it never blocks the rest.
        r.add("tool: Java", NEEDS_PERSON,
              f"{'found ' + java.splitlines()[0] if java else 'missing'}; JDK 25 is needed to prepare game "
              f"resources: {install_hint('java')}")

    if sys.platform.startswith("linux"):
        missing = [lib for lib in ("alsa", "libudev")
                   if subprocess.run(["pkg-config", "--exists", lib], capture_output=True).returncode]
        if missing or not shutil.which("pkg-config"):
            good = False
            r.add("tool: Linux libraries", FAILED, f"missing {missing or ['pkg-config']}: {install_hint('linux-libs')}")
        else:
            r.add("tool: Linux libraries", OK)
        if not os.environ.get("DISPLAY") and not os.environ.get("WAYLAND_DISPLAY") and not shutil.which("vscreen"):
            r.add("display (Linux)", NEEDS_PERSON,
                  "no DISPLAY: the live game run needs a desktop session or a virtual screen (Xvfb, vscreen)")
    elif WINDOWS and not long_paths_enabled():
        r.add("long paths (Windows)", NEEDS_PERSON,
              "off: Horizon's source and build go past 260 characters. In an administrator PowerShell run "
              "`New-ItemProperty HKLM:\\SYSTEM\\CurrentControlSet\\Control\\FileSystem -Name LongPathsEnabled "
              "-Value 1 -PropertyType DWORD -Force` and `git config --global core.longpaths true`, or clone to "
              "a short folder such as C:\\rk")
    return good


def step_submodule(r: Report, check: bool) -> bool:
    if (harness.HORIZON_ROOT / ".git").exists() and (harness.HORIZON_ROOT / "Cargo.toml").is_file():
        return r.add("horizon submodule", OK, "initialised") == OK
    if check:
        return r.add("horizon submodule", TODO, "empty") == OK
    res = sh(["git", "submodule", "update", "--init", "horizon"])
    if res.returncode:
        return r.add("horizon submodule", FAILED, "git submodule update --init horizon failed (network?)") == OK
    return r.add("horizon submodule", OK, "initialised now") == OK


def step_env(r: Report, check: bool) -> None:
    env = ROOT / ".env"
    if env.is_file():
        r.add(".env", OK, "present")
    elif check:
        r.add(".env", SKIPPED, "not created yet (optional: CURSEFORGE_API_KEY)")
    else:
        shutil.copy2(ROOT / ".env.example", env)
        r.add(".env", OK, "created from .env.example (the CurseForge key is optional)")


def step_agents(r: Report, check: bool) -> None:
    """Each agent tool's copy of .agents/ (Claude Code's .claude/), as `make agents` writes it."""
    cmd = [sys.executable, ROOT / "scripts" / "fleet" / "sync_agents.py"] + (["--check"] if check else [])
    res = sh(cmd, capture_output=True)
    if res.returncode == 0:
        r.add("agent copies", OK, "up to date" if check else "written from .agents/")
    elif check:
        r.add("agent copies", TODO, "out of date: make agents")
    else:
        r.add("agent copies", FAILED, f"`make agents` failed:\n{tail_of(res, 5)}")


def step_agy_hooks(r: Report, check: bool) -> None:
    """Windows: agy's hooks.json must run python (hooks.windows.json), not bash."""
    if not WINDOWS:
        return
    hooks = ROOT / ".agents"
    ready = (hooks / "hooks.json").read_text(encoding="utf-8") == (hooks / "hooks.windows.json").read_text(encoding="utf-8")
    if ready:
        r.add("agy hooks", OK, "hooks.json runs python (Windows)")
    elif check:
        r.add("agy hooks", TODO, "hooks.json needs bash: run setup.cmd or `make agy`")
    else:
        res = sh([sys.executable, ROOT / "scripts" / "fleet" / "agy.py", "--install-hooks"], capture_output=True)
        r.add("agy hooks", OK if res.returncode == 0 else FAILED,
              "hooks.windows.json installed as hooks.json" if res.returncode == 0 else tail_of(res, 5))


def step_tracker(r: Report, check: bool) -> None:
    """The issue tracker (issuectl), where the port's progress lives: installed, issues/ present, every mod an epic."""
    if not shutil.which("issuectl"):
        r.add("Issue tracker (issuectl)", NEEDS_PERSON,
              f"not installed; it tracks the port's progress: {install_hint('issuectl')}")
        return
    if not (ROOT / "issues" / ".schema.yaml").is_file():
        r.add("Issue tracker (issuectl)", FAILED, "issues/.schema.yaml is missing: the tracker's schema is committed "
                                                  "with the issues; restore it from git")
        return
    sys.path.insert(0, str(ROOT / "scripts" / "fleet"))
    import tracker
    try:
        mods = tracker.epics()
        if not mods and not check:
            sh([sys.executable, ROOT / "scripts" / "fleet" / "tracker.py", "seed"], capture_output=True)
            mods = tracker.epics()
    except tracker.TrackerError as exc:
        r.add("Issue tracker (issuectl)", FAILED, str(exc))
        return
    active = next((m for m, e in mods.items() if e["status"] == "in_progress"), None)
    r.add("Issue tracker (issuectl)", OK if mods else TODO,
          f"{len(mods)} mods tracked" + (f", {active} in progress" if active else "") if mods else "no mods yet: ./setup.sh seeds them")


def step_port_tools(r: Report, check: bool) -> None:
    """Vineflower (decompiles the originals for the spec) and rsvg-convert (SVG art)."""
    sys.path.insert(0, str(ROOT / "scripts"))
    import decompile
    have = decompile.TOOL.is_file() and decompile.sha256(decompile.TOOL) == decompile.VINEFLOWER_SHA256
    if not have and not check:
        try:
            decompile.vineflower()
            have = True
        except (OSError, SystemExit) as exc:
            r.add("tool: Vineflower", FAILED, f"download failed: {exc}")
            return
    r.add("tool: Vineflower", OK if have else TODO,
          f"{decompile.VINEFLOWER_VERSION} (decompiles the original mods)" if have else "not fetched yet: ./setup.sh fetches it")
    r.add("tool: rsvg-convert", OK if shutil.which("rsvg-convert") else SKIPPED,
          "SVG art" if shutil.which("rsvg-convert") else f"optional (SVG art; .pixels art works without): {install_hint('rsvg')}")


REPOMAPPER_URL = "https://github.com/pdavis68/RepoMapper.git"
REPOMAPPER_COMMIT = "3ef8914b3a2271695ac9e4b07ce1e8bf5a4c9be6"
REPOMAPPER_DIR = Path.home() / ".local" / "share" / "repomapper"
FFF_WINDOWS = "irm https://raw.githubusercontent.com/dmtrKovalenko/fff/main/install-mcp.ps1 | iex"


def fff_binary() -> str | None:
    found = shutil.which("fff-mcp")
    if found:
        return found
    local = Path.home() / ".local" / "bin" / ("fff-mcp.exe" if WINDOWS else "fff-mcp")
    return str(local) if local.is_file() else None


def repomapper_python() -> Path:
    return REPOMAPPER_DIR / ".venv" / ("Scripts/python.exe" if WINDOWS else "bin/python")


def step_mcp_tools(r: Report, check: bool) -> None:
    """The agents' code-finding MCP servers: fff (file and symbol search) and RepoMapper (ranked repo
    map). Both are optional; .mcp.json (gitignored, this machine's absolute paths) wires in what is installed."""
    fff = fff_binary()
    if not fff and not check and not WINDOWS and shutil.which("brew"):
        res = sh(["brew", "install", "dmtrKovalenko/fff/fff-mcp"], capture_output=True)
        fff = fff_binary()
        if not fff:
            r.add("tool: fff MCP", FAILED, f"brew install failed:\n{tail_of(res, 5)}")
    if fff:
        r.add("tool: fff MCP", OK, "file and symbol search for agents")
    elif not r.rows or r.rows[-1][0] != "tool: fff MCP":
        hint = FFF_WINDOWS + "   (in PowerShell)" if WINDOWS else "brew install dmtrKovalenko/fff/fff-mcp"
        r.add("tool: fff MCP", TODO if check and not WINDOWS else SKIPPED, f"optional: {hint}")

    server = REPOMAPPER_DIR / "repomap_server.py"
    py = repomapper_python()
    ready = server.is_file() and py.is_file()
    if not ready and not check:
        steps = []
        if not (REPOMAPPER_DIR / ".git").is_dir():
            steps.append(["git", "clone", "--quiet", REPOMAPPER_URL, REPOMAPPER_DIR])
        steps += [["git", "-C", REPOMAPPER_DIR, "checkout", "--quiet", REPOMAPPER_COMMIT],
                  [sys.executable, "-m", "venv", REPOMAPPER_DIR / ".venv"],
                  [py, "-m", "pip", "install", "--quiet", "-r", REPOMAPPER_DIR / "requirements.txt"]]
        for cmd in steps:
            res = sh(cmd, capture_output=True)
            if res.returncode:
                r.add("tool: RepoMapper MCP", FAILED, f"`{' '.join(str(c) for c in cmd[:3])}` failed:\n{tail_of(res, 5)}")
                break
        ready = server.is_file() and py.is_file()
    if ready:
        r.add("tool: RepoMapper MCP", OK, f"ranked repo map for agents ({REPOMAPPER_DIR})")
    elif check:
        r.add("tool: RepoMapper MCP", TODO, f"not installed: {RERUN} clones it to {REPOMAPPER_DIR}")

    servers = {}
    if fff:
        servers["fff"] = {"type": "stdio", "command": fff, "args": []}
    if ready:
        servers["repomap"] = {"type": "stdio", "command": str(py), "args": [str(server)]}
    mcp = ROOT / ".mcp.json"
    text = json.dumps({"mcpServers": servers}, indent=2) + "\n"
    current = mcp.read_text(encoding="utf-8") if mcp.is_file() else ""
    if current == text:
        r.add(".mcp.json", OK, ", ".join(servers) or "no MCP servers installed")
    elif check:
        r.add(".mcp.json", TODO, f"{RERUN} writes it for {', '.join(servers) or 'no servers'}")
    else:
        mcp.write_text(text, encoding="utf-8")
        r.add(".mcp.json", OK, f"written: {', '.join(servers) or 'no MCP servers installed'} (restart your agent)")


def step_git_hooks(r: Report, check: bool) -> None:
    """The repo's .githooks: export the board before each commit, import it (and refresh the MCP
    tools) after each pull. A clone's local core.hooksPath, so a global one does not hide them."""
    res = sh(["git", "config", "--local", "--get", "core.hooksPath"], capture_output=True)
    if res.stdout.strip() == ".githooks":
        r.add("git hooks", OK, "board and MCP tools sync on commit and pull (.githooks)")
    elif check:
        r.add("git hooks", TODO, f"not enabled: {RERUN} sets core.hooksPath to .githooks")
    else:
        res = sh(["git", "config", "--local", "core.hooksPath", ".githooks"], capture_output=True)
        r.add("git hooks", OK if res.returncode == 0 else FAILED,
              "enabled: board and MCP tools sync on commit and pull" if res.returncode == 0 else tail_of(res, 3))


def step_mods(r: Report, check: bool) -> bool:
    if check:
        built = list(ROOT.glob("mods/*/target/horizon-mod/*.hmod"))
        return r.add("mod tests + checks", OK if built else TODO,
                     "packaged before" if built else "not run yet") != FAILED
    # What `make test-mods` and `make check-mods` run, without needing make.
    for manifest in sorted(ROOT.glob("mods/*/Cargo.toml")):
        res = sh(["cargo", "test", "--manifest-path", manifest], capture_output=True)
        if res.returncode:
            return r.add("mod tests", FAILED, f"`cargo test` in {manifest.parent.name} failed:\n{tail_of(res, 8)}") == OK
    r.add("mod tests", OK)
    res = sh([sys.executable, ROOT / "scripts" / "pack_mod.py", "--check-only"], capture_output=True)
    if res.returncode:
        return r.add("mod checks", FAILED, f"`make check-mods` failed:\n{tail_of(res, 8)}") == OK
    r.add("mod checks", OK)
    return True


def pack_jars(pack: str) -> int:
    """Mod archives downloaded for a pack; 0 when missing or incomplete (Tekkit without its client build)."""
    info = ROOT / "assets" / pack / "pack-info.json"
    if not info.is_file():
        return 0
    data = json.loads(info.read_text(encoding="utf-8"))
    if pack == "tekkit" and not data.get("client_mods"):
        return 0
    return data.get("mod_jars", 0)


def step_packs(r: Report, check: bool, skip: bool) -> None:
    for pack in PACKS:
        jars = pack_jars(pack)
        if jars:
            r.add(f"modpack: {pack}", OK, f"{jars} mod archives in assets/{pack}")
            continue
        if skip or check:
            r.add(f"modpack: {pack}", SKIPPED if skip else TODO, "not downloaded (needed to port its mods): make " + pack)
            continue
        print(f"  … downloading {pack} (can take several minutes)", flush=True)
        res = sh([sys.executable, ROOT / "scripts" / "fetch_modpacks.py", pack, "--output", ROOT / "assets",
                  "--cache", ROOT / ".cache" / "modpacks"], capture_output=True, env=dotenv())
        jars = res.returncode == 0 and pack_jars(pack)
        if jars:
            r.add(f"modpack: {pack}", OK, f"{jars} mod archives")
        else:
            r.add(f"modpack: {pack}", FAILED, f"`make {pack}` failed:\n{tail_of(res, 5)}")


def harness_cmd(*extra: str) -> list:
    return [sys.executable, ROOT / "scripts" / "e2e_harness.py", "--phase", "all-mods", *extra]


def step_build(r: Report, check: bool, skip: bool) -> bool:
    bins = sorted(harness.E2E_ROOT.glob(f"bin/*/horizon-client{harness.EXE}"))
    if bins and check:
        return r.add("Horizon build", OK, f"cached in {bins[-1].parent.relative_to(ROOT)}") == OK
    if skip or check:
        return r.add("Horizon build", SKIPPED if skip else TODO, "not built: make e2e-setup") == OK
    print("  … building Horizon with every mod's native hooks (first time: 5-20 minutes); "
          f"log: {harness.E2E_ROOT / 'build.log'}", flush=True)
    res = sh(harness_cmd("--setup-only"), capture_output=True)
    if res.returncode:
        return r.add("Horizon build", FAILED, f"`make e2e-setup` failed:\n{tail_of(res, 12)}") == OK
    return r.add("Horizon build", OK, "horizon-client + horizon-ctl built with the mods' mixins") == OK


def step_resources(r: Report, check: bool, args) -> bool:
    mc = harness.minecraft_dir(args.minecraft_dir)
    have = sorted(harness.E2E_ROOT.glob(f"source/*/.local/horizon-client/{harness.GAME_VERSION}/horizon-resources.json"))
    if have:
        r.add("Minecraft 26.3", OK, f"resources prepared ({have[-1].parent.relative_to(ROOT)})")
        return True
    if not harness.minecraft_installed(mc):
        r.add("Minecraft 26.3", NEEDS_PERSON,
              f"not found in {mc / 'versions' / harness.GAME_VERSION}. A person who owns Minecraft: Java Edition "
              f"must install the official launcher (minecraft.net/download), sign in, choose version "
              f"{harness.GAME_VERSION}, press Play once, quit, then run {RERUN} again. "
              "(Other folder: --minecraft-dir PATH)")
        return False
    java = version_of(["java", "-version"]) or ""
    if not re.search(r'version "(2[5-9]|[3-9]\d)', java):
        r.add("Minecraft 26.3", NEEDS_PERSON, f"found in {mc}, but JDK 25 is needed to prepare: {install_hint('java')}")
        return False
    if check:
        r.add("Minecraft 26.3", TODO, f"found in {mc}; game resources not prepared yet")
        return False
    print(f"  … preparing game resources from {mc} (first time: a few minutes); "
          f"log: {harness.E2E_ROOT / 'prepare.log'}", flush=True)
    cmd = harness_cmd("--setup-only", "--prepare", "--minecraft-dir", mc)
    if args.server_jar:
        cmd += ["--server-jar", args.server_jar]
    res = sh(cmd, capture_output=True)
    if res.returncode:
        r.add("Minecraft 26.3", FAILED, f"preparing resources failed:\n{tail_of(res, 12)}")
        return False
    r.add("Minecraft 26.3", OK, "resources prepared")
    return True


def step_smoke(r: Report, check: bool, skip: bool, ready: bool) -> None:
    report = harness.REPORT_DIR / "summary.json"
    if check or skip or not ready:
        last = ""
        if report.is_file():
            phases = json.loads(report.read_text(encoding="utf-8")).get("phases", {})
            live = [e for entries in phases.values() for e in entries if "status" in e]
            if live:
                last = f"last live run: {', '.join(e['session'] + ' ' + e['status'] for e in live)}"
        if not ready and not check:
            state, last = SKIPPED, ""
        elif skip and not check:
            state = SKIPPED
        else:
            state = OK if ready and "ok" in last and "failed" not in last else TODO
        r.add("live smoke test", state, last or ("needs Minecraft 26.3 resources first" if not ready
                                                 else "not run: make e2e-mod MOD=example_mod"))
        return
    print("  … booting Horizon with example_mod and running its 7-stage action plan", flush=True)
    res = sh([sys.executable, ROOT / "scripts" / "e2e_harness.py", "--phase", "isolated", "--mod", "example_mod",
              "--no-fetch"], capture_output=True)
    if res.returncode:
        r.add("live smoke test", FAILED, f"see target/e2e-report/summary.json:\n{tail_of(res, 10)}")
    else:
        r.add("live smoke test", OK, "example_mod: every action ran with no errors")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true", help="Report what is ready; change nothing")
    parser.add_argument("--skip-packs", action="store_true", help="Do not download the three modpacks")
    parser.add_argument("--skip-build", action="store_true", help="Do not build Horizon (and skip the live test)")
    parser.add_argument("--skip-smoke", action="store_true", help="Do not boot the game at the end")
    parser.add_argument("--minecraft-dir", help="Official launcher game directory, if not the default")
    parser.add_argument("--server-jar", help="A 26.3 server jar you already have (else it is fetched)")
    parser.add_argument("--mcp-only", action="store_true", help="Only install the MCP tools and write .mcp.json "
                        "(the post-pull git hook runs this)")
    args = parser.parse_args()

    if args.mcp_only:
        r = Report()
        step_mcp_tools(r, args.check)
        return 0 if all(state != FAILED for _, state, _ in r.rows) else 1
    print(f"Rekkit setup in {ROOT}" + (" (check only)" if args.check else ""))
    r = Report()
    tools = check_tools(r)
    if not tools and not args.check:
        return r.print()
    if step_submodule(r, args.check):
        step_env(r, args.check)
        step_agents(r, args.check)
        step_agy_hooks(r, args.check)
        step_tracker(r, args.check)
        step_port_tools(r, args.check)
        step_mcp_tools(r, args.check)
        step_git_hooks(r, args.check)
        mods_ok = step_mods(r, args.check)
        step_packs(r, args.check, args.skip_packs)
        built = mods_ok and step_build(r, args.check, args.skip_build)
        ready = step_resources(r, args.check, args) if built or args.check else False
        step_smoke(r, args.check, args.skip_smoke or args.skip_build, ready and built)
    return r.print()


if __name__ == "__main__":
    raise SystemExit(main())
