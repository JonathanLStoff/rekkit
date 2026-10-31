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

export CURSEFORGE_API_KEY

.PHONY: setup doctor all tekkit direwolf20 agrarian list clean help extract-jar extract-assets test-mods check-mods pack-mods e2e-setup e2e-mod e2e-all-mods integration-test

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

# One target per crate (test-mod/mods/<id>/Cargo.toml), so no shell loop is needed.
test-mods: $(addprefix test-mod/,$(wildcard mods/*/Cargo.toml))

test-mod/%:
	$(info ==> Testing $*)
	cargo test --manifest-path "$*"

check-mods:
	$(PYTHON) scripts/pack_mod.py --check-only $(if $(MOD),--mod "$(MOD)",)

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

# Printed by make itself: cmd.exe would treat the | and < in it as pipes and redirects.
define HELP
make setup [ARGS="--skip-packs --skip-build"]
                      Set up everything a machine can (same as ./setup.sh or setup.cmd); names what needs a person
make doctor           Report what is set up and what is missing; changes nothing
make all              Download and collect all three modpacks
make tekkit           Tekkit Classic 1.2.5 (Technic 3.1.2)
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
CURSEFORGE_API_KEY is optional for Agrarian Skies 2: without a working key it falls back
                      to the public CurseForge download of pack 2.0.6 (file 3016703).
endef

help:
	$(info $(HELP))
	@cd .
