-include .env
export

PYTHON ?= python3
OUT ?= ./assets
CACHE ?= ./.cache/modpacks
FORCE ?= 0
JAR ?=
DIR ?=
EXTRACT_OUT ?= ./assets/extracted
ARGS ?=

export CURSEFORGE_API_KEY

.PHONY: all tekkit direwolf20 agrarian list clean help extract-jar extract-assets

all: tekkit direwolf20 agrarian

tekkit:
	mkdir -p $(OUT)
	$(PYTHON) scripts/fetch_modpacks.py tekkit --output "$(OUT)" --cache "$(CACHE)" $(if $(filter 1,$(FORCE)),--force,)

direwolf20:
	mkdir -p $(OUT)
	$(PYTHON) scripts/fetch_modpacks.py direwolf20 --output "$(OUT)" --cache "$(CACHE)" $(if $(filter 1,$(FORCE)),--force,)

agrarian:
	mkdir -p $(OUT)
	$(PYTHON) scripts/fetch_modpacks.py agrarian --output "$(OUT)" --cache "$(CACHE)" $(if $(filter 1,$(FORCE)),--force,)

list:
	$(PYTHON) scripts/fetch_modpacks.py list

clean:
	rm -rf "$(OUT)" "$(CACHE)"

extract-jar:
	$(PYTHON) scripts/extract_jar_assets.py $(if $(JAR),--jar "$(JAR)",) $(if $(EXTRACT_OUT),--output "$(EXTRACT_OUT)",) $(ARGS)

extract-assets:
	$(PYTHON) scripts/extract_jar_assets.py $(if $(JAR),--jar "$(JAR)",) $(if $(DIR),--dir "$(DIR)",) $(if $(EXTRACT_OUT),--output "$(EXTRACT_OUT)",) $(ARGS)

help:
	@echo 'make all              Download and collect all three modpacks'
	@echo 'make tekkit           Tekkit Classic 1.2.5 (Technic 3.1.2)'
	@echo 'make direwolf20       Direwolf20 1.4.7 (FTB 5.3.2)'
	@echo 'make agrarian         Agrarian Skies 2 1.7.10 (CurseForge 2.0.6a)'
	@echo 'make list             Show pinned sources and versions'
	@echo 'make clean            Remove ./assets and the download cache'
	@echo 'make all FORCE=1      Redownload instead of using cached archives/mods'
	@echo 'make extract-jar JAR=mod.jar [EXTRACT_OUT=./assets/extracted] [ARGS="--ext png,ogg"]'
	@echo '                      Extract assets (textures, audio, scripts) from a mod archive'
	@echo 'make extract-assets [DIR=path/to/mods] [EXTRACT_OUT=./assets/extracted] [ARGS="..."]'
	@echo '                      Batch extract assets from all mod archives in a folder'
	@echo 'CURSEFORGE_API_KEY is required for the Agrarian Skies download.'