#!/usr/bin/env bash
# Headless: one conductor run on ONE board row (see run_next.py). Kept for existing schedules;
# AGENT (the tool tried first) defaults to agy here, as before. Usage: scripts/fleet/run-next.sh [<id>]
here="$(cd "$(dirname "$0")" && pwd)"
AGENT="${AGENT:-agy}" exec "$("$here/python.sh")" "$here/run_next.py" "$@"
