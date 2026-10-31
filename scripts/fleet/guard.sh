#!/usr/bin/env bash
# PreToolUse guard for run_command: no publishing, signing, submodule changes,
# history rewrites or recursive deletes. A human does those.
input=$(cat)
if printf '%s' "$input" | grep -Eq 'git push|gh release|minisign|git submodule|git -C horizon|git reset --hard|--force|rm -rf'; then
  echo '{"decision":"deny","reason":"Blocked by the Rekkit fleet guard. Ask the human to run this."}'
else
  echo '{"decision":"allow"}'
fi
