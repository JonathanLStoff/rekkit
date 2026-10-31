#!/usr/bin/env bash
# PreToolUse guard for file writes: horizon/ is a read-only submodule, and
# content/ directories hold data only (no Cargo.toml, no .rs).
root=$(git rev-parse --show-toplevel)
ROOT="$root" "$("$root/scripts/fleet/python.sh")" -c '
import json, os, re, sys
from pathlib import Path

root = Path(os.environ["ROOT"]).resolve()
try:
    payload = json.load(sys.stdin)
except ValueError:
    print(json.dumps({"decision": "allow"})); sys.exit()

def paths(node):
    # Collect string values of keys that name a file, not file contents.
    if isinstance(node, dict):
        for key, value in node.items():
            low = key.lower()
            if isinstance(value, str) and ("path" in low or low in ("file", "target", "targetfile")):
                yield value
            else:
                yield from paths(value)
    elif isinstance(node, list):
        for item in node:
            yield from paths(item)

for raw in paths(payload):
    target = Path(raw)
    target = (target if target.is_absolute() else root / target).resolve()
    # macOS volumes are usually case-insensitive, so compare prefixes that way.
    full, base = target.as_posix(), root.as_posix()
    if full.lower() == base.lower():
        continue
    if not full.lower().startswith(base.lower() + "/"):
        continue
    rel = full[len(base) + 1:]
    if rel == "horizon" or rel.startswith("horizon/"):
        print(json.dumps({"decision": "deny",
            "reason": "horizon/ is read-only. Log the gap in planning/horizon-gaps.md."})); sys.exit()
    if re.match(r"mods/[^/]+/content/", rel) and (rel.endswith("Cargo.toml") or rel.endswith(".rs")):
        print(json.dumps({"decision": "deny",
            "reason": "No Cargo.toml or .rs files under mods/<id>/content/."})); sys.exit()
print(json.dumps({"decision": "allow"}))
'
