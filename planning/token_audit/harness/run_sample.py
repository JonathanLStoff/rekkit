#!/usr/bin/env python3
"""Run one token-audit sample on agy in a throwaway git worktree, record its usage, then remove the worktree.

  python run_sample.py --id pilot --step S0 --prompt "..." [--agent <role>] [--model <agy model>]
                       [--max-turns 60] [--max-billed 3000000] [--runs-dir <dir>] [--keep]

Writes planning/token_audit/runs/<id>.json (summary), <id>.stream.jsonl (every agy event) and <id>.agy.log.
Never touches Beads, never commits, never pushes. The worktree is a detached copy of HEAD plus the
working tree's uncommitted tracked changes; assets/ and horizon/ are junctions to the main checkout
(removed as links first, so nothing is deleted through them).
"""
import argparse, json, os, subprocess, sys, time
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RUNS = Path(__file__).resolve().parents[1] / "runs"
# Linked, not copied: big, read-mostly. target/e2e/{source,cargo-target} hold Horizon's prepared game
# resources (made from the player's Minecraft 26.3) and its build cache; stage/bin stay per-worktree.
LINKED = ["assets", "horizon", "target/e2e/source", "target/e2e/cargo-target"]


def sh(args, cwd=None, **kw):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, **kw)


def make_worktree(wt: Path):
    r = sh(["git", "worktree", "add", "--detach", str(wt), "HEAD"], cwd=REPO)
    if r.returncode:
        sys.exit(r.stderr)
    diff = sh(["git", "diff", "HEAD", "--binary"], cwd=REPO).stdout
    if diff:
        p = subprocess.run(["git", "apply", "--whitespace=nowarn", "-"], cwd=wt, input=diff, text=True,
                           capture_output=True)
        if p.returncode:
            print("warn: could not overlay uncommitted changes:", p.stderr[:300], file=sys.stderr)
    # New files not committed yet (new scripts, role files): copied too, except the audit's own runs.
    import shutil
    for rel in sh(["git", "ls-files", "--others", "--exclude-standard"], cwd=REPO).stdout.splitlines():
        if rel.startswith("planning/token_audit/runs/"):
            continue
        src, dst = REPO / rel, wt / rel
        if src.is_file() and not dst.exists():
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
    for name in LINKED:
        dst = wt / name
        dst.parent.mkdir(parents=True, exist_ok=True)
        if dst.is_dir() and not dst.is_symlink():
            if any(dst.iterdir()):
                continue
            dst.rmdir()
        if (REPO / name).is_dir() and not dst.exists():
            subprocess.run(["cmd", "/c", "mklink", "/J", str(dst), str(REPO / name)], capture_output=True)
    hooks = REPO / ".agents" / "hooks.json"
    if hooks.is_file():
        (wt / ".agents" / "hooks.json").write_text(hooks.read_text(encoding="utf-8"), encoding="utf-8")


def remove_worktree(wt: Path):
    for name in LINKED:  # drop junctions as links, never recursively
        link = wt / name
        if link.exists() and os.path.isjunction(link):
            subprocess.run(["cmd", "/c", "rmdir", str(link)], capture_output=True)
    sh(["git", "worktree", "remove", "--force", str(wt)], cwd=REPO)
    sh(["git", "worktree", "prune"], cwd=REPO)
    # git fails on very long paths (cargo output); fall back to a long-path delete, but only for our own
    # audit folder and only if no junction is left inside it (never delete through a link).
    if wt.exists() and wt.name.startswith("rekkit-audit-"):
        for root, dirs, _ in os.walk(wt):
            if any(os.path.isjunction(os.path.join(root, d)) for d in dirs):
                print("left in place (junction inside):", wt, file=sys.stderr)
                return
        import shutil
        shutil.rmtree("\\?\\" + str(wt), ignore_errors=True)


def usage_of(ev: dict):
    u = ev.get("usage")
    return u if isinstance(u, dict) else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--id", required=True)
    ap.add_argument("--step", required=True)
    ap.add_argument("--prompt", required=True)
    ap.add_argument("--agent")
    ap.add_argument("--model", default=None, help="agy model (default: the one agy is on)")
    ap.add_argument("--max-turns", type=int, default=60)
    ap.add_argument("--max-billed", type=int, default=3_000_000)
    ap.add_argument("--runs-dir", type=Path, default=None, help="Where to write the run (default ../runs)")
    ap.add_argument("--keep", action="store_true")
    a = ap.parse_args()

    global RUNS
    RUNS = a.runs_dir or RUNS
    RUNS.mkdir(parents=True, exist_ok=True)
    wt = REPO.parent / f"rekkit-audit-{a.id}"
    make_worktree(wt)
    cmd = ["agy", "--settings", json.dumps({"toolPermission": "always-proceed"}), "-p", a.prompt,
           *(["--model", a.model] if a.model else []), "--output-format", "stream-json",
           "--log-file", str(RUNS / f"{a.id}.agy.log")]
    if a.agent:
        cmd += ["--agent", a.agent]
    started, turns, billed, last, capped = time.time(), 0, 0, {}, None
    tools, per_turn = [], []  # per_turn rows: [input, cache_read, output, thinking]
    try:
        with open(RUNS / f"{a.id}.stream.jsonl", "w", encoding="utf-8") as out:
            proc = subprocess.Popen(cmd, cwd=wt, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                                    encoding="utf-8", errors="replace")
            for line in proc.stdout:
                out.write(line)
                try:
                    ev = json.loads(line)
                except ValueError:
                    continue
                su = ev.get("step_update") or {}
                if su.get("state") == "DONE" and su.get("tool_name"):
                    p = (su.get("tool_info") or {}).get("parameters") or {}
                    o = (su.get("tool_info") or {}).get("output") or ""
                    tools.append({"step": su.get("step_index"), "tool": su["tool_name"],
                                  "arg": str(next(iter(p.values()), ""))[:200], "out_chars": len(o)})
                u = usage_of(su)
                if u and su.get("state") == "DONE":
                    last = u
                    turns += 1
                    billed += u.get("input_tokens", 0) + u.get("cache_read_tokens", 0)
                    per_turn.append([u.get("input_tokens", 0), u.get("cache_read_tokens", 0),
                                     u.get("output_tokens", 0), u.get("thinking_tokens", 0)])
                if turns >= a.max_turns:
                    capped = "turns"
                elif billed >= a.max_billed:
                    capped = "billed"
                if capped:
                    proc.kill()
                    break
            proc.wait()
    finally:
        summary = {"id": a.id, "step": a.step, "agent": a.agent, "model": a.model, "turns": turns,
                   "billed_input_sum": billed, "last_usage": last, "capped": capped,
                   "peak_input": max((r[0] + r[1] for r in per_turn), default=0),
                   "output_sum": sum(r[2] for r in per_turn), "thinking_sum": sum(r[3] for r in per_turn),
                   "per_turn": per_turn, "tools": tools,
                   "seconds": round(time.time() - started, 1)}
        (RUNS / f"{a.id}.json").write_text(json.dumps(summary, indent=1), encoding="utf-8")
        if not a.keep:
            remove_worktree(wt)
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
