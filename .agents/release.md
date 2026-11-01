# Releasing a mod

Every mod is released on its own, never bundled: the GitHub release `<id>-<version>` with `<id>.hmod`, `<id>.hnmod` and their `.minisig` signatures. [`.github/workflows/release-mods.yml`](../.github/workflows/release-mods.yml) makes it when a push to `main` carries a version of the mod that has no release yet. Agents never push, sign or publish: a person pushes (and approves the `mod-release` environment); the workflow does the rest.

## Pre-release or full release: the art decides

Rekkit's code is our own (clean-room: written from the spec, never from the decompiled Java). The **art and sound are the original mod's** until we replace them, and we keep them only to test. So:

| The mod ships | Release | Horizon's registry |
|---|---|---|
| Any original texture or sound | **Pre-release**, notes say "Test build" and credit the original authors | Skips it: not listed, not installable from the launcher |
| Only our own art and sound | **Full release** | Reads it within the hour and publishes it (once the mod is listed) |

What counts, from `mods/<id>/asset-map.json`:

- Every `.png`, `.ogg` and `.wav` under `content/assets/` is **original** unless an entry with that `out` path says otherwise: `slices` and `copies` default to `"origin": "original"`, `generated` entries are `"origin": "generated"` (made by `make art`), and redrawn originals are `"origin": "derived"` once they are no longer the original's pixels.
- A file the map does not mention counts as original.
- `"credits"`: one line naming the original mod and its authors (for example `"RedPower 2 by Eloraam"`), quoted in every pre-release's notes.

Check a mod before a push: `python3 scripts/release_plan.py --mod <id>` prints `to release` or `to pre-release; N original asset(s)`.

## Going to a full release

A mod reaches a full release only through its **11 Own art** story (deferred until a person starts that pass):

1. Every original texture and sound is replaced: drawn in `mods/<id>/art/` and built with `make art MOD=<id>` (same size and palette rules as new art), or recorded/edited sound. Each replaced entry in `asset-map.json` becomes `generated` (or `derived`), and the original files are deleted.
2. `python3 scripts/release_plan.py --mod <id>` shows 0 original assets; `make check-mods MOD=<id>` (with `--complete`) and the live harness still pass.
3. Bump the version (lockstep, 4 files) and commit. The person pushes; the workflow publishes a full release.
4. Once per mod, a person lists it at [playhorizon.dev/mods/submit](https://playhorizon.dev/mods/submit) with this repository, its page and the key in `minisign.pub`. A Horizon reviewer reads it before it is published.

Until then, test builds live on the repository's GitHub releases page as pre-releases.
