#!/usr/bin/env python3
"""Attribute context growth to the tool calls that caused it. For every turn after the first, the growth in
context (input + cache read) minus the previous turn's output is what the tools between the two turns put
into context. Prints per-class totals and writes attribution.json. Usage: attribute.py [run-id ...]"""
import json, re, sys, glob
from pathlib import Path
from collections import defaultdict
T = Path(__file__).resolve().parents[1]

def klass(tool, arg):
    a = arg.lower()
    if tool == "read_file": return "read_file"
    if tool in ("write_file", "edit_file", "replace_file_content", "multi_replace_file_content"): return "write/edit"
    if "java-show" in a: return "make java-show"
    if "java-outline" in a: return "make java-outline"
    if re.search(r"spec-(show|toc)|fleet-context", a): return "make spec/context"
    if re.search(r"agent-gate|e2e|cargo|clippy|pack-mods|check-mods|contract-check", a): return "build/test/gate"
    if re.search(r"get-childitem|\bls\b|\bdir\b|find |tree", a): return "dir listing sweep"
    if re.search(r"select-string|grep|rg ", a): return "text search"
    if re.search(r"git (status|log|diff|show)", a): return "git inspect"
    if "make" in a: return "other make"
    return "other command"

def run(rid):
    seq = []  # ("turn", ctx, out) or ("tool", class, out_chars)
    for l in open(T / "runs" / f"{rid}.stream.jsonl", encoding="utf-8", errors="replace"):
        try: s = json.loads(l).get("step_update") or {}
        except ValueError: continue
        if s.get("state") != "DONE": continue
        u = s.get("usage")
        if u: seq.append(("turn", u["input_tokens"] + u["cache_read_tokens"], u["output_tokens"]))
        elif s.get("tool_name"):
            p = (s.get("tool_info") or {}).get("parameters") or {}
            arg = " ".join(str(v) for v in p.values())
            seq.append(("tool", klass(s["tool_name"], arg), arg[:80]))
    byk = defaultdict(int); cnt = defaultdict(int); worst = []
    prev_ctx = prev_out = None; pending = []
    for e in seq:
        if e[0] == "tool": pending.append(e); continue
        if prev_ctx is not None:
            grown = max(e[1] - prev_ctx - prev_out, 0)
            if pending:
                share = grown / len(pending)
                for _, k, a in pending:
                    byk[k] += share; cnt[k] += 1; worst.append((share, k, a))
            else: byk["model/system (no tool)"] += grown
        prev_ctx, prev_out, pending = e[1], e[2], []
    worst.sort(reverse=True)
    return dict(by_class={k: round(v) for k, v in byk.items()}, calls=dict(cnt), worst=[(round(w), k, a) for w, k, a in worst[:3]])

ids = sys.argv[1:] or [Path(f).stem for f in sorted(glob.glob(str(T / "runs" / "*.json"))) if "pilot" not in f]
out = {i: run(i) for i in ids}
(T / "attribution.json").write_text(json.dumps(out, indent=1), encoding="utf-8")
tot = defaultdict(int); n = defaultdict(int)
for r in out.values():
    for k, v in r["by_class"].items(): tot[k] += v
    for k, v in r["calls"].items(): n[k] += v
print("| Tool class | Calls | Tokens added to context | Per call | Share |"); print("|---|---|---|---|---|")
g = sum(tot.values()) or 1
for k, v in sorted(tot.items(), key=lambda x: -x[1]):
    print(f"| {k} | {n.get(k,'-')} | {v/1000:.1f}k | {(v/n[k]/1000 if n.get(k) else 0):.2f}k | {100*v/g:.0f}% |")
