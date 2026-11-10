#!/usr/bin/env sh
# Rekkit git hooks: keep the Beads board and the agents' MCP tools in step with git.
# Enabled per clone by ./setup.sh / setup.cmd (git config core.hooksPath .githooks).
#   sync.sh export            before a commit: write the board from Beads and stage it
#   sync.sh pulled OLD NEW    after a pull/merge/checkout/rebase: import the board if it changed,
#                             refresh the MCP tools if setup.py changed or .mcp.json is missing
# Set REKKIT_NO_HOOKS=1 to skip all of it for one command.
[ -n "${REKKIT_NO_HOOKS:-}" ] && exit 0
cd "$(git rev-parse --show-toplevel)" || exit 0
py=$(scripts/fleet/python.sh 2>/dev/null) || { echo "rekkit hooks: no Python 3.11+, board not synced" >&2; exit 0; }
have_bd() { command -v bd >/dev/null 2>&1 && [ -d .beads/embeddeddolt ]; }
gitdir=$(git rev-parse --git-dir)

case "$1" in
export)
  # Mid-merge or mid-rebase the staged board is the resolved one: do not overwrite it.
  [ -f "$gitdir/MERGE_HEAD" ] || [ -d "$gitdir/rebase-merge" ] || [ -d "$gitdir/rebase-apply" ] && exit 0
  have_bd || exit 0
  "$py" scripts/fleet/beads.py board >/dev/null 2>&1 || { echo "rekkit hooks: board export failed (commit goes on)" >&2; exit 0; }
  git add -- .beads/issues.jsonl planning/fleet-board.md
  ;;
pulled)
  old=$2 new=$3
  [ -n "$old" ] && [ -n "$new" ] && [ "$old" != "$new" ] || exit 0
  changed=$(git diff --name-only "$old" "$new" -- .beads/issues.jsonl scripts/setup.py 2>/dev/null)
  if printf '%s\n' "$changed" | grep -qx '.beads/issues.jsonl' && have_bd; then
    echo "rekkit hooks: importing the pulled board into Beads"
    bd import .beads/issues.jsonl >/dev/null 2>&1 || echo "rekkit hooks: bd import failed; run: bd import .beads/issues.jsonl" >&2
  fi
  if printf '%s\n' "$changed" | grep -qx 'scripts/setup.py' || [ ! -f .mcp.json ]; then
    echo "rekkit hooks: refreshing the agents' MCP tools"
    "$py" scripts/setup.py --mcp-only || true
  fi
  ;;
esac
exit 0
