# Token audit

Why: a mod run was using about 30% of a weekly agy budget in 20 minutes. This folder finds where.

Phases (a person reviews between each):

1. **Map and predict** (done): [`process-map.md`](process-map.md), [`predictions.md`](predictions.md). No tokens spent on samples.
2. **Harness and pilot**: `harness/` runs one headless `agy -p --output-format json` per sample and saves its `usage` to `runs/`. One pilot first.
3. **Sample and chart**: 2 to 3 samples per step on `buildcraft`, then `performance.md` and a chart built from `runs/*.json`.
4. **Fix plan**: [`fix-plan.md`](fix-plan.md), how to cut token use, putting tokens ahead of speed.

Rules: samples run on `agy` only (the dedicated subscription), never on Claude Code. One mod, `buildcraft`. Each sample stops after its step. Changes are discarded afterwards; no `git push`, no release.
