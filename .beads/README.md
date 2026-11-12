# Beads (retired): archive

Rekkit no longer tracks issues in Beads. Every issue moved to `issues/` (managed with `issuectl`, from `brew install jarimustonen/issuectl/issuectl`): one folder per issue, `item.md` with frontmatter, committed with the code.

`issues.jsonl` here is the frozen export the move was made from. Its `id` is kept as `beads_id` in each migrated issue (`issuectl ls --all --json`), so a Beads id can be traced to its new slug. Do not add to it or run `bd` against it.
