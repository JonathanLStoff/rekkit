#!/usr/bin/env bash
# Stop hook: keep the agent working while any mod or crate it touched fails
# tests, validation or the harness dry run.
cat > /dev/null
root=$(git rev-parse --show-toplevel)
cd "$root" || exit 1

changed=$( { git diff --name-only HEAD; git ls-files --others --exclude-standard; } 2>/dev/null )
mods=$(printf '%s\n' "$changed" | sed -n 's#^mods/\([^/]*\)/.*#\1#p' | sort -u)
crates=$(printf '%s\n' "$changed" | sed -n 's#^crates/\([^/]*\)/.*#\1#p' | sort -u)
if [ -z "$mods$crates" ]; then echo '{"decision":"allow"}'; exit 0; fi

py=$(scripts/fleet/python.sh) || { echo '{"decision":"continue","reason":"Install Python 3.11+ so make check-mods can run."}'; exit 0; }
log=$(mktemp); fail=""
for c in $crates; do
  [ -f "crates/$c/Cargo.toml" ] || continue
  cargo test --manifest-path "crates/$c/Cargo.toml" >>"$log" 2>&1 || fail="$fail crates/$c"
done
for m in $mods; do
  [ -f "mods/$m/Cargo.toml" ] || continue
  { cargo test --manifest-path "mods/$m/Cargo.toml" \
    && make check-mods MOD="$m" PYTHON="$py" \
    && make e2e-mod MOD="$m" DRY_RUN=1 PYTHON="$py"; } >>"$log" 2>&1 || fail="$fail mods/$m"
done

if [ -z "$fail" ]; then
  echo '{"decision":"allow"}'
else
  tail -n 60 "$log" | FAIL="$fail" "$py" -c '
import json, os, sys
print(json.dumps({"decision": "continue",
  "reason": "Gate failed for:" + os.environ["FAIL"] + "\n" + sys.stdin.read()}))'
fi
rm -f "$log"
