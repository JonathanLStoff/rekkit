#!/usr/bin/env bash
# Print the first Python >= 3.11 on PATH (pack_mod.py needs tomllib).
for c in "${PYTHON:-}" python3.13 python3.12 python3.11 python3; do
  [ -n "$c" ] || continue
  if command -v "$c" >/dev/null 2>&1 && "$c" -c 'import sys; sys.exit(sys.version_info < (3, 11))' 2>/dev/null; then
    echo "$c"; exit 0
  fi
done
echo "No Python 3.11+ found" >&2
exit 1
