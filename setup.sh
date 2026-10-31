#!/usr/bin/env bash
# Rekkit setup: run this first on a fresh clone. Safe to run again.
#   ./setup.sh           do every step a machine can do
#   ./setup.sh --check   report what is ready; change nothing
# Exit status: 0 ready, 2 ready except steps that need a person, 1 a step failed.
set -euo pipefail
cd "$(dirname "$0")"
if ! py=$(scripts/fleet/python.sh 2>/dev/null); then
  echo "✗ Python 3.11 or newer is required." >&2
  case "$(uname -s)" in
    Darwin) echo "  Install it with: brew install python@3.13" >&2 ;;
    Linux)  echo "  Install it with: sudo apt install python3.12   (or your distribution's python3.11+)" >&2 ;;
  esac
  exit 1
fi
exec "$py" scripts/setup.py "$@"
