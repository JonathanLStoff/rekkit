#!/usr/bin/env bash
# Headless: run the conductor on exactly ONE mod, then exit.
# Usage: scripts/fleet/run-next.sh            # resume the active row, else the next queued row
#        scripts/fleet/run-next.sh <id>       # a specific board row
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
board=planning/fleet-board.md
mkdir -p target

# One run at a time. mkdir is atomic, so it doubles as a lock.
lock=target/fleet.lock
if ! mkdir "$lock" 2>/dev/null; then
  echo "Another fleet run holds $lock. Remove it if that run is gone." >&2
  exit 1
fi
trap 'rmdir "$lock"' EXIT

# Board columns: | # | Mod | Path | Wave | Tier | Deps | State | ...
field() { awk -F'|' -v id="$1" -v col="$2" '{gsub(/ /,"",$3)} $3==id {v=$col; gsub(/^ +| +$/,"",v); print v; exit}' "$board"; }
active=$(awk -F'|' '{s=$8; gsub(/ /,"",s); m=$3; gsub(/ /,"",m)} s=="active" {print m; exit}' "$board")
id="${1:-$active}"
if [ -n "$active" ] && [ "$id" != "$active" ]; then
  echo "$active is still active. Finish it (or mark it blocked) before starting $id." >&2
  exit 1
fi
if [ -z "$id" ]; then
  # First queued row whose deps are all done.
  while IFS= read -r row; do
    deps=$(field "$row" 7)
    ok=1
    for d in $(printf '%s' "$deps" | tr ',' ' '); do
      [ "$d" = "—" ] && continue
      [ "$(field "$d" 8)" = "done" ] || { ok=0; break; }
    done
    if [ "$ok" = 1 ]; then id="$row"; break; fi
  done < <(awk -F'|' '{s=$8; gsub(/ /,"",s); m=$3; gsub(/ /,"",m)} s=="queued" {print m}' "$board")
fi
if [ -z "$id" ]; then echo "Nothing to run: no active row and no queued row with finished deps."; exit 0; fi

state=$(field "$id" 8)
if [ "$state" = "active" ] && field "$id" 17 | grep -q '▶'; then
  echo "$id is waiting for you to push, sign and release (steps 9–10)."; exit 0
fi

echo "Fleet run: $id"
agy -p "You are the conductor. Run board row '$id' per .agents/fleet-plan.md §4. Work only on this one row. Stop before push." \
  --effort high | tee "target/fleet-$id.log"
