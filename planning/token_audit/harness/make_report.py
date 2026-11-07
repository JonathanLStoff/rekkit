#!/usr/bin/env python3
"""Build performance.md tables and performance.svg from runs/*.json. Narrative lives in performance_notes.md."""
import json, glob
from pathlib import Path
T = Path(__file__).resolve().parents[1]
ORDER = ["S0","C1","C2","S1b","S1c","S3a","S3a+","S3b","S3b+","S3c","S3c+","S3d","S3d+","S4a","S4b","S5","S6","S7","S8","S8c","S9"]
LABEL = {"C2":"C2 conductor loop (3 events)","S3a+":"S3 content-author, 1.5M cap","S3b+":"S3 domain-builder, 1.5M cap","S3c+":"S3 hook-engineer, 1.5M cap","S3d+":"S3 test-smith, 1.5M cap","S8c":"S8 live harness (real)","S0":"S0 session start","C1":"C1 conductor boot","S1b":"S1b outline (1 block)","S1c":"S1c tickets (1 ticket)",
 "S3a":"S3 content-author*","S3b":"S3 domain-builder*","S3c":"S3 hook-engineer*","S3d":"S3 test-smith*",
 "S4a":"S4 agent-gate","S4b":"S4 parity-critic","S5":"S5 asset-smith","S6":"S6 new assets","S7":"S7 recheck","S8":"S8 harness","S9":"S9 package"}
runs = {}
for f in glob.glob(str(T / "runs" / "*.json")):
    d = json.load(open(f))
    if d["id"] != "pilot": runs[d["step"]] = d
rows = []
for s in ORDER:
    d = runs[s]; pt = d["per_turn"]
    fresh = sum(r[0] for r in pt); cache = sum(r[1] for r in pt); out = d["output_sum"]
    weighted = fresh + 0.1 * cache + 5 * out
    cmds = sum(1 for t in d["tools"] if t["tool"] == "run_command"); reads = sum(1 for t in d["tools"] if t["tool"] == "read_file")
    rows.append(dict(step=s, label=LABEL[s], turns=d["turns"], peak=d["peak_input"], billed=fresh + cache, fresh=fresh,
                     cache=cache, out=out, think=d["thinking_sum"], weighted=round(weighted), cmds=cmds, reads=reads,
                     floor=pt[0][0] + pt[0][1], capped=d["capped"], secs=d["seconds"]))
k = lambda n: f"{n/1000:.1f}k" if n < 1e6 else f"{n/1e6:.2f}M"
md = ["| Step | Turns | Peak ctx | Billed input (sum of turns) | of which cache read | Output (thinking) | Weighted cost | Cmds / reads | Note |", "|---|---|---|---|---|---|---|---|---|"]
for r in rows:
    md.append(f"| {r['label']} | {r['turns']} | {k(r['peak'])} | {k(r['billed'])} | {100*r['cache']//max(r['billed'],1)}% | {k(r['out'])} ({k(r['think'])}) | {k(r['weighted'])} | {r['cmds']} / {r['reads']} | {(('capped at '+('500k' if r['step'] in ('S3a','S3b','S3c','S3d') else '1.5M')+' billed, task unfinished') if r['capped'] else '')} |")
(T / "performance_table.md").write_text("\n".join(md) + "\n", encoding="utf-8")

# SVG: horizontal bars, billed input split fresh/cache, weighted marker
W, L, RW, H, top = 880, 190, 640, 26, 60
mx = max(r["billed"] for r in rows)
svg = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {top + H*len(rows) + 70}" font-family="system-ui,sans-serif" font-size="12">',
       '<style>.t{fill:#222}.g{stroke:#ddd}@media(prefers-color-scheme:dark){.t{fill:#ddd}.g{stroke:#444}rect.bg{fill:#111}}</style>',
       f'<rect class="bg" width="100%" height="100%" fill="#fff"/>',
       f'<text class="t" x="16" y="24" font-size="15" font-weight="600">Billed input tokens per sampled step (1 sample each, gemini-3.8-flash-medium)</text>',
       f'<rect x="{L}" y="36" width="12" height="12" fill="#c0392b"/><text class="t" x="{L+18}" y="46">fresh input</text>',
       f'<rect x="{L+100}" y="36" width="12" height="12" fill="#e8a09a"/><text class="t" x="{L+118}" y="46">cache read</text>',
       f'<rect x="{L+210}" y="36" width="12" height="12" fill="#2c3e50"/><text class="t" x="{L+228}" y="46">weighted cost (in 0.1x cache, 5x output)</text>']
for i, r in enumerate(rows):
    y = top + i * H
    wf, wc, ww = r["fresh"] / mx * RW, r["cache"] / mx * RW, r["weighted"] / mx * RW
    svg += [f'<text class="t" x="{L-8}" y="{y+15}" text-anchor="end">{r["label"]}</text>',
            f'<rect x="{L}" y="{y+2}" width="{max(wf,1):.1f}" height="9" fill="#c0392b"/>',
            f'<rect x="{L+wf:.1f}" y="{y+2}" width="{wc:.1f}" height="9" fill="#e8a09a"/>',
            f'<rect x="{L}" y="{y+13}" width="{max(ww,1):.1f}" height="6" fill="#2c3e50"/>',
            f'<text class="t" x="{L+wf+wc+6:.1f}" y="{y+11}">{k(r["billed"])} / {r["turns"]} turns</text>']
svg.append(f'<text class="t" x="16" y="{top+H*len(rows)+30}">* stopped by the 500k billed cap before finishing: a lower bound.  S4a/S6/S8/S9 were one-command prompts, so they show the floor only.</text></svg>')
(T / "performance.svg").write_text("\n".join(svg), encoding="utf-8")
print("\n".join(md))
