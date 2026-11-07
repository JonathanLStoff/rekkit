#!/usr/bin/env python3
"""Re-run token-audit samples after a change and compare them with the first audit (fix-plan E1).

Runs each chosen sample of samples.json N times on agy (run_sample.py: a throwaway worktree of HEAD plus
the uncommitted and new files, never Beads, never a push), writes the runs to runs/after/<id>-<n>.json and
a table, after.md, beside performance.md: per sample the baseline's (runs/<id>.json, one sample) and the
mean of the new runs: turns, billed input, weighted cost, discovery commands, guard refusals, finished.

Usage:
  make token-audit [SAMPLES=s0-start,s3a-full] [N=2] [MODEL=<agy model>]
  python planning/token_audit/harness/audit.py [--samples ...] [--n 2] [--model M] [--report-only]

Without SAMPLES: the steps the fix plan re-measures (S0, C1, C2, S1b, S1c, S3a+-S3d+, S4b, S7). MODEL
defaults to the model agy is on. It spends agy quota: run it yourself, not from an agent.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

H = Path(__file__).resolve().parent
T = H.parent
RUNS = T / "runs"
AFTER = RUNS / "after"
DEFAULT = ["s0-start", "c1-conductor", "c2-conductor-loop", "s1b-outline", "s1c-tickets", "s3a-full", "s3b-full",
           "s3c-full", "s3d-full", "s4-parity-critic", "s7-recheck"]
DISCOVERY = re.compile(r"Get-ChildItem[^|;&]*-Recurse|\b(?:ls|dir)\b[^|;&]*(?:-R\b|-Recurse|/s\b)|(?<![\w-])find\s|"
                       r"(?<![\w-])tree\b|git\s+status|git\s+log[^|;&]*-p\b|^\s*(?:dir|ls|Get-ChildItem)\s*$", re.I)
REFUSED = re.compile(r"fleet guard|Token budget reached|Do not read|Refused \(|already ran|already read|"
                     r"At most 2 lines|No glob|has \d+ lines: read|Search the decompiled|Find a spec section", re.I)


def summarize(run: dict, stream: Path | None) -> dict:
    pt = run.get("per_turn", [])
    fresh, cache = sum(r[0] for r in pt), sum(r[1] for r in pt)
    out = run.get("output_sum", 0)
    cmds = [t["arg"] for t in run.get("tools", []) if t["tool"] == "run_command"]
    refused = 0
    if stream and stream.is_file():
        refused = len(REFUSED.findall(stream.read_text(encoding="utf-8", errors="replace")))
    return {"turns": run.get("turns", 0), "billed": fresh + cache, "weighted": fresh + 0.1 * cache + 5 * out,
            "discovery": sum(bool(DISCOVERY.search(c)) for c in cmds), "refused": refused,
            "finished": not run.get("capped")}


def k(n: float) -> str:
    return f"{n / 1e6:.2f}M" if n >= 1e6 else f"{n / 1000:.0f}k"


def report(samples: list[dict]) -> str:
    rows = ["| Sample | Step | Turns before -> after | Billed before -> after | Weighted before -> after | "
            "Discovery cmds | Guard refusals | Finished | Runs |", "|---|---|---|---|---|---|---|---|---|"]
    for s in samples:
        base_path = RUNS / f"{s['id']}.json"
        news = sorted(AFTER.glob(f"{s['id']}-*.json"))
        if not news:
            continue
        base = summarize(json.loads(base_path.read_text(encoding="utf-8")), RUNS / f"{s['id']}.stream.jsonl") \
            if base_path.is_file() else None
        after = [summarize(json.loads(p.read_text(encoding="utf-8")), p.with_suffix(".stream.jsonl")) for p in news]
        mean = {key: sum(a[key] for a in after) / len(after) for key in ("turns", "billed", "weighted", "discovery", "refused")}
        fin = sum(a["finished"] for a in after)

        def pair(key: str, fmt=k) -> str:
            b = fmt(base[key]) if base else "-"
            return f"{b} -> {fmt(mean[key])}"
        rows.append(f"| {s['id']} | {s['step']} | {pair('turns', lambda n: f'{n:.0f}')} | {pair('billed')} | "
                    f"{pair('weighted')} | {pair('discovery', lambda n: f'{n:.1f}')} | {mean['refused']:.1f} | "
                    f"{fin}/{len(after)} | {len(after)} |")
    return ("# Token audit: after the fix plan\n\nBaseline: the first audit's one sample per step (`runs/<id>.json`, "
            "gemini-3.8-flash-medium). After: the mean of the runs in `runs/after/`. Built by "
            "`harness/audit.py`; the targets are in [`fix-plan.md`](fix-plan.md) (Phase E).\n\n" + "\n".join(rows) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--samples", default="", help="Comma-separated sample ids (default: the fix plan's)")
    ap.add_argument("--n", type=int, default=2, help="Runs per sample (default 2)")
    ap.add_argument("--model", default=None)
    ap.add_argument("--report-only", action="store_true", help="Rebuild after.md from the runs there are")
    a = ap.parse_args()
    all_samples = json.loads((H / "samples.json").read_text(encoding="utf-8"))
    wanted = [x.strip() for x in a.samples.split(",") if x.strip()] or DEFAULT
    chosen = [s for s in all_samples if s["id"] in wanted]
    missing = set(wanted) - {s["id"] for s in chosen}
    if missing:
        print(f"Unknown sample(s): {', '.join(sorted(missing))}", file=sys.stderr)
        return 1
    AFTER.mkdir(parents=True, exist_ok=True)
    if not a.report_only:
        for s in chosen:
            for n in range(1, a.n + 1):
                run_id = f"{s['id']}-{n}"
                if (AFTER / f"{run_id}.json").is_file():
                    continue
                cmd = [sys.executable, str(H / "run_sample.py"), "--id", run_id, "--step", s["step"],
                       "--prompt", s["prompt"], "--max-billed", str(s["max_billed"]), "--runs-dir", str(AFTER),
                       *(["--model", a.model] if a.model else [])]
                print(f">> {run_id}", flush=True)
                r = subprocess.run(cmd, capture_output=True, text=True)
                print((r.stdout or r.stderr)[-300:], flush=True)
    text = report(chosen)
    (T / "after.md").write_text(text, encoding="utf-8", newline="\n")
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
