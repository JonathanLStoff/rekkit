#!/usr/bin/env bash
# What agent hooks run: scripts/fleet/hook.sh <shell|paths|gate> <agy|claude>.
# Finds a Python 3.11+ and hands the hook's stdin to guard.py.
here="$(cd "$(dirname "$0")" && pwd)"
py="$("$here/python.sh")" || { echo "Rekkit hook: no Python 3.11+ found" >&2; exit 0; }
exec "$py" "$here/guard.py" "$1" --format "$2"
