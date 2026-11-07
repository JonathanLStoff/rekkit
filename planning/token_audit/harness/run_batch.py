#!/usr/bin/env python3
"""Run every sample in samples.json once, in order, skipping ones already in ../runs/. Usage: python run_batch.py [step ...]"""
import json, subprocess, sys
from pathlib import Path
H = Path(__file__).resolve().parent
done = {p.stem for p in (H.parent / "runs").glob("*.json")}
want = set(sys.argv[1:])
for s in json.load(open(H / "samples.json")):
    if s["id"] in done or (want and s["step"] not in want):
        continue
    cmd = [sys.executable, str(H / "run_sample.py"), "--id", s["id"], "--step", s["step"],
           "--prompt", s["prompt"], "--max-billed", str(s["max_billed"])]
    print(">>", s["step"], flush=True)
    print(subprocess.run(cmd, capture_output=True, text=True).stdout[-400:], flush=True)
