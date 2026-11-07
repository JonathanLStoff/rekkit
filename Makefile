-include .env
export

# The scripts need Python 3.11+ (tomllib); pick the newest one on PATH. Recipes avoid Unix-only
# shell so they run under cmd.exe too (GNU make on Windows, e.g. `winget install ezwinports.make`).
ifndef PYTHON
ifeq ($(OS),Windows_NT)
PYTHON := $(if $(shell py -3 -c "print(1)"),py -3,python)
else
PYTHON := $(shell scripts/fleet/python.sh 2>/dev/null || echo python3)
endif
endif
OUT ?= ./assets
CACHE ?= ./.cache/modpacks
FORCE ?= 0
JAR ?=
DIR ?=
MOD ?=
DRY_RUN ?= 0
REV ?=
EXTRACT_OUT ?= ./assets/extracted
ARGS ?=
BEAD ?=
ROLE ?=
COMPLETE ?= 0

export CURSEFORGE_API_KEY

.PHONY: fleet-local contract-check fleet-contract fleet-parts fleet-tickets fleet-ready setup doctor all tekkit direwolf20 agrarian list clean help extract-jar extract-assets test-mods check-mods pack-mods e2e-setup e2e-mod e2e-all-mods integration-test agents agents-check agent-gate fleet-next fleet-models agy fleet-start fleet-resume tasks watch board-html board-live tickets-live fleet-stop \
	fleet-task fleet-inbox fleet-features fleet-gaps core-need version-sync fleet-status fleet-context spec-toc spec-show board decompile java-outline java-show java-find art play horizon-baseline horizon-status horizon-pull pull \
	fleet-dispatch content-gen e2e-gen skill-toc skill-show conductor-step spec-merge token-audit

all: tekkit direwolf20 agrarian

setup:
	$(PYTHON) scripts/setup.py $(ARGS)

doctor:
	$(PYTHON) scripts/setup.py --check

tekkit:
	$(PYTHON) scripts/fetch_modpacks.py tekkit --output "$(OUT)" --cache "$(CACHE)" $(if $(filter 1,$(FORCE)),--force,)

direwolf20:
	$(PYTHON) scripts/fetch_modpacks.py direwolf20 --output "$(OUT)" --cache "$(CACHE)" $(if $(filter 1,$(FORCE)),--force,)

agrarian:
	$(PYTHON) scripts/fetch_modpacks.py agrarian --output "$(OUT)" --cache "$(CACHE)" $(if $(filter 1,$(FORCE)),--force,)

list:
	$(PYTHON) scripts/fetch_modpacks.py list

clean:
	$(PYTHON) -c "import shutil, sys; [shutil.rmtree(p, ignore_errors=True) for p in sys.argv[1:]]" "$(OUT)" "$(CACHE)"

extract-jar:
	$(PYTHON) scripts/extract_jar_assets.py $(if $(JAR),--jar "$(JAR)",) $(if $(EXTRACT_OUT),--output "$(EXTRACT_OUT)",) $(ARGS)

extract-assets:
	$(PYTHON) scripts/extract_jar_assets.py $(if $(JAR),--jar "$(JAR)",) $(if $(DIR),--dir "$(DIR)",) $(if $(EXTRACT_OUT),--output "$(EXTRACT_OUT)",) $(ARGS)

# One target per crate (test-mod/crates/<id>/Cargo.toml and test-mod/mods/<id>/Cargo.toml), so no shell loop is needed.
test-mods: $(addprefix test-mod/,$(wildcard crates/*/Cargo.toml) $(wildcard mods/*/Cargo.toml))

test-mod/%:
	$(info ==> Testing $*)
	cargo test --manifest-path "$*"

# COMPLETE=1 also fails on any asset 26.3 needs that the mod lacks (the New assets and Recheck steps).
check-mods:
	$(PYTHON) scripts/pack_mod.py --check-only $(if $(filter 1,$(COMPLETE)),--complete,) $(if $(MOD),--mod "$(MOD)",)

pack-mods:
	$(PYTHON) scripts/pack_mod.py $(if $(MOD),--mod "$(MOD)",)

E2E_FLAGS = $(if $(MOD),--mod "$(MOD)",) $(if $(REV),--horizon-rev "$(REV)",) $(if $(filter 1,$(DRY_RUN)),--dry-run,) $(ARGS)

e2e-setup:
	$(PYTHON) scripts/e2e_harness.py --phase both --setup-only $(E2E_FLAGS)

e2e-mod:
	$(PYTHON) scripts/e2e_harness.py --phase isolated $(E2E_FLAGS)

e2e-all-mods:
	$(PYTHON) scripts/e2e_harness.py --phase all-mods $(E2E_FLAGS)

integration-test:
	$(PYTHON) scripts/e2e_harness.py --phase both $(E2E_FLAGS)

# Agents: .agents/ is the source; each tool's copy is generated from it.
agents:
	$(PYTHON) scripts/fleet/sync_agents.py

agents-check:
	$(PYTHON) scripts/fleet/sync_agents.py --check

# The port against its contract (planning/mods/<id>.features.json): shapes, facing, every face's and icon's
# sprite pixel for pixel, stack sizes and durability. COMPLETE=1 check-mods runs it too.
contract-check:
	$(PYTHON) scripts/contract_check.py $(if $(filter 1,$(SCHEMA)),--schema,--mod "$(MOD)") $(if $(OWNER),--owner $(OWNER),) $(if $(filter 1,$(VERBOSE)),--limit 0,)

# Content and e2e drafts from a mod's contract (the mechanical files no agent should write by hand).
content-gen:
	$(PYTHON) scripts/content_gen.py --mod "$(MOD)" $(if $(NAME),--name "$(NAME)",) $(ARGS)

e2e-gen:
	$(PYTHON) scripts/e2e_gen.py --mod "$(MOD)" $(ARGS)

# Local models only: how many agents this machine's memory holds on the selected model (1 = no subagents).
fleet-local:
	$(PYTHON) scripts/fleet/local_capacity.py $(ARGS)

# Step 1: the decompiled code in parts one spec-writer can read in ~200k tokens (one writer per part).
fleet-parts:
	$(PYTHON) scripts/fleet/parts.py $(MOD)

# The spec-writers' tickets (planning/mods/<id>.tickets/*.json): CHECK=1 only validates (sizes, roles,
# order, files two tickets edit at once, coverage); otherwise files them with their order.
fleet-tickets:
	$(PYTHON) scripts/fleet/beads.py $(if $(filter 1,$(SCHEMA)),tickets-schema,$(if $(filter 1,$(CHECK)),tickets-check,tickets)) $(MOD)

# The spec-writers' part outlines and attributes into the spec and outline (fleet-tickets runs it too).
spec-merge:
	$(PYTHON) scripts/fleet/spec_merge.py $(MOD)

# Tickets whose predecessors are all closed: one agent each (fleet-dispatch works them).
fleet-ready:
	$(PYTHON) scripts/fleet/beads.py ready $(MOD)

# The ticket loop without an LLM: a fresh headless agent per ready ticket, at most MAX at once.
fleet-dispatch:
	$(PYTHON) scripts/fleet/dispatch.py $(MOD) $(if $(AGENT),--agent $(AGENT),) $(if $(MAX),--max $(MAX),) $(if $(BUDGET),--budget $(BUDGET),) $(if $(filter 1,$(FALLBACK)),--fallback,) $(if $(filter 1,$(CONDUCTOR)),--conductor,)

# Token audit samples on agy (planning/token_audit/): N runs of each, compared with the baseline.
token-audit:
	$(PYTHON) planning/token_audit/harness/audit.py $(if $(SAMPLES),--samples "$(SAMPLES)",) $(if $(N),--n $(N),) $(if $(MODEL),--model "$(MODEL)",) $(ARGS)

# File each contract difference as a bead under the step that owns it.
fleet-contract:
	$(PYTHON) scripts/fleet/beads.py contract $(MOD)

agent-gate:
	$(PYTHON) scripts/fleet/guard.py gate $(if $(MOD),--mod "$(MOD)",)

fleet-next:
	$(PYTHON) scripts/fleet/run_next.py $(if $(AGENT),--agent $(AGENT),) $(if $(filter 1,$(FALLBACK)),--fallback,) $(MOD)

# agy on this repository with its terminal sandbox off (the port needs the network, Beads and the game).
agy:
	$(PYTHON) scripts/fleet/agy.py $(ARGS)

# The model chain headless runs use (.agents/models.json) and which models are resting after a limit.
fleet-models:
	$(PYTHON) scripts/fleet/models.py $(if $(AGENT),--prefer $(AGENT),) $(ARGS)

# The conductor's first command for a mod: claim it (the hooks hold the session until it is
# finished), create its stories in Beads, and check Horizon itself at the pinned revision.
fleet-start:
	$(PYTHON) scripts/fleet/guard.py claim $(MOD)
	$(PYTHON) scripts/fleet/beads.py start $(MOD)
	-$(PYTHON) scripts/e2e_harness.py --baseline --no-fetch
	$(PYTHON) scripts/fleet/resume.py $(MOD)

# Where a mod's run stands after a crash, lost credits or a network drop: steps, interrupted subagents
# and how to restart them, unfiled reports, the last conductor's actions.
fleet-resume:
	$(PYTHON) scripts/fleet/resume.py $(MOD)

fleet-stop:
	$(PYTHON) scripts/fleet/guard.py stop

# A role subagent's first command: it works BEAD as ROLE and reports to target/fleet/inbox/.
fleet-task:
	$(PYTHON) scripts/fleet/guard.py task $(BEAD) $(if $(ROLE),--role $(ROLE),)

fleet-inbox:
	$(PYTHON) scripts/fleet/beads.py inbox

fleet-features:
	$(PYTHON) scripts/fleet/beads.py features $(MOD)

fleet-gaps:
	$(PYTHON) scripts/fleet/beads.py gaps $(MOD)

# Every changed mod (and rekkit_core) one patch version ahead of its last commit, in lockstep; more work
# before a commit stays on that version. The stop gate, fleet-inbox and agent-gate run it too.
version-sync:
	$(PYTHON) scripts/version_sync.py $(if $(MOD),--mod $(MOD),) $(ARGS)

# Shared code a mod needs: a bead under rekkit_core (reopened if closed), done within this mod's run.
core-need:
	$(PYTHON) scripts/fleet/beads.py core-need $(MOD) --title "$(TITLE)" --why "$(WHY)"

fleet-status:
	$(PYTHON) scripts/fleet/beads.py status $(MOD)

# One short page per mod: Beads state, spec, every original archive in every pack, what is decompiled
# and what the mod folder holds (instead of exploring with ls and find).
fleet-context:
	$(PYTHON) scripts/fleet/context.py $(MOD)

# A spec a section at a time (builders read only what their bead needs).
spec-toc:
	$(PYTHON) scripts/fleet/spec.py toc $(MOD)

spec-show:
	$(PYTHON) scripts/fleet/spec.py show $(MOD) "$(SECTION)"

# A skill, or the conductor's instructions for one step, a section at a time.
skill-toc:
	$(PYTHON) scripts/fleet/spec.py skill-toc $(SKILL)

skill-show:
	$(PYTHON) scripts/fleet/spec.py skill-show $(SKILL) "$(SECTION)"

conductor-step:
	$(PYTHON) scripts/fleet/spec.py step $(STEP)

board:
	$(PYTHON) scripts/fleet/beads.py board

# Looking at Beads: the mod in progress on one screen, redrawn every 30 s, or a browser page.
tasks:
	@$(PYTHON) scripts/fleet/view.py tasks $(MOD)

watch:
	@$(PYTHON) scripts/fleet/view.py watch $(MOD)

board-html:
	$(PYTHON) scripts/fleet/view.py html

board-live:
	$(PYTHON) scripts/fleet/view.py serve

# Every Beads ticket in a web page (filter, search, read the wording), like Jira.
tickets-live:
	$(PYTHON) scripts/fleet/view.py tickets-live

decompile:
	$(PYTHON) scripts/decompile.py --jar "$(JAR)" $(ARGS)

# Decompiled Java a piece at a time: the outline, then only the members needed.
java-outline:
	$(PYTHON) scripts/java_outline.py outline "$(DIR)" $(if $(GREP),--grep "$(GREP)",) $(ARGS)

java-show:
	$(PYTHON) scripts/java_outline.py show "$(DIR)" "$(MEMBER)" $(ARGS)

java-find:
	$(PYTHON) scripts/java_outline.py find "$(DIR)" "$(TEXT)" $(ARGS)

art:
	$(PYTHON) scripts/art.py --mod $(MOD) $(ARGS)

play:
	$(PYTHON) scripts/play.py $(if $(MOD),--mod "$(MOD)",) $(ARGS)

horizon-baseline:
	$(PYTHON) scripts/e2e_harness.py --baseline $(ARGS)

horizon-status:
	$(PYTHON) scripts/e2e_harness.py --status

# A person's: force the horizon submodule to origin/main (local changes and stuck operations dropped,
# errors shown and skipped, .local/ kept). Agents are blocked from it.
horizon-pull:
	$(PYTHON) scripts/horizon_pull.py $(if $(BRANCH),--branch "$(BRANCH)",)

# A person's: fast-forward this repository from its remote, then horizon-pull. Agents are blocked from it.
pull:
	git pull --ff-only
	$(PYTHON) scripts/horizon_pull.py $(if $(BRANCH),--branch "$(BRANCH)",)

# Printed by make itself: cmd.exe would treat the | and < in it as pipes and redirects.
define HELP
make setup [ARGS="--skip-packs --skip-build"]
                      Set up everything a machine can (same as ./setup.sh or setup.cmd); names what needs a person
make doctor           Report what is set up and what is missing; changes nothing
make all              Download and collect all three modpacks
make tekkit           Tekkit Classic 1.2.5 (Technic 3.1.2): server pack and client build (the art)
make direwolf20       Direwolf20 1.4.7 (FTB 5.3.2)
make agrarian         Agrarian Skies 2 1.7.10 (CurseForge 2.0.6a)
make list             Show pinned sources and versions
make clean            Remove ./assets and the download cache
make all FORCE=1      Redownload instead of using cached archives/mods
make extract-jar JAR=mod.jar [EXTRACT_OUT=./assets/extracted] [ARGS="--ext png,ogg"]
                      Extract assets (textures, audio, scripts) from a mod archive
make extract-assets [DIR=path/to/mods] [EXTRACT_OUT=./assets/extracted] [ARGS="..."]
                      Batch extract assets from all mod archives in a folder
make test-mods        Run cargo test across all Rust mod crates under ./mods/
make check-mods [MOD=example_mod]
                      Validate .hmod content and .hnmod native manifests/assets
make pack-mods [MOD=example_mod]
                      Package deterministic .hmod and .hnmod release bundles
make e2e-setup [MOD=a,b] [REV=pinned|submodule|latest|<sha>]
                      Fetch the horizon submodule, apply native mixins, build horizon-client (no game)
make e2e-mod [MOD=a,b] [REV=...] [DRY_RUN=1]
                      Phase 1: build, boot Horizon with one mod at a time, run all 7 action stages
make e2e-all-mods [MOD=a,b] [REV=...] [DRY_RUN=1]
                      Phase 2: boot Horizon with the selected mods (default: all) loaded together
make integration-test [MOD=a,b] [REV=...] [DRY_RUN=1]
                      Phase 1 + Phase 2; fails on any command error, game-log ERROR/panic or crash
make agents           Regenerate each agent tool's copy of .agents/ (Claude Code: .claude/); agents-check reports drift
make agent-gate [MOD=a,b]
                      Tests, check-mods and harness dry run for the mods/crates you changed (what the stop hooks run)
make fleet-next [AGENT=agy|claude|codex] [MOD=<id>] [FALLBACK=1]
                      One headless conductor run on one board row (AGENT_CMD="<cli flags>" for any other agent);
                      it relaunches the agent until the row is finished, on the model the tool is on, and stops
                      when that model hits a quota or rate limit (FALLBACK=1: on to the next of .agents/models.json)
make fleet-dispatch MOD=<id> [AGENT=<tool>] [MAX=2] [BUDGET=<tokens>] [FALLBACK=1] [CONDUCTOR=1]
                      The ticket loop without an LLM: a fresh headless agent per ready ticket (a role's first
                      alone), reports filed, tokens in target/fleet/usage.jsonl; exits 2 for the conductor, 3 for a person
make agy [ARGS="--agent conductor"]
                      Start agy here with its terminal sandbox off (the fleet's hooks still guard it)
make fleet-models [ARGS=--clear]
                      The model chain and which models are resting after a limit (--clear: none)
make fleet-start MOD=<id>
                      The conductor's first command: claims the mod, creates its Beads stories, checks Horizon
make fleet-stop       Tell every fleet agent to stop at the end of its current turn
make fleet-task BEAD=<id> ROLE=<role>
                      A role subagent's first command: its whole brief; it reports to target/fleet/inbox/<id>.json
make fleet-tickets MOD=<id> [CHECK=1] / fleet-tickets SCHEMA=1 / spec-merge MOD=<id>
                      File the spec-writers' tickets (CHECK=1: only validate) / their format / merge the part files
make fleet-inbox      File every subagent report in Beads (the conductor)
make fleet-features MOD=<id> / fleet-gaps MOD=<id> / fleet-status [MOD=<id>]
                      Features from the spec's list / New-assets features from the completeness check / progress
make board            Regenerate planning/fleet-board.md and .beads/issues.jsonl from Beads
make decompile JAR=<archive>
                      Original mod -> Java source in assets/decompiled/ (spec-writer and reviewer only)
make java-outline DIR=<decompiled> [GREP=<regex>] [ARGS=--classes|--consts]
                      Classes and members with line ranges (read this, not whole files)
make java-show DIR=<decompiled> MEMBER=<Class#method|Class:120-180> / java-find DIR=<d> TEXT=<name>
                      One member's source / where a name occurs
make version-sync [MOD=<id>] [ARGS=--check]
                      Changed mods/core one patch version ahead of their last commit (once; later work bundles in)
make core-need MOD=<id> TITLE="<what>" [WHY="<why it is shared>"]
                      Shared code the mod needs, filed under rekkit_core (conductor; done in this mod's run)
make tasks [MOD=<id>] / make watch [MOD=<id>]
                      The mod in progress on one screen: steps, what is worked, open items, bugs, subagents
                      (watch: redrawn every 30 s)
make board-live / make board-html
                      The board in your browser: every mod, and the current mod as a column per step
                      (live: http://127.0.0.1:8765, refreshes itself)
make tickets-live     Every ticket in a Jira-style page (http://127.0.0.1:8766): filter, search, read the wording
make fleet-resume MOD=<id>
                      Where the mod's run stands after a crash or lost credits: what to restart, what is waiting
make fleet-context MOD=<id>
                      One page: the mod's Beads state, spec, original archives in every pack, decompiled folders
make spec-toc MOD=<id> / spec-show MOD=<id> SECTION="<heading text>"
                      The spec's headings with line ranges / just the sections you need
make skill-toc SKILL=<name> / skill-show SKILL=<name> SECTION="<heading>" / conductor-step STEP=<n>
                      A skill's headings / one section of it / the conductor's instructions for one step
make content-gen MOD=<id> [NAME=a,b] [ARGS=--check|--force] / e2e-gen MOD=<id>
                      Blockstates, models, loot tables, lang, creative tab from the contract / a tests/e2e_actions.json draft
make contract-check MOD=<id> [OWNER=<role>] [VERBOSE=1] / contract-check SCHEMA=1
                      The port against its contract (40 lines; one role's; all) / the contract file's format
make token-audit [SAMPLES=s0-start,s3a-full] [N=2] [MODEL=<agy model>]
                      Token-audit samples on agy, compared with the baseline (planning/token_audit/)
make art MOD=<id>     mods/<id>/art/*.svg|*.pixels -> PNGs at the original size and palette
make check-mods MOD=<id> COMPLETE=1
                      Also fail on any asset Minecraft 26.3 needs that the mod lacks
make play [MOD=<id>]  Open Horizon in a window with the mod (default: all mods) and a showcase world
make horizon-baseline / horizon-status
                      Build and run Horizon with no mods at the pinned revision / show the result
make horizon-pull [BRANCH=main]
                      Force the horizon submodule to the remote's newest commit (yours to run)
make pull [BRANCH=main]
                      git pull this repository (fast-forward only), then make horizon-pull (yours to run)
CURSEFORGE_API_KEY is optional for Agrarian Skies 2: without a working key it falls back
                      to the public CurseForge download of pack 2.0.6 (file 3016703).
endef

help:
	$(info $(HELP))
	@cd .
