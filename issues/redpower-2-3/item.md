---
created: 2026-11-12
updated: 2026-11-12
type: epic
status: needs-info
priority: high
labels:
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- tier:Hard
- wave:1
beads_id: rk-qst
---

# redpower_2

domain (`src/lib.rs`) + unit tests (`tests/mod_tests.rs`) written; content subagents authoring 42 blocks, 29 items, 15 recipes

## Beads import

- Beads id: `rk-qst`
- Beads status: `blocked`
- Beads priority: `1`
- Beads type: `epic`
- Created (Beads): 2026-11-01T17:40:17Z

## Comments

### 2026-11-12T16:47:55Z · @beads-import

[2026-11-02T14:38:52Z · Jonathan Stoff] Reopened 2026-11-02: play-testing found a stand-in, not RedPower. Blocks are cube_all models with vanilla or single textures, thin blocks (gates, wires, panels) are full cubes, nothing faces the player, every GUI is a stock chest grid with the wrong slots, sprites are swapped, machines and tools do nothing. The harness only checked placing, opening some screen and dropping. Done now means `make contract-check MOD=redpower_2` is clean and the harness's shape, GUI and scenario checks pass.

### 2026-11-12T16:47:55Z · @beads-import

[2026-11-03T03:51:22Z · Jonathan Stoff] BLOCKED (needs a person), 2026-11-02. (1) Step 8 live harness: install JDK 25 (brew install openjdk@25) and Minecraft Java 26.3 (launcher, play 26.3 once), then ./setup.sh. (2) Horizon gaps found by ~60 tickets, filed as follow-ups labelled horizon: block-entity world access in tick (get/set block, drops, fake-player use), neighbor-change/redstone-input hook, chunk-gen/feature hook, random-tick/hurt-enemy/place hooks in HorizonNativeMod, native containers with N slots + crafting result slot, ItemStack NBT. Without them breaker/igniter/deployer/frame motor/ores/flora/athame stay pure logic with no world effect, so contract scenarios cannot pass. Decision for the human: extend horizon/ (submodule, agents may not) or accept those features as deferred. Progress: step 3 ~100/493 beads, contract diffs 1155->1033; cargo test 270+17 pass.

### 2026-11-12T16:47:55Z · @beads-import

[2026-11-07T13:21:26Z · unknown] BLOCKED: agy:default model quota exhausted during fleet-dispatch (resets ~08:49 / 09:20). To resume: wait until reset and run 'make fleet-dispatch MOD=redpower_2' or resume immediately with fallback model via 'make fleet-dispatch MOD=redpower_2 FALLBACK=1'.

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-07T18:28:02Z · unknown] BLOCKED: agy:default model quota exhausted during fleet-dispatch (resets ~Sat 13:54). All tests pass (363 tests) and agent-gate is clean. To resume: wait until ~13:54 and run 'make fleet-dispatch MOD=redpower_2' or immediately run 'make fleet-dispatch MOD=redpower_2 FALLBACK=1'.

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-08T02:10:03Z · unknown] Unblocked by user: continuing in flat mode.

### 2026-11-12T16:47:56Z · @beads-import

[2026-11-10T18:25:07Z · Jonathan Stoff] Paused by person (2026-11-10): local redpower_2 changes discarded back to 41cca5b; the user is restarting with the new fff/repomap MCP tools. Unblock with: bd update rk-qst --status in_progress
