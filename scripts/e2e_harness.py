#!/usr/bin/env python3
"""
End-to-End Horizon Mod Integration & Compatibility Harness (planning/harness.md).

A live run does every step itself:
  1. Submodule: initialises ./horizon if needed and fetches it, so pinned revisions exist.
     The player's own checkout is never moved.
  2. Source: unpacks Horizon at the build revision into target/e2e/source/<rev12>
     (default: the checked-out horizon/ submodule; --horizon-rev pinned|latest|<sha>).
  3. Native mixins: builds horizon-mixins at the mods' pinned revision, applies every selected
     mod's native/mixins/<id>/ bundle, and stages the result in target/e2e/stage/<rev12>.
  4. Build: cargo build --profile fast of horizon-client and horizon-ctl (cached per mod set).
  5. Run: a scratch game home with the selected .hmod files turned on, horizon-ctl launch
     --offscreen, then every step of the 7-stage action plan through horizon-ctl.
  6. Check: any command error, wrong block, missing drop, ERROR/panic line in the game log or
     crash report fails the run. target/e2e-report/summary.json lists every step (--dry-run and
     --setup-only write setup.json instead).

Exit status: 0 every session passed, 1 a mod failed, 3 only Horizon's own problems (it builds badly or
logs the same lines with no mods: target/horizon-status.json, written by --baseline and by any live run
at a revision it has not checked).

Phases:
  1. isolated: one game per selected mod, only that mod loaded.
  2. all-mods: one game with every selected mod (default: all mods) loaded together.

Usage:
  python3 scripts/e2e_harness.py --dry-run
  python3 scripts/e2e_harness.py --setup-only                 # submodule + mixins + build, no game
  python3 scripts/e2e_harness.py --phase isolated --mod example_mod
  python3 scripts/e2e_harness.py --phase all-mods --mod a,b
  python3 scripts/e2e_harness.py --phase both --horizon-rev latest
"""

from __future__ import annotations

import argparse
import filecmp
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
import urllib.request
from pathlib import Path

import contract_check  # noqa: E402
import e2e_probes as probes
import pack_mod

ROOT = Path(__file__).resolve().parents[1]
HORIZON_ROOT = ROOT / "horizon"
MODS_ROOT = ROOT / "mods"
REPORT_DIR = ROOT / "target" / "e2e-report"
FAILURES_SHOWN = 25  # failures a live run prints (the rest are in summary.json)
E2E_ROOT = ROOT / "target" / "e2e"
MIXINS_GIT = "https://github.com/horizon-rust/horizon-mixins"
GAME_VERSION = "26.3"
EXE = ".exe" if os.name == "nt" else ""
# Resources carry the data version of the horizon-prepare that made them and the client refuses
# others, so each build revision gets its own, prepared inside target/e2e/source/<rev12>.
# Lines in the game log that fail a run: WARN/ERROR/FATAL log levels, panics, and messages that say
# a mod did not load. Only the patterns a person lists in ALLOWED_WARNINGS are accepted.
LOG_ERROR = re.compile(
    r"/WARN\]|/ERROR\]|/FATAL\]|panicked at|stack backtrace"
    r"|(?i:mods? not loaded|content not loaded|failed to load (?:content |native )?mod|mod .*(?:rejected|refused|invalid))"
)
ALLOWED_WARNINGS = ROOT / "planning" / "harness-allowed-warnings.txt"
# What Horizon itself does at a revision, with no mods (--baseline): decides whether a failure is
# Horizon's or the mod's.
HORIZON_STATUS = ROOT / "target" / "horizon-status.json"
# Blockstate variants placed per block, at most (a multipart block can have thousands).
MAX_VARIANTS = 32


class HarnessError(Exception):
    pass


def discover_mod_spec(mod_dir: Path) -> dict:
    """Inspect a mod's content/ and native/ directories and build its 7-stage action matrix."""
    mod_id = mod_dir.name
    content_dir = mod_dir / "content"
    native_dir = mod_dir / "native"

    manifest = tomllib.loads((content_dir / "horizon-mod.toml").read_text(encoding="utf-8"))
    native_json = json.loads((native_dir / "horizon-native.json").read_text(encoding="utf-8"))

    blocks = []
    block_dir = content_dir / "data" / mod_id / "horizon" / "block"
    if block_dir.is_dir():
        for p in sorted(block_dir.glob("*.json")):
            bdata = json.loads(p.read_text(encoding="utf-8"))
            states = content_dir / "assets" / mod_id / "blockstates" / f"{p.stem}.json"
            blocks.append(
                {
                    "id": f"{mod_id}:{p.stem}",
                    "name": p.stem,
                    "template": bdata["template"],
                    "has_item": bdata.get("item", True),
                    "properties": bdata.get("properties", {}),
                    "variants": blockstate_variants(states),
                }
            )

    items = []
    item_dir = content_dir / "data" / mod_id / "horizon" / "item"
    if item_dir.is_dir():
        for p in sorted(item_dir.glob("*.json")):
            idata = json.loads(p.read_text(encoding="utf-8"))
            items.append(
                {
                    "id": f"{mod_id}:{p.stem}",
                    "name": p.stem,
                    "template": idata["template"],
                    "max_stack_size": idata.get("max_stack_size", 64),
                }
            )

    recipes = []
    recipe_dir = content_dir / "data" / mod_id / "recipe"
    if recipe_dir.is_dir():
        for p in sorted(recipe_dir.glob("*.json")):
            rdata = json.loads(p.read_text(encoding="utf-8"))
            recipes.append(
                {
                    "id": f"{mod_id}:{p.stem}",
                    "type": rdata.get("type"),
                    "result": rdata.get("result"),
                }
            )

    loot_tables = []
    loot_drops = {}
    loot_dir = content_dir / "data" / mod_id / "loot_table" / "blocks"
    if loot_dir.is_dir():
        for p in sorted(loot_dir.glob("*.json")):
            loot_tables.append(f"{mod_id}:blocks/{p.stem}")
            try:
                ldata = json.loads(p.read_text(encoding="utf-8"))
                drops = []
                for pool in ldata.get("pools", []):
                    for entry in pool.get("entries", []):
                        if entry.get("type") == "minecraft:item" and "name" in entry:
                            drops.append(entry["name"])
                loot_drops[f"{mod_id}:blocks/{p.stem}"] = drops
            except Exception:
                pass

    mob_effects = []
    effect_dir = content_dir / "data" / mod_id / "horizon" / "mob_effect"
    if effect_dir.is_dir():
        for p in sorted(effect_dir.glob("*.json")):
            edata = json.loads(p.read_text(encoding="utf-8"))
            mob_effects.append(
                {
                    "id": f"{mod_id}:{p.stem}",
                    "name": p.stem,
                    "category": edata["category"],
                    "color": edata["color"],
                }
            )

    creative_tabs = []
    tab_dir = content_dir / "data" / mod_id / "horizon" / "creative_tab"
    if tab_dir.is_dir():
        for p in sorted(tab_dir.glob("*.json")):
            tdata = json.loads(p.read_text(encoding="utf-8"))
            creative_tabs.append({"id": f"{mod_id}:{p.stem}", "title": tdata.get("title", ""),
                                  "icon": tdata.get("icon", ""), "items": tdata.get("items", [])})
    lang_file = content_dir / "assets" / mod_id / "lang" / "en_us.json"
    lang = json.loads(lang_file.read_text(encoding="utf-8")) if lang_file.is_file() else {}
    for tab in creative_tabs:  # the title the tab's button shows
        name = tab["id"].split(":", 1)[1]
        tab["label"] = lang.get(f"itemGroup.{mod_id}.{name}") or lang.get(tab["title"]) or tab["title"]

    # The contract (the outline and each block's planning/mods/<id>.contract/<name>.json, from the
    # original's code): each block's shape and GUI, and the scenarios testing its attributes, checked
    # live below (scripts/contract_check.py checks the rest).
    contract = contract_check.load_contract(mod_id) or {}
    by_name = {c["id"]: c for c in contract.get("blocks", []) if isinstance(c, dict) and "id" in c}
    for b in blocks:
        b["contract"] = by_name.get(b["name"], {})
    items_by_name = {c["id"]: c for c in contract.get("items", []) if isinstance(c, dict) and "id" in c}
    for it in items:
        it["contract"] = items_by_name.get(it["name"], {})

    namespaces = [mod_id, *manifest.get("namespaces", [])]
    extra = probes.discover(content_dir, mod_id, namespaces, {b["id"] for b in blocks}, contract)
    # The test-smith's tests/e2e_actions.json (make e2e-gen drafts it): trigger items, the interactions
    # that must change a block's property, and the neighbours placed around a block.
    actions = {}
    try:
        actions = json.loads((mod_dir / "tests" / "e2e_actions.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        pass
    extra["actions"] = actions
    return {**extra,
        "id": mod_id,
        "version": manifest["version"],
        "namespaces": namespaces,
        "horizon_revision": native_json["horizon"],
        "mixins_revision": native_json["mixins"],
        "hooks_api": native_json.get("hooks"),
        "blocks": blocks,
        "items": items,
        "creative_tabs": creative_tabs,
        # Every item and block item, which must be in one of the mod's creative tabs.
        "creative_expected": [b["id"] for b in blocks if b["has_item"]] + [i["id"] for i in items],
        "mob_effects": mob_effects,
        "recipes": recipes,
        "loot_tables": loot_tables,
        "loot_drops": loot_drops,
        "scenarios": contract.get("scenarios", []),
    }


def blockstate_variants(path: Path) -> list[str]:
    """Every state a blockstate file names, as "key=value,...": the variants' keys, or for a
    multipart file the combinations of the values its conditions name (at most MAX_VARIANTS)."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return []
    if isinstance(data.get("variants"), dict):
        return [k for k in data["variants"] if k][:MAX_VARIANTS]
    values: dict[str, set[str]] = {}

    def conditions(node: object) -> None:
        if isinstance(node, dict):
            for key, value in node.items():
                if key in ("OR", "AND") and isinstance(value, list):
                    for item in value:
                        conditions(item)
                elif isinstance(value, (str, bool, int)):
                    values.setdefault(key, set()).update(str(value).lower().split("|"))
    for part in data.get("multipart", []):
        conditions(part.get("when", {}))
    combos = [""]
    for key in sorted(values):
        combos = [f"{c},{key}={v}".lstrip(",") for c in combos for v in sorted(values[key])][:MAX_VARIANTS]
    return [c for c in combos if c]


def tp_step(stage: str, x: float, y: float, z: float) -> dict:
    """A teleport by the server's own /tp. horizon-ctl's tp moves the client, and the server then logs
    "moved wrongly" when that path crosses a block (as in survival, from one row of the plot to another)."""
    return {"stage": stage, "cmd": "command", "args": {"command": f"tp @s {x} {y} {z}"}}


def build_action_plan(spec: dict, plot_index: int = 0) -> list[dict]:
    """Generate the 7-stage exhaustive action sequence for a mod at its assigned world plot."""
    mod_id = spec["id"]
    # Each mod's plot is its own row, 16 blocks along z from the last: a plot runs 2 blocks per block
    # along x (RedPower's ~240), so side by side on x they would overlap.
    base_x = 0
    # A floating plot: high enough to clear terrain, cleared to air, on a stone floor the player
    # can stand on in survival.
    base_y = 100
    base_z = plot_index * 16
    plot_x2 = base_x + max(2 * len(spec["blocks"]), 4)
    steps: list[dict] = []

    # Stage 1: Registry, Item Spawn & Custom Mob Effect Application
    for b in spec["blocks"]:
        if b["has_item"]:
            steps.append({"stage": "1_give_block_item", "cmd": "give", "args": {"item": b["id"], "count": 1}})
    for it in spec["items"]:
        steps.append({"stage": "1_give_item", "cmd": "give", "args": {"item": it["id"], "count": 1}})
    for eff in spec.get("mob_effects", []):
        steps.append(
            {
                "stage": "1_apply_mob_effect",
                "cmd": "command",
                "args": {"command": f"effect give @s {eff['id']} 5 0"},
            }
        )
        steps.append(
            {
                "stage": "1_clear_mob_effect",
                "cmd": "command",
                "args": {"command": f"effect clear @s {eff['id']}"},
            }
        )
    # Every item: stack size, and armor and held attributes (worn armor's protection, a tool's damage).
    steps += probes.item_steps(spec)
    steps.append({"stage": "1_clear_inventory", "cmd": "command", "args": {"command": "clear"}})
    steps.append({"stage": "1_clear_sync", "cmd": "wait", "args": {"ticks": 2}})

    # Stage 2: World Grid Placement
    # Commands land a few ticks after they reply, so every world change is followed by a wait.
    # Row base_z: placed with /setblock. Row base_z + 4: placed by the player, then every variant.
    # Keep the whole plot loaded: past the spawn chunks a fill or setblock lands nowhere, the floor is
    # missing and the player falls (forceload takes at most 256 chunks a call).
    for x0 in range(base_x - 2, plot_x2 + 1, 1024):
        steps.append({"stage": "2_forceload_plot", "cmd": "command",
                      "args": {"command": f"forceload add {x0} {base_z - 4} {min(x0 + 1023, plot_x2)} {base_z + 13}"}})
    steps.append({"stage": "2_plot_loads", "cmd": "wait", "args": {"ticks": 100}})
    steps.append({"stage": "2_clear_plot", "cmd": "fill",
                  "args": {"box": [base_x - 2, base_y, base_z - 4, plot_x2, base_y + 4, base_z + 6], "block": "minecraft:air"}})
    steps.append({"stage": "2_floor_plot", "cmd": "fill",
                  "args": {"box": [base_x - 2, base_y - 1, base_z - 4, plot_x2, base_y - 1, base_z + 6], "block": "minecraft:stone"}})
    steps.append({"stage": "2_peaceful", "cmd": "command", "args": {"command": "difficulty peaceful"}})
    steps.append({"stage": "2_day", "cmd": "command", "args": {"command": "time set day"}})
    steps.append({"stage": "2_no_daylight", "cmd": "command", "args": {"command": "gamerule doDaylightCycle false"}})
    steps.append({"stage": "2_no_mob_spawning", "cmd": "command", "args": {"command": "gamerule doMobSpawning false"}})
    steps.append({"stage": "2_plot_ready", "cmd": "wait", "args": {"ticks": 10}})
    steps.append(
        tp_step("2_teleport_plot", base_x + 0.5, base_y, base_z - 2.5)
    )
    steps.append({"stage": "2_settle", "cmd": "wait", "args": {"ticks": 5}})
    for idx, b in enumerate(spec["blocks"]):
        first = len(steps)
        bx = base_x + (idx * 2)
        bz = base_z
        steps.append(tp_step("2_stand_by_block", bx + 0.5, base_y, bz - 1.5))
        # Ensure the local plot column has a solid stone floor even if the chunk was unloaded at world start
        steps.append({"stage": "2_column_floor", "cmd": "fill",
                      "args": {"box": [bx - 1, base_y - 1, base_z - 4, bx + 1, base_y - 1, base_z + 6], "block": "minecraft:stone"}})
        steps.append({"stage": "2_column_clear", "cmd": "fill",
                      "args": {"box": [bx - 1, base_y, base_z - 4, bx + 1, base_y + 4, base_z + 6], "block": "minecraft:air"}})
        if b["template"] == "minecraft:wheat":
            steps.append({"stage": "2_support_floor", "cmd": "setblock", "args": {"x": bx, "y": base_y - 1, "z": bz, "block": "minecraft:farmland[moisture=7]"}})
            steps.append({"stage": "2_support_water", "cmd": "setblock", "args": {"x": bx, "y": base_y - 1, "z": bz - 1, "block": "minecraft:water"}})
        elif "sapling" in b["template"] or b["template"] == "minecraft:poppy":
            steps.append({"stage": "2_support_floor", "cmd": "setblock", "args": {"x": bx, "y": base_y - 1, "z": bz, "block": "minecraft:grass_block"}})
        block_to_place = f"{b['id']}[persistent=true]" if "leaves" in b["template"] else b["id"]
        # The empty spot, seen from where the block will be looked at: the render check compares the two.
        steps += [{"stage": "2_render_aim_empty", "cmd": "look_at", "args": {"x": bx + 0.5, "y": base_y + 0.5, "z": bz + 0.5}},
                  {"stage": "2_render_empty", "cmd": "screenshot", "args": {"path": f"{mod_id}_{b['name']}_empty.png"}}]
        steps.append(
            {"stage": "2_place_mod_block", "cmd": "setblock", "args": {"x": bx, "y": base_y, "z": bz, "block": block_to_place}}
        )
        steps.append({"stage": "2_block_lands", "cmd": "wait", "args": {"ticks": 5}})
        steps.append(
            {
                "stage": "2_verify_block",
                "cmd": "block",
                "args": {"x": bx, "y": base_y, "z": bz},
                "expect_block": b["id"],
            }
        )
        contract = b.get("contract", {})
        steps += [{"stage": "2_render_aim", "cmd": "look_at_block", "args": {"x": bx, "y": base_y, "z": bz}},
                  {"stage": "2_render", "cmd": "screenshot", "args": {"path": f"{mod_id}_{b['name']}_world.png"},
                   "render_check": f"{mod_id}_{b['name']}", "block": b["id"]}]
        steps += shape_probe(contract.get("shape"), bx, base_y, bz)
        pz = base_z + 4
        if b["has_item"]:
            # As a player places it: hold the item, look at the floor in front, right-click. From 1.5
            # blocks back the ray meets the floor block's top face, so the block lands at (bx, base_y, pz).
            floor_block = ("minecraft:farmland" if b["template"] == "minecraft:wheat"
                           else "minecraft:grass_block" if "sapling" in b["template"] or b["template"] == "minecraft:poppy"
                           else "minecraft:stone")
            steps += [
                {"stage": "2_player_floor", "cmd": "setblock", "args": {"x": bx, "y": base_y - 1, "z": pz, "block": floor_block}},
                {"stage": "2_player_clear", "cmd": "command", "args": {"command": "clear"}},
                {"stage": "2_player_give", "cmd": "give", "args": {"item": b["id"], "count": 1}},
                {"stage": "2_player_sync", "cmd": "wait", "args": {"ticks": 6}},
                {"stage": "2_player_select", "cmd": "select", "args": {"slot": b["id"]}},
                tp_step("2_player_stand", bx + 0.5, base_y, pz - 0.6),
                {"stage": "2_player_settle", "cmd": "wait", "args": {"ticks": 5}},
                {"stage": "2_player_aim", "cmd": "look_at_block", "args": {"x": bx, "y": base_y - 1, "z": pz}},
                {"stage": "2_player_place", "cmd": "mouse", "args": {"button": "right", "action": "click"}},
                {"stage": "2_player_lands", "cmd": "wait", "args": {"ticks": 5}},
                {"stage": "2_player_verify", "cmd": "block", "args": {"x": bx, "y": base_y, "z": pz},
                 "expect_block": b["id"], "what": "placed by a player"},
                {"stage": "2_player_restore_floor", "cmd": "setblock", "args": {"x": bx, "y": base_y - 1, "z": pz, "block": "minecraft:stone"}},
            ]
        for variant in b.get("variants", []):
            steps += [
                {"stage": "2_variant_place", "cmd": "setblock",
                 "args": {"x": bx, "y": base_y, "z": pz, "block": f"{b['id']}[{variant}]"}},
                {"stage": "2_variant_lands", "cmd": "wait", "args": {"ticks": 2}},
                {"stage": "2_variant_verify", "cmd": "block", "args": {"x": bx, "y": base_y, "z": pz},
                 "expect_block": b["id"], "what": f"variant [{variant}]"},
            ]
        steps.append({"stage": "2_player_row_clear", "cmd": "setblock",
                      "args": {"x": bx, "y": base_y, "z": pz, "block": "minecraft:air"}})
        steps.append(tp_step("2_player_return", bx + 0.5, base_y, bz - 1.5))
        steps.append({"stage": "2_player_returns", "cmd": "wait", "args": {"ticks": 5}})

        # Stage 3: Item-on-Block & Right-Click Interactions: the stock triggers, then each of
        # e2e_actions.json's interactions on this block, whose property must change.
        actions = spec.get("actions") or {}
        targeted = [(x["item"], x.get("expected_property")) for x in actions.get("stage_3_interactions", [])
                    if b["id"] in x.get("target_blocks", []) and not x.get("draft")]
        for trigger_item, prop in [("minecraft:orange_dye", None), ("minecraft:diamond", None), *targeted]:
            if prop:
                steps.append({"stage": "3_state_before", "cmd": "block", "args": {"x": bx, "y": base_y, "z": bz},
                              "save": {"name": f"state_{prop}", "state_prop": prop}})
            steps.append(tp_step("3_player_stand", bx + 0.5, base_y, bz - 1.5))
            steps.append({"stage": "3_player_settle", "cmd": "wait", "args": {"ticks": 2}})
            steps.append({"stage": "3_clear_trigger", "cmd": "command", "args": {"command": "clear"}})
            steps.append({"stage": "3_give_trigger", "cmd": "give", "args": {"item": trigger_item, "count": 1}})
            steps.append({"stage": "3_sync_inventory", "cmd": "wait", "args": {"ticks": 2}})
            steps.append({"stage": "3_select_trigger", "cmd": "select", "args": {"slot": trigger_item}})
            steps.append(
                {"stage": "3_look_at_block", "cmd": "look_at_block", "args": {"x": bx, "y": base_y, "z": bz}}
            )
            steps.append({"stage": "3_use_item_on", "cmd": "mouse", "args": {"button": "right", "action": "click"}})
            steps.append({"stage": "3_wait_use", "cmd": "wait", "args": {"ticks": 5}})
            if prop:
                steps.append({"stage": "3_state_changed", "cmd": "block", "args": {"x": bx, "y": base_y, "z": bz},
                              "check": {"type": "prop_changed", "prop": prop},
                              "what": f"using {trigger_item} on it changes its {prop} (e2e_actions.json)"})
            steps.append({"stage": "3_close_after_use", "cmd": "close", "args": {}})

        # Stage 4: Container & Custom GUI Screen Actions
        steps.append(tp_step("4_player_stand", bx + 0.5, base_y, bz - 1.5))
        steps.append({"stage": "4_player_settle", "cmd": "wait", "args": {"ticks": 2}})
        steps.append({"stage": "4_clear_hand", "cmd": "command", "args": {"command": "clear"}})
        steps.append(
            {"stage": "4_open_container", "cmd": "look_at_block", "args": {"x": bx, "y": base_y, "z": bz}}
        )
        steps.append({"stage": "4_right_click_open", "cmd": "mouse", "args": {"button": "right", "action": "click"}})
        steps.append({"stage": "4_wait_ui", "cmd": "wait", "args": {"ticks": 5}})
        if isinstance(contract.get("gui"), dict):
            # The original's GUI: its slots where the original's Container puts them, not a stock grid.
            steps.append({"stage": "4_gui_layout", "cmd": "ui", "args": {}, "expect_gui": contract["gui"],
                          "what": "right-clicked"})
        elif "gui" in contract:
            steps.append({"stage": "4_gui_none", "cmd": "state", "args": {}, "expect_no_menu": True,
                          "what": "right-clicked, a block the original gives no GUI"})
        elif MENU_TEMPLATE.fullmatch(b["template"]):
            # Its template opens a menu (interact.rs OpenMenu): without the mod's own menu the player
            # gets "<block> is not available yet" and nothing opens.
            steps.append({"stage": "4_menu_opens", "cmd": "state", "args": {}, "expect_screen": ["container", "station"],
                          "what": f"right-clicked, a block on the {b['template']} template"})
        steps.append(
            {
                "stage": "4_screenshot_gui",
                "cmd": "screenshot",
                "args": {"path": f"{mod_id}_{b['name']}_gui.png"},
            }
        )
        steps.append({"stage": "4_close_container", "cmd": "close", "args": {}, "allow_kinds": ["state"]})

        # Stage 5: Server Ticks & Neighbor/Hopper Automation
        steps.append(
            {
                "stage": "5_place_hopper",
                "cmd": "setblock",
                "args": {"x": bx, "y": base_y + 1, "z": bz, "block": "minecraft:hopper[facing=down]"},
            }
        )
        sides = {"top": (0, 1, 0), "bottom": (0, -1, 0), "north": (0, 0, -1), "south": (0, 0, 1),
                 "east": (1, 0, 0), "west": (-1, 0, 0)}
        placed = []
        for n in actions.get("stage_5_neighbors", []):
            if n.get("center") != b["id"] or n.get("draft"):
                continue
            for side, (dx, dy, dz) in sides.items():
                if n.get(side) and side != "top":
                    pos = {"x": bx + dx, "y": base_y + dy, "z": bz + dz}
                    steps.append({"stage": "5_place_neighbor", "cmd": "setblock", "args": {**pos, "block": n[side]}})
                    placed.append((pos, "minecraft:stone" if side == "bottom" else "minecraft:air"))
        steps.append({"stage": "5_tick_simulation", "cmd": "wait", "args": {"ticks": 20}})
        for pos, restore in placed:
            steps.append({"stage": "5_remove_neighbor", "cmd": "setblock", "args": {**pos, "block": restore}})
        steps.append(
            {
                "stage": "5_remove_hopper",
                "cmd": "setblock",
                "args": {"x": bx, "y": base_y + 1, "z": bz, "block": "minecraft:air"},
            }
        )
        for step in steps[first:]:
            step["block"] = b["id"]

    # Stage 7: Survival Block Break & Loot Drop Verification
    steps.append({"stage": "7_clear_inventory", "cmd": "command", "args": {"command": "clear"}})
    steps.append({"stage": "7_resistance", "cmd": "command", "args": {"command": "effect give @s resistance 99999 255 true"}})
    steps.append({"stage": "7_saturation", "cmd": "command", "args": {"command": "effect give @s saturation 99999 255 true"}})
    steps.append({"stage": "7_gamemode_survival", "cmd": "command", "args": {"command": "gamemode survival"}})
    steps.append(
        {"stage": "7_give_pickaxe", "cmd": "give", "args": {"item": "minecraft:netherite_pickaxe", "count": 1}}
    )
    steps.append({"stage": "7_sync_inventory", "cmd": "wait", "args": {"ticks": 10}})
    steps.append({"stage": "7_select_pickaxe", "cmd": "select", "args": {"slot": "minecraft:netherite_pickaxe"}})
    loot = set(spec["loot_tables"])
    for idx, b in enumerate(spec["blocks"]):
        bx = base_x + (idx * 2)
        bz = base_z
        steps.append(tp_step("7_stand_by_block", bx + 0.5, base_y, bz - 1.5))
        steps.append({"stage": "7_settle", "cmd": "wait", "args": {"ticks": 5}})
        # `break` reports what the block dropped (the player may pick it up a moment later).
        brk = {"stage": "7_break_block", "cmd": "break", "args": {"x": bx, "y": base_y, "z": bz}}
        loot_table_id = f"{mod_id}:blocks/{b['name']}"
        expected_drops = spec.get("loot_drops", {}).get(loot_table_id, [])
        if expected_drops:
            brk["expect_drop"] = expected_drops[0]
        elif loot_table_id in loot and b["has_item"]:
            brk["expect_drop"] = b["id"]
        steps.append(brk)
        steps.append({"stage": "7_drops_spawn", "cmd": "wait", "args": {"ticks": 10}})
        steps.append({"stage": "7_list_drops", "cmd": "entities", "args": {"radius": 8, "type": "item"}})
        for step in steps[-5:]:
            step["block"] = b["id"]
    steps.append({"stage": "7_restore_creative", "cmd": "command", "args": {"command": "gamemode creative"}})
    steps.append({"stage": "7_clear_effects", "cmd": "command", "args": {"command": "effect clear @s"}})

    # Stage 6: every crafting, smelting and smithing recipe made in its station, its result checked.
    steps += probes.recipe_steps(spec, spec.get("all_tags", spec.get("tags", {})), base_x, base_y, base_z)

    # Stage 11: every entity type summoned, drawn, its attributes and drops. Stage 12: every feature,
    # structure and template placed and its blocks found. Each on its own far-off row.
    steps += probes.entity_steps(spec, plot_index)
    steps += probes.worldgen_steps(spec, plot_index)

    # Stage 10: the contract's behaviour scenarios (what the original does: a solar panel at noon charges,
    # a voltmeter reads it, quantum boots jump higher), each on its own patch of the mod's scenario row.
    ox = base_x
    for scenario in spec.get("scenarios", []):
        width, depth, _ = probes.scenario_area(scenario)
        steps += scenario_steps(scenario, ox, base_y, 1024 + plot_index * 128, depth)
        ox += width + 4
    return steps


def shape_probe(shape, bx: int, y: int, bz: int) -> list[dict]:
    """Aim where the original's shape (its default state's outline, in pixels) is or is not, and ask
    what the crosshair meets. A full block must be hit near the top of the face the player sees. A low
    one (a gate, a panel, a wire) is aimed at just above its top at the far side of its cell: the ray
    comes down from the eye above the shape and leaves the cell there, so it must meet something else."""
    if shape == "full":
        point, hit = (bx + 0.5, y + 0.85, bz + 0.02), True
    elif isinstance(shape, list) and len(shape) == 6 and shape[4] / 16 + 0.12 < 0.95:
        point, hit = (bx + 0.5, y + shape[4] / 16 + 0.12, bz + 0.97), False
    else:
        return []
    return [{"stage": "2_shape_aim", "cmd": "look_at", "args": {"x": point[0], "y": point[1], "z": point[2]}},
            {"stage": "2_shape_settle", "cmd": "wait", "args": {"ticks": 2}},
            {"stage": "2_shape_probe", "cmd": "looking_at", "args": {}, "expect_hit": [[bx, y, bz], hit],
             "what": f"the original's shape is {shape}"}]


def scenario_steps(scenario: dict, ox: int, oy: int, oz: int, depth: int = 6) -> list[dict]:
    """A contract scenario's steps at origin (ox, oy, oz), on a cleared stone patch. Its steps:
    {"setblock": [dx, dy, dz, block]}, {"command": text}, {"wait": ticks},
    {"use": item, "on": [dx, dy, dz]} (right-click the block holding item), {"interact": [dx, dy, dz]},
    {"expect_message": regex} (chat or action bar), {"expect_block": [dx, dy, dz, regex]} (its state),
    {"expect_data": [dx, dy, dz, regex]} (`data get block`),
    {"ctl": [words...], "at": [dx, dy, dz], "expect": regex} (any horizon-ctl command, "{x}" "{y}" "{z}"
    in its words replaced by the position; the regex is matched against its JSON reply),
    {"reload_chunk": true} (forget the patch's chunk and load it again from disk: level 5, persistence),
    and the player, entity, inventory, worldgen and measuring steps of e2e_probes.scenario_step
    (planning/harness.md, "Scenario steps"). Each scenario starts in creative, with an empty inventory and
    no effects."""
    name = scenario.get("name", "scenario")
    tag = {"scenario": name, "for": scenario.get("for", []), "what": f"scenario {name!r}"}
    far = oz + max(3, depth - 3)
    width, _, height = probes.scenario_area(scenario)
    out = [{"stage": "10_scenario_pin", "cmd": "command", "args": {"command": f"forceload add {ox - 2} {oz - 2} {ox + width} {far}"}},
           tp_step("10_scenario_go", ox + 0.5, oy, oz - 1.5), {"stage": "10_scenario_loads", "cmd": "wait", "args": {"ticks": 40}},
           {"stage": "10_scenario_reset", "cmd": "command", "args": {"command": "gamemode creative"}},
           {"stage": "10_scenario_reset", "cmd": "command", "args": {"command": "effect clear @s"}},
           {"stage": "10_scenario_reset", "cmd": "command", "args": {"command": "clear"}},
           {"stage": "10_scenario_reset", "cmd": "command", "args": {"command": "difficulty peaceful"}},
           {"stage": "10_scenario_floor", "cmd": "fill", "args": {"box": [ox - 2, oy - 1, oz - 2, ox + width - 3, oy - 1, far], "block": "minecraft:stone"}},
           {"stage": "10_scenario_clear", "cmd": "fill", "args": {"box": [ox - 2, oy, oz - 2, ox + width - 3, oy + height, far], "block": "minecraft:air"}},
           tp_step("10_scenario_stand", ox + 0.5, oy, oz - 1.5), {"stage": "10_scenario_settle", "cmd": "wait", "args": {"ticks": 5}}]
    stand = (ox + 0.5, oy, oz - 1.5)
    ctx = {"oy": oy, "bounds": (ox - 2, oz - 2, ox + width - 3, far), "occupied": set(), "stand": stand}

    def at(rel):
        return {"x": ox + rel[0], "y": oy + rel[1], "z": oz + rel[2]}
    tag["level"] = scenario.get("level")
    for st in scenario.get("steps", []):
        first = len(out)
        if "ctl" in st:
            pos = at(st.get("at", [0, 0, 0]))
            words = [str(w).format(**pos) if isinstance(w, str) else w for w in st["ctl"]]
            step = {"stage": "10_scenario_ctl", "cmd": "raw", "args": {"words": words}}
            if st.get("expect"):
                step.update({"expect_json": st["expect"], **tag})
            out.append(step)
        elif st.get("reload_chunk"):
            # Out of range and back: the chunk is saved, dropped and read again from the region file.
            out += [{"stage": "10_scenario_save", "cmd": "command", "args": {"command": "save-all flush"}},
                    {"stage": "10_scenario_unpin", "cmd": "command", "args": {"command": "forceload remove all"}},
                    tp_step("10_scenario_away", ox + 4000.5, oy + 50, oz + 0.5),
                    {"stage": "10_scenario_unload", "cmd": "wait", "args": {"ticks": 200}},
                    tp_step("10_scenario_back", ox + 0.5, oy, oz - 1.5),
                    {"stage": "10_scenario_pin", "cmd": "command",
                     "args": {"command": f"forceload add {ox - 2} {oz - 2} {ox + 3} {oz + 3}"}},
                    {"stage": "10_scenario_reload", "cmd": "wait", "args": {"ticks": 60}}]
        elif "setblock" in st:
            out += [{"stage": "10_scenario_setblock", "cmd": "setblock", "args": {**at(st["setblock"]), "block": st["setblock"][3]}},
                    {"stage": "10_scenario_lands", "cmd": "wait", "args": {"ticks": 5}}]
        elif "command" in st:
            out.append({"stage": "10_scenario_command", "cmd": "command", "args": {"command": st["command"]}})
        elif "wait" in st:
            out.append({"stage": "10_scenario_wait", "cmd": "wait", "args": {"ticks": int(st["wait"])}})
        elif "use" in st or "interact" in st:
            pos = at(st.get("on") or st.get("interact"))
            out += [{"stage": "10_scenario_hand", "cmd": "command", "args": {"command": "clear"}}]
            if "use" in st:
                out += [{"stage": "10_scenario_give", "cmd": "give", "args": {"item": st["use"], "count": 1}},
                        {"stage": "10_scenario_sync", "cmd": "wait", "args": {"ticks": 4}},
                        {"stage": "10_scenario_select", "cmd": "select", "args": {"slot": st["use"]}}]
            out += [{"stage": "10_scenario_aim", "cmd": "look_at_block", "args": pos},
                    {"stage": "10_scenario_click", "cmd": "mouse", "args": {"button": "right", "action": "click"}},
                    {"stage": "10_scenario_after", "cmd": "wait", "args": {"ticks": 5}}]
        elif "expect_message" in st:
            out.append({"stage": "10_scenario_expect", "cmd": "messages", "args": {"n": 10}, "expect_match": st["expect_message"], **tag})
        elif "expect_block" in st:
            out.append({"stage": "10_scenario_expect", "cmd": "block", "args": at(st["expect_block"]), "expect_match": st["expect_block"][3], **tag})
        elif "expect_data" in st:
            out.append({"stage": "10_scenario_expect", "cmd": "command", "args": {"command": "data get block {x} {y} {z}".format(**at(st["expect_data"]))},
                        "expect_match": st["expect_data"][3], **tag})
        else:
            try:
                extra = probes.scenario_step(st, at, stand, ctx)
            except (KeyError, ValueError, TypeError) as exc:
                raise HarnessError(f"scenario {name!r}: bad step {json.dumps(st)[:120]}: {exc}") from exc
            if extra is None:
                raise HarnessError(f"scenario {name!r}: unknown step {json.dumps(st)[:120]} (planning/harness.md, Scenario steps)")
            for step in extra:
                if "check" in step:
                    step.update(tag)
            out += extra
        # "save" on a ctl or command step keeps a number of its reply ("regex" or "path").
        if isinstance(st.get("save"), (dict, str)) and ("ctl" in st or "command" in st):
            s = st["save"] if isinstance(st["save"], dict) else {"name": st["save"], "regex": probes.NUMBER}
            for step in reversed(out[first:]):
                if step.get("cmd") in ("raw", "command"):
                    step["save"] = s
                    break
    out.append({"stage": "10_scenario_close", "cmd": "close", "args": {}, "allow_kinds": ["state"]})
    # Whatever it summoned (a wither from its altar, a ritual's mob) goes before the next scenario.
    out.append({"stage": "10_scenario_cleanup", "cmd": "command",
                "args": {"command": f"kill @e[type=!player,x={ox - 2},y={oy - 2},z={oz - 2},dx={width + 2},dy={height + 8},dz={far - oz + 4}]"},
                "allow_kinds": ["usage", "state"]})
    out.append({"stage": "10_scenario_reset", "cmd": "command", "args": {"command": "difficulty peaceful"}})
    out.append({"stage": "10_scenario_unpin", "cmd": "command", "args": {"command": f"forceload remove {ox - 2} {oz - 2} {ox + width} {far}"},
                "allow_kinds": ["usage", "state"]})
    for step in out:
        step.setdefault("scenario", name)
        step.setdefault("for", tag["for"])
        step.setdefault("level", tag["level"])
        step.setdefault("what", tag["what"])
    return out


def gui_problem(gui: dict, ui: dict) -> str | None:
    """How the open screen differs from the original's GUI: its container slots (300 on) where the
    original's Container puts them, relative to a screen of the original's size, centred as Minecraft
    centres it."""
    want = sorted(tuple(s) for s in gui.get("slots", []))
    if ui.get("screen") not in ("container", "station"):
        return f"it opened {ui.get('screen')!r}; the original opens its GUI ({len(want)} slots)"
    width, height = gui.get("size", [176, 166])
    left = int((ui.get("gui_size", [0, 0])[0] - width) // 2)
    top = int((ui.get("gui_size", [0, 0])[1] - height) // 2)
    got = sorted((round(w["bounds"][0] - left), round(w["bounds"][1] - top)) for w in ui.get("widgets", [])
                 if w.get("kind") == "slot" and w.get("id", "")[5:].isdigit() and int(w["id"][5:]) >= 300)
    if len(got) != len(want):
        return f"its screen has {len(got)} container slots; the original's GUI has {len(want)} (a stock grid?)"
    off = [(g, w) for g, w in zip(got, want) if abs(g[0] - w[0]) > 1 or abs(g[1] - w[1]) > 1]
    if off:
        return (f"{len(off)} slot(s) are not where the original's are (first at {list(off[0][0])}, the original's "
                f"at {list(off[0][1])}): give it the original's layout (native_screen), not a stock grid")
    return None


def check_cross_mod_compatibility(specs: list[dict]) -> None:
    """Verify that all mods can be loaded together without namespace, tag, or revision conflicts."""
    seen_namespaces: dict[str, str] = {}
    revisions: set[str] = set()

    for spec in specs:
        mod_id = spec["id"]
        revisions.add(spec["horizon_revision"])
        for ns in spec["namespaces"]:
            if ns in seen_namespaces:
                other = seen_namespaces[ns]
                raise HarnessError(
                    f"Namespace clash: mods {other!r} and {mod_id!r} both claim namespace {ns!r}"
                )
            seen_namespaces[ns] = mod_id

        # Check shared tags for destructive `"replace": true`
        for shared_ns in ("minecraft", "c"):
            tag_root = MODS_ROOT / mod_id / "content" / "data" / shared_ns / "tags"
            if tag_root.is_dir():
                for tag_file in tag_root.rglob("*.json"):
                    tdata = json.loads(tag_file.read_text(encoding="utf-8"))
                    if tdata.get("replace") is True:
                        raise HarnessError(
                            f"{tag_file}: mod {mod_id!r} sets 'replace': true on shared tag, breaking multi-mod compatibility"
                        )

    if len(revisions) > 1:
        print(
            f"[WARN] Mods pin multiple Horizon revisions ({sorted(revisions)}); "
            "launcher will build against the newest revision."
        )


def run(cmd: list, *, cwd: Path | None = None, env: dict | None = None, timeout: int | None = None,
        capture: bool = False, log: Path | None = None) -> subprocess.CompletedProcess:
    """Run a setup command, echoing it; a failure raises HarnessError with the end of its output."""
    print(f"  $ {' '.join(str(c) for c in cmd)}", flush=True)
    if log:
        with log.open("a", encoding="utf-8") as handle:
            result = subprocess.run([str(c) for c in cmd], cwd=cwd, env=env, timeout=timeout,
                                    stdout=handle, stderr=subprocess.STDOUT)
        if result.returncode:
            tail = "\n".join(log.read_text(encoding="utf-8", errors="replace").splitlines()[-40:])
            raise HarnessError(f"{cmd[0]} failed (exit {result.returncode}); end of {log}:\n{tail}")
        return result
    result = subprocess.run([str(c) for c in cmd], cwd=cwd, env=env, timeout=timeout,
                            capture_output=capture, text=True, encoding="utf-8", errors="replace")
    if result.returncode:
        detail = (result.stderr or result.stdout or "").strip()[-2000:] if capture else ""
        raise HarnessError(f"command failed (exit {result.returncode}): {' '.join(map(str, cmd))}\n{detail}")
    return result


def git(*args: str, capture: bool = True) -> str:
    return run(["git", "-C", HORIZON_ROOT, *args], capture=capture).stdout.strip() if capture else ""


# ---------------------------------------------------------------- 1. submodule

def ensure_submodule(fetch: bool) -> dict:
    """Initialise ./horizon when it is empty, then fetch it. The checkout itself is never moved."""
    if not (HORIZON_ROOT / ".git").exists():
        print("[setup] Initialising the horizon submodule")
        run(["git", "-C", ROOT, "submodule", "update", "--init", "horizon"])
    if fetch:
        print("[setup] Fetching horizon (origin)")
        run(["git", "-C", HORIZON_ROOT, "fetch", "--quiet", "origin"])
    recorded = run(["git", "-C", ROOT, "ls-tree", "HEAD", "horizon"], capture=True).stdout.split()
    info = {
        "checkout": git("rev-parse", "HEAD"),
        "recorded": recorded[2] if len(recorded) > 2 else None,
        "origin_main": git("rev-parse", "origin/main"),
    }
    print(f"[setup] horizon checkout {info['checkout'][:12]}, recorded {str(info['recorded'])[:12]}, "
          f"origin/main {info['origin_main'][:12]}")
    return info


def have_commit(rev: str) -> bool:
    # `cat-file -t` rather than `-e rev^{commit}`: the braces get mangled by some Windows git builds.
    r = subprocess.run(["git", "-C", str(HORIZON_ROOT), "cat-file", "-t", rev], capture_output=True, text=True)
    return r.returncode == 0 and r.stdout.strip() == "commit"


def commit_time(rev: str) -> int:
    return int(git("log", "-1", "--format=%ct", rev))


def build_revision(choice: str, specs: list[dict], sub: dict) -> str:
    """The Horizon commit to build: the newest the mods pin (as the launcher does), the submodule's
    checkout, origin/main, or an explicit commit."""
    if choice == "pinned":
        pins = sorted({s["horizon_revision"] for s in specs})
        for rev in pins:
            if not have_commit(rev):
                raise HarnessError(f"a mod pins Horizon {rev}, which origin does not have")
        rev = max(pins, key=commit_time)
        if len(pins) > 1:
            print(f"[WARN] Mods pin {len(pins)} Horizon revisions; building the newest, {rev[:12]}")
        return rev
    if choice == "submodule":
        return sub["checkout"]
    if choice == "latest":
        return sub["origin_main"]
    rev = git("rev-parse", choice)
    if not have_commit(rev):
        raise HarnessError(f"no such Horizon commit: {choice}")
    return rev


# ---------------------------------------------------------------- 2. source

def unpack_source(rev: str) -> Path:
    """Horizon's tree at `rev`, unpacked once into target/e2e/source/<rev12>. Links are dropped:
    horizon-mixins refuses them and the build needs none (as the launcher does)."""
    dest = E2E_ROOT / "source" / rev[:12]
    if (dest / ".horizon-revision").is_file():
        return dest
    print(f"[setup] Unpacking Horizon {rev[:12]} into {dest}")
    partial = dest.with_name(f".{rev[:12]}.partial")
    shutil.rmtree(partial, ignore_errors=True)
    partial.mkdir(parents=True)
    archive = partial.with_suffix(".tar")
    run(["git", "-C", HORIZON_ROOT, "archive", "--format=tar", "-o", archive, rev])
    with tarfile.open(archive) as tar:
        members = [m for m in tar.getmembers() if not (m.issym() or m.islnk())]
        tar.extractall(partial, members=members, filter="data")
    archive.unlink()
    (partial / ".horizon-revision").write_text(rev, encoding="utf-8")
    shutil.rmtree(dest, ignore_errors=True)
    # Windows refuses to rename a folder something (the virus scanner, the indexer) is still
    # reading: retry for a while, then copy it over instead.
    for attempt in range(10):
        try:
            partial.rename(dest)
            return dest
        except PermissionError:
            time.sleep(1 + attempt)
    shutil.copytree(partial, dest)
    shutil.rmtree(partial, ignore_errors=True)
    return dest


# ---------------------------------------------------------------- 3. mixins

def mixins_tool(rev: str) -> Path:
    """horizon-mixins at commit `rev`, built once into target/e2e/tools/<rev>."""
    root = E2E_ROOT / "tools" / rev[:12]
    binary = root / "bin" / f"horizon-mixins{EXE}"
    if not binary.is_file():
        print(f"[setup] Building horizon-mixins {rev[:12]} (once)")
        run(["cargo", "install", "--locked", "--git", MIXINS_GIT, "--rev", rev, "--root", root, "horizon-mixins-cli"],
            log=E2E_ROOT / "build.log")
    return binary


def native_bundles(specs: list[dict]) -> list[tuple[str, Path]]:
    bundles = []
    for spec in specs:
        bundle = MODS_ROOT / spec["id"] / "native" / "mixins" / spec["id"]
        if (bundle / "horizon-mixin.toml").is_file():
            bundles.append((spec["id"], bundle))
    return bundles


def bundle_digest(bundles: list[tuple[str, Path]]) -> str:
    h = hashlib.sha256()
    for mod_id, bundle in bundles:
        for f in sorted(p for p in bundle.rglob("*") if p.is_file()):
            h.update(f"{mod_id}/{f.relative_to(bundle).as_posix()}\0".encode())
            h.update(f.read_bytes())
    return h.hexdigest()[:12]


def sync_tree(src: Path, dest: Path) -> None:
    """Make `dest` match `src` (as `rsync -a --checksum --delete` would): files whose bytes are
    unchanged are left alone, so their timestamps and Cargo's fingerprints survive."""
    dest.mkdir(parents=True, exist_ok=True)
    for dirpath, dirnames, filenames in os.walk(src):
        here = Path(dirpath)
        there = dest / here.relative_to(src)
        wanted = set(dirnames) | set(filenames)
        for old in there.iterdir():
            if old.name in wanted and old.is_dir() == (old.name in dirnames) and not old.is_symlink():
                continue
            if old.is_dir() and not old.is_symlink():
                shutil.rmtree(old)
            else:
                old.unlink()
        for name in dirnames:
            (there / name).mkdir(exist_ok=True)
        for name in filenames:
            target = there / name
            if not (target.is_file() and filecmp.cmp(here / name, target, shallow=False)):
                shutil.copy2(here / name, target)


def stage_source(source: Path, rev: str, specs: list[dict], mixins_rev: str) -> Path:
    """Apply the selected mods' bundles to `source` and sync the result into target/e2e/stage/<rev12>,
    copying only changed files so Cargo recompiles only the crates the mods touch."""
    stage = (E2E_ROOT / "stage" / rev[:12]).resolve()
    bundles = native_bundles(specs)
    incoming = (E2E_ROOT / "mods.incoming").resolve()
    shutil.rmtree(incoming, ignore_errors=True)
    incoming.mkdir(parents=True)
    for mod_id, bundle in bundles:
        shutil.copytree(bundle, incoming / mod_id)
        # Built on `rev`, so the bundle is repinned to it, as the launcher's build does.
        manifest = incoming / mod_id / "horizon-mixin.toml"
        text = manifest.read_text(encoding="utf-8")
        text = re.sub(r'(horizon_revision\s*=\s*")[0-9a-f]+(")', rf"\g<1>{rev}\g<2>", text)
        manifest.write_text(text, encoding="utf-8")
    fresh = stage.with_name(f"{rev[:12]}.new")
    shutil.rmtree(fresh, ignore_errors=True)
    if bundles:
        print(f"[setup] Applying native mixins: {', '.join(m for m, _ in bundles)}")
        tool = mixins_tool(mixins_rev)
        run([tool, "apply", "--workspace", source, "--mods", incoming, "--out", fresh], log=E2E_ROOT / "build.log")
    else:
        shutil.copytree(source, fresh)
    print(f"[setup] Syncing {fresh.name} into {stage}")
    sync_tree(fresh, stage)
    shutil.rmtree(fresh, ignore_errors=True)
    return stage


# ---------------------------------------------------------------- 4. build

def build_game(stage: Path, rev: str, key: str) -> Path:
    """horizon-client and horizon-ctl from `stage`, copied to target/e2e/bin/<key>."""
    bins = E2E_ROOT / "bin" / key
    if have_bins(bins):
        print(f"[setup] Using the cached build {bins}")
        return bins
    cargo_toml = tomllib.loads((stage / "Cargo.toml").read_text(encoding="utf-8"))
    profile = "fast" if "fast" in cargo_toml.get("profile", {}) else "release"
    client_toml = tomllib.loads((stage / "crates" / "horizon-client" / "Cargo.toml").read_text(encoding="utf-8"))
    target = (E2E_ROOT / "cargo-target" / rev[:12]).resolve()
    cmd = ["cargo", "build", "--manifest-path", stage / "Cargo.toml", "--locked", "--profile", profile]
    # Each binary from the package that declares it (horizon-ctl moved to horizon-client-lib in late 2026).
    for name in ("horizon-client", "horizon-ctl"):
        cmd += ["-p", bin_package(stage, name), "--bin", name]
    if "mods" in client_toml.get("features", {}):
        cmd += ["--features", "horizon-client/mods"]
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    for var in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL"):
        env.pop(var, None)
    print(f"[setup] Building horizon-client + horizon-ctl ({profile}); the first build takes a while. "
          f"Log: {E2E_ROOT / 'build.log'}")
    started = time.time()
    run(cmd, env=env, log=E2E_ROOT / "build.log")
    print(f"[setup] Built in {time.time() - started:.0f} s")
    bins.mkdir(parents=True, exist_ok=True)
    for name in ("horizon-client", "horizon-ctl"):
        shutil.copy2(target / profile / f"{name}{EXE}", bins / f"{name}{EXE}")
    return bins


def bin_package(stage: Path, name: str) -> str:
    """The workspace package that builds binary `name` (a [[bin]] entry or src/bin/<name>)."""
    manifests = [(m, tomllib.loads(m.read_text(encoding="utf-8"))) for m in sorted((stage / "crates").glob("*/Cargo.toml"))]
    # A [[bin]] entry first: a package can keep a stale src/bin/<name> it no longer builds (autobins = false).
    for manifest, data in manifests:
        if any(b.get("name") == name for b in data.get("bin", [])):
            return data.get("package", {}).get("name", manifest.parent.name)
    for manifest, data in manifests:
        package = data.get("package", {})
        if package.get("autobins", True) and ((manifest.parent / "src" / "bin" / name).exists()
                                              or (name == package.get("name") and (manifest.parent / "src" / "main.rs").is_file())):
            return package.get("name", manifest.parent.name)
    return "horizon-client"


def have_bins(bins: Path) -> bool:
    return (bins / f"horizon-client{EXE}").is_file() and (bins / f"horizon-ctl{EXE}").is_file()


def prepare_build(specs: list[dict], args, sub: dict, rev: str | None = None) -> tuple[Path, str]:
    """Steps 2-4 for one mod set (none: Horizon alone): the folder holding its horizon-client and horizon-ctl."""
    if args.bin_dir:
        return args.bin_dir.resolve(), "prebuilt"
    rev = rev or build_revision(args.horizon_rev, specs, sub)
    mixins_revs = sorted({s["mixins_revision"] for s in specs})
    mixins_rev = mixins_revs[-1] if mixins_revs else ""
    if len(mixins_revs) > 1:
        print(f"[WARN] Mods pin {len(mixins_revs)} horizon-mixins revisions; using {mixins_rev[:12]}")
    key = f"{rev[:12]}-{bundle_digest(native_bundles(specs))}"
    bins = E2E_ROOT / "bin" / key
    if args.rebuild:
        shutil.rmtree(bins, ignore_errors=True)
    if not have_bins(bins):
        source = unpack_source(rev)
        stage = stage_source(source, rev, specs, mixins_rev)
        build_game(stage, rev, key)
    else:
        print(f"[setup] Using the cached build {bins}")
    return bins, rev


# ---------------------------------------------------------------- 5-6. run and check

def minecraft_dir(explicit: str | None = None) -> Path:
    """The official launcher's game directory (where horizon-prepare looks too)."""
    if explicit or os.environ.get("HORIZON_MINECRAFT_DIR"):
        return Path(explicit or os.environ["HORIZON_MINECRAFT_DIR"]).expanduser()
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / "minecraft"
    if sys.platform.startswith("win"):
        return Path(os.environ.get("APPDATA", "")) / ".minecraft"
    return Path.home() / ".minecraft"


def minecraft_installed(mc: Path) -> bool:
    v = mc / "versions" / GAME_VERSION
    return (v / f"{GAME_VERSION}.json").is_file() and (v / f"{GAME_VERSION}.jar").is_file()


def pinned_minecraft(source: Path) -> dict:
    return json.loads((source / "research-lock.json").read_text(encoding="utf-8"))["minecraft"]


def sha1_of(path: Path) -> str:
    h = hashlib.sha1()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def server_jar(source: Path, mc: Path, explicit: str | None) -> Path | None:
    """The pinned 26.3 server jar: given, else HORIZON_SERVER_JAR, else fetched once from the URL in
    the player's own versions/26.3/26.3.json and checked against research-lock.json's SHA-1."""
    given = explicit or os.environ.get("HORIZON_SERVER_JAR")
    if given:
        return Path(given).expanduser()
    want = pinned_minecraft(source)["server_sha1"]
    jar = E2E_ROOT / "minecraft" / GAME_VERSION / "server.jar"
    if jar.is_file() and sha1_of(jar) == want:
        return jar
    meta_path = mc / "versions" / GAME_VERSION / f"{GAME_VERSION}.json"
    if not meta_path.is_file():
        return None
    server = json.loads(meta_path.read_text(encoding="utf-8")).get("downloads", {}).get("server", {})
    url, sha1 = server.get("url"), server.get("sha1")
    if not url or sha1 != want:
        raise HarnessError(f"{meta_path} does not list the pinned 26.3 server jar (sha1 {want})")
    print(f"[setup] Fetching the 26.3 server jar named in your launcher's {meta_path.name}")
    jar.parent.mkdir(parents=True, exist_ok=True)
    print(f"  $ GET {url}", flush=True)
    try:
        import ssl  # certificates unverified: this machine's network re-signs HTTPS
        unverified = ssl._create_unverified_context()
        with urllib.request.urlopen(url, timeout=300, context=unverified) as response, jar.open("wb") as out:
            shutil.copyfileobj(response, out)
    except OSError as exc:
        jar.unlink(missing_ok=True)
        raise HarnessError(f"could not download the 26.3 server jar from {url}: {exc}") from exc
    if sha1_of(jar) != want:
        jar.unlink()
        raise HarnessError("the downloaded server jar does not match research-lock.json's SHA-1")
    return jar


def ensure_sounds(source: Path, res: Path, rev: str, mc: Path) -> None:
    """Minecraft's sounds in the prepared resources (horizon-prepare sounds: copied from the player's
    own launcher, nothing downloaded). Without them Horizon warns "no sounds in ...", which fails runs."""
    if (res / "assets" / "minecraft" / "sounds.json").is_file() or not minecraft_installed(mc):
        return
    print(f"[setup] Copying Minecraft's sounds from {mc} into the prepared resources")
    env = dict(os.environ, CARGO_TARGET_DIR=str((E2E_ROOT / "cargo-target" / rev[:12]).resolve()))
    run(["cargo", "run", "--release", "--locked", "--manifest-path", source / "Cargo.toml", "-p", "horizon-prepare",
         "--", "sounds", "--root", source, "--minecraft-dir", mc], env=env, log=E2E_ROOT / "prepare.log")


def resources_for(rev: str, args, *, prepare_now: bool = False) -> Path:
    """Prepared resources for Horizon `rev`. With --prepare (or prepare_now), builds them from the
    player's own Minecraft 26.3 with that revision's horizon-prepare."""
    if args.resources:
        res = args.resources.resolve()
        if (res / "horizon-resources.json").is_file():
            return res
        raise HarnessError(f"--resources {res} has no horizon-resources.json")
    source = unpack_source(rev)
    res = source / ".local" / "horizon-client" / GAME_VERSION
    mc = minecraft_dir(args.minecraft_dir)
    if (res / "horizon-resources.json").is_file():
        ensure_sounds(source, res, rev, mc)
        return res
    if (args.prepare or prepare_now) and minecraft_installed(mc):
        jar = server_jar(source, mc, args.server_jar)
        cmd = ["cargo", "run", "--release", "--locked", "--manifest-path", source / "Cargo.toml",
               "-p", "horizon-prepare", "--", "--prepare", "--root", source, "--minecraft-dir", mc]
        if jar:
            cmd += ["--server-jar", jar]
        env = dict(os.environ, CARGO_TARGET_DIR=str((E2E_ROOT / "cargo-target" / rev[:12]).resolve()))
        print(f"[setup] Preparing game resources for Horizon {rev[:12]} from {mc} "
              f"(Java {pinned_minecraft(source)['java_major']}); log: {E2E_ROOT / 'prepare.log'}")
        run(cmd, env=env, log=E2E_ROOT / "prepare.log")
        if (res / "horizon-resources.json").is_file():
            ensure_sounds(source, res, rev, mc)
            return res
        raise HarnessError(f"horizon-prepare finished but {res} has no horizon-resources.json; see {E2E_ROOT / 'prepare.log'}")
    raise HarnessError(
        f"No prepared game resources for Horizon {rev[:12]}.\n"
        "Horizon builds them from your own copy of Minecraft: Java Edition 26.3 and never ships them.\n"
        + (f"  Minecraft 26.3 is installed in {mc}: re-run with --prepare (or `./setup.sh`).\n"
           if minecraft_installed(mc) else
           f"  1. Install the official Minecraft launcher, sign in, and play version {GAME_VERSION} once\n"
           f"     (it must exist in {mc / 'versions' / GAME_VERSION}).\n"
           "  2. Run `./setup.sh` again: it fetches the matching server jar and prepares everything.\n")
        + "Use --dry-run or --setup-only to check everything else without the game."
    )


class Ctl:
    """One horizon-ctl process per command, against one game."""

    def __init__(self, ctl: Path, socket: Path, cwd: Path, env: dict):
        self.base = [str(ctl), "--socket", str(socket)]
        self.cwd, self.env = cwd, env

    def __call__(self, *words, timeout: int = 120) -> tuple[int, dict]:
        try:
            result = subprocess.run(self.base + [str(w) for w in words], cwd=self.cwd, env=self.env,
                                    capture_output=True, text=True, encoding="utf-8", errors="replace",
                                    timeout=timeout)
        except subprocess.TimeoutExpired:
            return 124, {"error": f"timed out after {timeout} s", "kind": "timeout"}
        lines = result.stdout.strip().splitlines()
        try:
            reply = json.loads(lines[-1]) if lines else {"error": result.stderr.strip() or "no reply"}
        except json.JSONDecodeError:
            reply = {"error": lines[-1], "kind": "unparsed"}
        if result.returncode and "error" not in reply:
            reply = {**reply, "error": result.stderr.strip() or f"exit {result.returncode}"}
        return result.returncode, reply


# Templates whose block opens a menu on right-click (horizon-world level/interact.rs, PlayerEffect::OpenMenu);
# signs and lecterns open theirs only in some states.
MENU_TEMPLATE = re.compile(r"minecraft:(?:furnace|blast_furnace|smoker|(?:chipped_|damaged_)?anvil|barrel|beacon"
                           r"|brewing_stand|cartography_table|(?:trapped_)?chest|(?:\w+_)?copper_chest|crafter"
                           r"|crafting_table|dispenser|dropper|enchanting_table|ender_chest|grindstone|hopper|loom"
                           r"|(?:\w+_)?shulker_box|smithing_table|stonecutter)")
# The test levels every block and item passes (planning/harness.md); a contract scenario names its level.
TEST_LEVELS = {1: "placement & hitbox", 2: "item use", 3: "block function (energy, fuel, storage, GUI slots)",
               4: "interaction with other blocks", 5: "persistence (chunk reloaded from disk)",
               6: "automation & sides (hoppers, tubes, sided access)", 7: "all mods together"}
CREATIVE_DISPLAY = 900  # Horizon's first creative display slot (Creative::DISPLAY): slot-900 onward show a tab


def stack_item(widget: dict) -> str | None:
    stack = widget.get("stack")
    return stack.get("item") if isinstance(stack, dict) else None


def creative_check(ctl: "Ctl", spec: dict, shots: Path) -> tuple[list[dict], set[str]]:
    """Open the creative inventory, go through every tab on every page after vanilla's (where mods' tabs
    are, as NeoForge pages them) and every row of each, and return (errors, items seen). An item or
    block item in no tab is only in the search. `ui` lists the grid's slots but not the tabs, so the
    tabs and page buttons are clicked where Horizon draws them (creative.rs tab_position)."""
    expected = spec["creative_expected"]
    errors: list[dict] = []
    seen: set[str] = set()
    if not expected:
        return errors, seen
    ctl("close")
    ctl("key", "e", "tap")
    ctl("wait", 5)
    _, ui = ctl("ui")
    (shots / "creative-ui.json").write_text(json.dumps(ui, indent=1), encoding="utf-8")  # what the check saw

    def grid() -> list[dict]:
        _, ui = ctl("ui")
        return sorted((w for w in ui.get("widgets", []) if w.get("kind") == "slot" and w.get("id", "")[5:].isdigit()
                       and int(w["id"][5:]) >= CREATIVE_DISPLAY), key=lambda w: int(w["id"][5:]))

    def read_tab() -> tuple[str, ...]:
        """Every item of the selected tab, scrolling until no new ones show."""
        found: list[str] = []
        for scroll in range(200):
            slots = grid()
            new = [i for w in slots if (i := stack_item(w)) and i not in found]
            if scroll and not new:
                break
            found += new
            if not slots:
                break
            x, y, w, h = slots[len(slots) // 2]["bounds"]
            ctl("scroll-at", x + w / 2, y + h / 2, -1)
            ctl("wait", 1)
        ctl("scroll-at", 0, 0, 0)
        return tuple(found)

    first = grid()
    if ui.get("screen") != "creative" or not first:
        errors.append({"stage": "9_creative_menu", "error": f"the inventory opened {ui.get('screen')!r} with no item grid"})
        return errors, seen
    # The screen's origin: the first grid slot is 9, 18 inside it (CreativeModeInventoryScreen).
    ox, oy = first[0]["bounds"][0] - 9, first[0]["bounds"][1] - 18
    tabs = [(ox + col * 27 + 13, oy - 28 + 16) for col in range(5)] + [(ox + col * 27 + 13, oy + 132 + 16) for col in range(5)]
    previous = None
    for page in range(1, 20):
        ctl("click-at", ox + 175 + 10, oy - 50 + 10)  # the next-page button (selects the page's first tab)
        ctl("wait", 2)
        opened = read_tab()
        if opened == previous:  # the last page: the button did nothing
            break
        previous = opened
        seen |= set(opened)
        for x, y in tabs[1:]:
            ctl("click-at", x, y)
            ctl("wait", 2)
            seen |= set(read_tab())
        ctl("screenshot", shots / f"creative-page-{page}.png")
    ctl("close")
    for item in expected:
        if item not in seen:
            errors.append({"stage": "9_creative_menu", "item": item,
                           "error": f"{item} is not in any of the mods' creative tabs (only in the search)"})
    return errors, seen


def render_problem(shots: Path, key: str) -> str | None:
    """What the screenshot of a placed block shows, against the same view with the spot empty.

    Horizon draws a block with no usable model or texture as the magenta missing texture, or not at
    all, and logs nothing; the game log cannot find that, the pixels can. Returns the problem, or None."""
    sys.path.insert(0, str(ROOT / "scripts"))
    from art import read_png
    try:
        w, h, after = read_png(shots / f"{key}_world.png")
        w2, h2, before = read_png(shots / f"{key}_empty.png")
    except (OSError, ValueError) as exc:
        return f"render check: no screenshot to read ({exc})"
    if (w, h) != (w2, h2):
        return f"render check: screenshots differ in size ({w}x{h} against {w2}x{h2})"
    x0, x1, y0, y1 = w * 3 // 10, w * 7 // 10, h * 3 // 10, h * 7 // 10
    magenta = changed = total = 0
    for y in range(y0, y1):
        row = y * w
        for x in range(x0, x1):
            r, g, b, _ = after[row + x]
            br, bg, bb, _ = before[row + x]
            total += 1
            if r > 170 and b > 170 and g < 90:
                magenta += 1
            if abs(r - br) + abs(g - bg) + abs(b - bb) > 30:
                changed += 1
    if magenta > max(40, total // 4000):
        return (f"the block is drawn with the magenta missing texture ({magenta} pixels of {total} at its view, "
                f"{key}_world.png): its blockstate, model or a texture does not resolve in the engine")
    if changed < max(40, total // 4000):
        return (f"the block is invisible: the view of it differs from the empty spot in {changed} of {total} pixels "
                f"({key}_world.png against {key}_empty.png)")
    return None


def step_words(step: dict, shots: Path) -> list:
    a = step["args"]
    cmd = step["cmd"]
    if cmd == "give":
        return ["give", a["item"], a.get("count", 1)]
    if cmd == "command":
        return ["command", a["command"]]
    if cmd in ("block", "look_at_block", "break", "tp"):
        words = [cmd.replace("_", "-"), a["x"], a["y"], a["z"]]
        if "face" in a:
            words.append(a["face"])
        return words
    if cmd == "fill":
        return ["fill", *a["box"], a["block"]]
    if cmd == "setblock":
        return ["setblock", a["x"], a["y"], a["z"], a["block"]]
    if cmd == "mouse":
        return ["mouse", a["button"], a["action"]]
    if cmd == "wait":
        return ["wait", a["ticks"]]
    if cmd == "screenshot":
        return ["screenshot", shots / a["path"]]
    if cmd == "select":
        return ["select", a["slot"]]
    if cmd == "close":
        return ["close"]
    if cmd == "state":
        return ["state"]
    if cmd == "look_at":
        return ["look-at", a["x"], a["y"], a["z"]]
    if cmd == "looking_at":
        return ["looking-at"]
    if cmd == "ui":
        return ["ui"]
    if cmd == "messages":
        return ["messages", a.get("n", 10)]
    if cmd == "raw":
        return list(a["words"])
    if cmd == "entities":
        return ["entities", a["radius"], a["type"]]
    raise HarnessError(f"action plan uses a command the harness does not map: {cmd}")


def allowed_warnings() -> list[re.Pattern]:
    """The log lines a person accepted (planning/harness-allowed-warnings.txt: one regex a line)."""
    if not ALLOWED_WARNINGS.is_file():
        return []
    return [re.compile(line.strip()) for line in ALLOWED_WARNINGS.read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.lstrip().startswith("#")]


def normalize_log_line(line: str) -> str:
    """A log line without its time and scratch paths, to compare runs."""
    # "[16:11:00] [Render thread/WARN]: text" -> "WARN: text", then scratch paths -> <path>.
    m = re.match(r"^\[[^\]]*\]\s*\[[^\]]*?/(\w+)\]:?\s*(.*)$", line.strip())
    line = f"{m.group(1)}: {m.group(2)}" if m else line.strip()
    return re.sub(r"(?:/[\w.@+-]+){2,}/?", "<path>", line)


def horizon_status() -> dict:
    try:
        return json.loads(HORIZON_STATUS.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def log_errors(text: str, allow: list[re.Pattern]) -> list[str]:
    return [line for line in text.splitlines()
            if LOG_ERROR.search(line) and not any(p.search(line) for p in allow)]


def link_dir(target: Path, link: Path) -> None:
    """A symlink to a directory; on Windows without symlink rights (no Developer Mode or admin),
    a directory junction, which needs none."""
    try:
        os.symlink(target, link, target_is_directory=True)
    except OSError:
        if os.name != "nt":
            raise
        import _winapi
        _winapi.CreateJunction(str(target.resolve()), str(link))


def run_live_session(session_name: str, specs: list[dict], bins: Path, resources: Path, args) -> dict:
    """Boot horizon-client with exactly `specs` enabled, run their action plans, collect every error."""
    client_bin, ctl_bin = bins / f"horizon-client{EXE}", bins / f"horizon-ctl{EXE}"
    shots = REPORT_DIR / session_name
    shutil.rmtree(shots, ignore_errors=True)
    shots.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix=f"rekkit-e2e-{session_name}-")).resolve()
    errors: list[dict] = []
    results: list[dict] = []
    started = time.time()
    # Unix socket paths are capped at 104 bytes on macOS (108 on Linux); the temp home is too deep.
    # Windows has no Unix sockets: the game listens on localhost TCP and writes port + token to a
    # .json endpoint file instead, which horizon-ctl reads.
    sock_dir = Path(tempfile.mkdtemp(prefix="rk-", dir="/tmp" if os.name == "posix" else None))
    socket = sock_dir / ("c.json" if os.name == "nt" else "c.sock")
    ctl = None
    try:
        # A scratch game home, laid out as Horizon's own harnesses do (tools/agent_control_e2e.py).
        (work / ".local" / "horizon-client").mkdir(parents=True)
        link_dir(resources, work / ".local" / "horizon-client" / GAME_VERSION)
        (work / "options.txt").write_text("onboardAccessibility:false\n", encoding="utf-8")
        (work / "horizon-extras.txt").write_text("agentControl:true\n", encoding="utf-8")
        (work / "horizon-mods").mkdir()
        for spec in specs:
            hmod = MODS_ROOT / spec["id"] / "target" / "horizon-mod" / f"{spec['id']}.hmod"
            shutil.copy2(hmod, work / "horizon-mods" / hmod.name)
        (work / "horizon-mods.txt").write_text("".join(f"mod.{s['id']}:true\n" for s in specs), encoding="utf-8")

        env = os.environ.copy()
        for key in ("HORIZON_LAUNCHER_HOME", "XDG_DATA_HOME", "XDG_CONFIG_HOME"):
            (work / key.lower()).mkdir()
            env[key] = str(work / key.lower())
        env.update({
            "HORIZON_HOME": str(work), "HORIZON_NO_AUDIO": "1", "HORIZON_TELEMETRY_URL": "off",
            "HORIZON_LAUNCHER_NO_SELF_UPDATE": "1", "HORIZON_NO_DISCORD": "1",
        })
        ctl = Ctl(ctl_bin, socket, work, env)

        print(f"[{session_name}] Launching Horizon with {', '.join(s['id'] for s in specs)}")
        code, reply = ctl("launch", "--game", client_bin, "--world", f"E2E_{session_name}", "--creative",
                          "--offscreen", "--wait", "--resources", resources, timeout=900)
        results.append({"stage": "0_launch", "reply": reply})
        if code or "error" in reply:
            errors.append({"stage": "0_launch", "error": reply})
            raise HarnessError(f"{session_name}: the game did not start: {reply}")

        plan = []
        for plot_idx, spec in enumerate(specs):
            plan += build_action_plan(spec, plot_index=plot_idx)
        variables: dict[str, float] = {}  # what scenario steps saved ("save"), per session
        scenario_vars = None
        for i, step in enumerate(plan):
            if "cmd" not in step:
                results.append({"i": i, "stage": step["stage"], "skipped": step.get("skipped_why", "no horizon-ctl command yet"),
                                "recipe": step.get("recipe_id")})
                continue
            if step.get("scenario") != scenario_vars:  # each scenario starts with no saved values
                scenario_vars = step.get("scenario")
                variables = {k: v for k, v in variables.items() if k.startswith("wg_")}
            if step["cmd"] in probes.COMPOUND:
                words = [step["cmd"], json.dumps(step.get("args", {}))]
                try:
                    code, reply = probes.run_compound(ctl, step)
                except (KeyError, ValueError, TypeError) as exc:
                    code, reply = 1, {"error": f"{step['cmd']}: {exc}", "kind": "harness"}
            else:
                words = step_words(step, shots)
                code, reply = ctl(*words, timeout=180)
            entry = {"i": i, "stage": step["stage"], "cmd": " ".join(map(str, words)), "reply": reply}
            for key in ("block", "item", "entity", "feature", "recipe"):
                if step.get(key):
                    entry[key] = step[key]
            if step.get("scenario"):
                entry["scenario"] = step["scenario"]
                entry["for"] = step.get("for", [])
                entry["level"] = step.get("level")
            problem = None
            if "error" not in reply and step.get("save"):
                problem = probes.save(step, reply, variables)
            if problem:
                pass
            elif "error" in reply and reply.get("kind") not in step.get("allow_kinds", []):
                problem = reply
            elif step.get("check"):
                problem = probes.check(step, reply, variables)
            elif step.get("expect_block") and step["expect_block"] not in json.dumps(reply):
                problem = f"expected block {step['expect_block']}" + (f" ({step['what']})" if step.get("what") else "")
            elif step.get("expect_hit"):
                pos, want_hit = step["expect_hit"]
                hit = (reply.get("block") or {}).get("pos") == pos
                if hit != want_hit:
                    met = (reply.get("block") or {}).get("state") or reply.get("kind")
                    problem = (f"{step['what']}: the crosshair " + ("missed it" if want_hit else "met it above the original's shape")
                               + f" (met {met}): its outline is not the original's shape")
            elif step.get("expect_gui"):
                problem = gui_problem(step["expect_gui"], reply)
                problem = problem and f"{step['what']}: {problem}"
            elif step.get("expect_no_menu") and reply.get("screen") in ("container", "station"):
                problem = f"{step['what']}: it opened a {reply.get('screen')} screen"
            elif step.get("expect_json"):
                if not re.search(step["expect_json"], json.dumps(reply)):
                    problem = f"{step['what']}: expected /{step['expect_json']}/ in {json.dumps(reply)[:300]}"
            elif step.get("expect_match"):
                text = " | ".join([*(reply.get("chat") or []), reply.get("actionbar") or "", reply.get("state") or ""])
                if not re.search(step["expect_match"], text):
                    problem = f"{step['what']}: expected /{step['expect_match']}/, got {text[:300]!r}"
            elif step.get("expect_screen") and reply.get("screen") not in step["expect_screen"]:
                problem = (f"{step['what']}: it opened {reply.get('screen')!r}, not a menu (the player sees "
                           "'... is not available yet'). If the original has a GUI (the spec's GUIs), give it "
                           "that menu (native_block_entity_container); only if it has none, a template that "
                           "opens none")
            elif step.get("expect_drop"):
                exp = step["expect_drop"]
                got_drops = reply.get("drops") or {}
                got_picked = reply.get("picked_up") or {}
                if not (got_drops.get(exp) or got_picked.get(exp)):
                    # Check if the drops spawn/appear in subsequent list_drops
                    dropped_found = False
                    for next_idx in range(i + 1, min(i + 4, len(plan))):
                        next_step = plan[next_idx]
                        if next_step.get("stage") == "7_list_drops":
                            ctl("wait", 10)
                            next_words = step_words(next_step, shots)
                            _, next_reply = ctl(*next_words, timeout=30)
                            ents = next_reply.get("entities") or []
                            for ent in ents:
                                item_id = ent.get("item", {}).get("item") if isinstance(ent.get("item"), dict) else ent.get("item")
                                if item_id == exp:
                                    dropped_found = True
                                    break
                            if not dropped_found:
                                _, inv_reply = ctl("inventory")
                                if (inv_reply.get("counts") or {}).get(exp):
                                    dropped_found = True
                            break
                    if not dropped_found:
                        problem = f"breaking it dropped {got_drops or got_picked or 'nothing'}, expected {exp} (loot table)"
            if problem is None and step.get("render_check"):
                problem = render_problem(shots, step["render_check"])
            if problem:
                entry["error"] = problem
                errors.append(entry)
                print(f"  FAIL {step['stage']}: {' '.join(map(str, words))} -> {problem}")
            results.append(entry)

        for spec in specs:
            bad, seen = creative_check(ctl, spec, shots)
            results.append({"stage": "9_creative_menu", "mod": spec["id"], "seen": sorted(seen),
                            "expected": len(spec["creative_expected"])})
            for e in bad:
                print(f"  FAIL 9_creative_menu: {e['error']}")
            errors += bad

    except HarnessError as exc:
        if not errors:
            errors.append({"stage": "harness", "error": str(exc)})
    finally:
        if ctl:
            ctl("quit", timeout=60)
            time.sleep(2)
        if work.exists():
            # The game's own view of what went wrong: crash reports and its logs, read from the scratch home.
            crashes = sorted((work / "crash-reports").glob("*")) if (work / "crash-reports").is_dir() else []
            for crash in crashes:
                shutil.copy2(crash, shots / crash.name)
                errors.append({"stage": "8_crash_report", "error": f"crash report {crash.name} (copied to {shots})"})
            allow = allowed_warnings()
            baseline = set(horizon_status().get("log_lines", [])) if specs else set()
            for log_file in (work / "logs" / "latest.log", socket.with_suffix(".log")):
                if not log_file.is_file():
                    continue
                text = log_file.read_text(encoding="utf-8", errors="replace")
                (shots / log_file.name).write_text(text, encoding="utf-8")
                for line in log_errors(text, allow):
                    entry = {"stage": "8_game_log", "file": log_file.name, "error": line.strip()}
                    if normalize_log_line(line) in baseline:
                        entry["horizon"] = True  # Horizon logs it with no mods loaded too
                    errors.append(entry)
            bad_log = sum(1 for e in errors if e["stage"] == "8_game_log")
            if bad_log:
                print(f"  FAIL game log: {bad_log} error line(s); logs copied to {shots}")
        if args.keep_home:
            print(f"[{session_name}] Kept the game home: {work} (socket and launch log: {sock_dir})")
        else:
            shutil.rmtree(work, ignore_errors=True)
            shutil.rmtree(sock_dir, ignore_errors=True)

    executed = sum(1 for r in results if "cmd" in r)
    skipped = sum(1 for r in results if "skipped" in r)
    # ok; horizon: every error is one Horizon also has without mods; failed: the mod's.
    status = "ok" if not errors else "horizon" if all(e.get("horizon") for e in errors) else "failed"
    blocks = {b["id"]: "ok" for spec in specs for b in spec["blocks"]}
    creative = {i: "ok" for spec in specs for i in spec["creative_expected"]}
    # Everything else the plan tests, by what it tests: items (stack, armor, held), entities, worldgen, recipes.
    subjects: dict[str, dict[str, str]] = {"items": {}, "entities": {}, "worldgen": {}, "recipes": {}}
    for r in results:
        for key, group in (("item", "items"), ("entity", "entities"), ("feature", "worldgen"), ("recipe", "recipes")):
            if r.get(key):
                subjects[group].setdefault(r[key], "skipped" if "skipped" in r else "ok")
    for e in errors:
        if e.get("block") in blocks:
            blocks[e["block"]] = "failed"
        if e.get("item") in creative:
            creative[e["item"]] = "failed"
        for key, group in (("item", "items"), ("entity", "entities"), ("feature", "worldgen"), ("recipe", "recipes")):
            if e.get(key):
                subjects[group][e[key]] = "failed"
    print(f"[{session_name}] {status.upper()}: {executed} actions run, {skipped} skipped, {len(errors)} error(s), "
          f"{sum(v == 'ok' for v in blocks.values())}/{len(blocks)} blocks ok, {time.time() - started:.0f} s")
    if creative:
        print(f"[{session_name}] creative menu: {sum(v == 'ok' for v in creative.values())}/{len(creative)} items found")
    for group, status_of in subjects.items():
        if status_of:
            skipped_n = sum(v == "skipped" for v in status_of.values())
            print(f"[{session_name}] {group}: {sum(v == 'ok' for v in status_of.values())}/{len(status_of)} ok"
                  + (f", {skipped_n} skipped (need a scenario)" if skipped_n else ""))
    # The test levels (planning/harness.md): 1 is every block's placement and outline; 2-6 are the
    # contract's scenarios by their level.
    placed = {b: "failed" if any(e.get("block") == b and str(e.get("stage", "")).startswith("2_") for e in errors)
              else "ok" for b in blocks}
    levels = {"1": {"name": TEST_LEVELS[1], "ok": sum(v == "ok" for v in placed.values()), "total": len(placed)}}
    scenarios = [sc for spec in specs for sc in spec.get("scenarios", [])]
    bad = {e.get("scenario") for e in errors if e.get("scenario")}
    for n in sorted({int(sc.get("level") or 0) for sc in scenarios}):
        mine = [sc.get("name") for sc in scenarios if int(sc.get("level") or 0) == n]
        levels[str(n)] = {"name": TEST_LEVELS.get(n, "unlevelled"), "ok": sum(m not in bad for m in mine),
                          "total": len(mine), "failed": sorted(m for m in mine if m in bad)}
    for n, lv in levels.items():
        print(f"[{session_name}] level {n} {lv['name']}: {lv['ok']}/{lv['total']} ok")
    # The failures themselves, short: what an agent needs without reading summary.json (each line it
    # reads is paid again on every later turn).
    for e in errors[:FAILURES_SHOWN]:
        what = " ".join(f"{k}={e[k]}" for k in ("block", "item", "entity", "feature", "recipe", "scenario") if e.get(k))
        print(f"[{session_name}] FAIL {e.get('stage', '?')} {what}: {str(e.get('error', ''))[:200]}".rstrip(": "))
    if len(errors) > FAILURES_SHOWN:
        print(f"[{session_name}] ... {len(errors) - FAILURES_SHOWN} more in {REPORT_DIR / 'summary.json'} "
              "(search it for one block or scenario; do not read it whole)")
    return {"session": session_name, "mods": [s["id"] for s in specs], "status": status, "blocks": blocks,
            "creative": creative, "levels": levels, **subjects,
            "steps_executed": executed, "steps_skipped": skipped, "errors": errors, "steps": results}


def run_baseline(rev: str, args, sub: dict) -> dict:
    """Horizon at `rev` with no mods: does it build, and what does a run with nothing loaded log? Written
    to target/horizon-status.json, which decides whether a later failure is Horizon's or the mod's."""
    status = {"rev": rev, "checked": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "built": False,
              "build_error": "", "run": "skipped", "log_lines": [], "errors": []}
    print(f"[baseline] Horizon {rev[:12]} with no mods")
    try:
        bins, _ = prepare_build([], args, sub, rev=rev)
        status["built"] = True
    except HarnessError as exc:
        status["build_error"] = str(exc)[-3000:]
    if status["built"]:
        try:
            resources = resources_for(rev, args)
        except HarnessError as exc:
            # Not Horizon's fault: this machine has no game resources for it yet (./setup.sh says why).
            status["run"], status["errors"] = "no-resources", [str(exc)[-1000:]]
            resources = None
    if status["built"] and resources:
        try:
            entry = run_live_session("horizon_baseline", [], bins, resources, args)
            status["run"] = entry["status"]
            status["errors"] = entry["errors"][:50]
            status["log_lines"] = sorted({normalize_log_line(e["error"]) for e in entry["errors"]
                                          if e["stage"] == "8_game_log"})
        except HarnessError as exc:
            status["run"], status["errors"] = "failed", [str(exc)[-2000:]]
    good = status["built"] and status["run"] in ("ok", "no-resources")
    if not good and sub and sub.get("origin_main") and sub["origin_main"] != rev:
        # Only to report whether a newer Horizon fixes it; mods are never repinned here.
        try:
            prepare_build([], args, sub, rev=sub["origin_main"])
            status["latest"] = {"rev": sub["origin_main"], "built": True}
        except HarnessError as exc:
            status["latest"] = {"rev": sub["origin_main"], "built": False, "error": str(exc)[-500:]}
    HORIZON_STATUS.parent.mkdir(parents=True, exist_ok=True)
    HORIZON_STATUS.write_text(json.dumps(status, indent=2) + "\n", encoding="utf-8")
    print(f"[baseline] Horizon {rev[:12]}: " + ("builds" if status["built"] else "DOES NOT BUILD")
          + f", run {status['run']}, {len(status['log_lines'])} log line(s) of its own"
          + (f"; origin/main {status['latest']['rev'][:12]} " + ("builds" if status['latest']['built'] else "does not build either")
             if "latest" in status else "") + f". Status: {HORIZON_STATUS}")
    return status


def selected_mods(values: list[str] | None, all_dirs: list[Path]) -> list[Path]:
    if not values:
        return all_dirs
    names = [n for v in values for n in v.split(",") if n]
    dirs = []
    for name in names:
        d = MODS_ROOT / name
        if not (d / "Cargo.toml").is_file():
            raise HarnessError(f"no mod {name!r} under mods/")
        dirs.append(d)
    return dirs


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Rekkit E2E Horizon Mod Integration & Compatibility Harness.")
    parser.add_argument("--phase", choices=["isolated", "all-mods", "both"], default="both",
                        help="Phase 1 (one game per mod), Phase 2 (one game with the selected mods together), or both")
    parser.add_argument("--mod", action="append",
                        help="Mod id(s) to run, repeatable or comma-separated (default: every mod under mods/)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Package, audit and print the action plans; no submodule, build or game")
    parser.add_argument("--setup-only", action="store_true",
                        help="Submodule, source, mixins and build only; do not start the game")
    parser.add_argument("--horizon-rev", default="submodule",
                        help="Horizon commit to build: submodule (the checked-out horizon/, default), pinned (what the mods pin), latest, or a SHA")
    parser.add_argument("--no-fetch", action="store_true", help="Do not fetch the horizon submodule")
    parser.add_argument("--rebuild", action="store_true", help="Ignore the cached build for this mod set")
    parser.add_argument("--bin-dir", type=Path, help="Prebuilt horizon-client and horizon-ctl (skips steps 1-4)")
    parser.add_argument("--resources", type=Path,
                        help="Prepared resources (default: target/e2e/source/<rev12>/.local/horizon-client/26.3)")
    parser.add_argument("--prepare", action="store_true",
                        help="Build missing resources from your Minecraft 26.3 install (fetches the pinned server jar)")
    parser.add_argument("--server-jar", help="Minecraft 26.3 server jar (default: HORIZON_SERVER_JAR, else fetched)")
    parser.add_argument("--minecraft-dir", help="Official launcher game directory (default: HORIZON_MINECRAFT_DIR, else the OS default)")
    parser.add_argument("--baseline", action="store_true",
                        help="Build and run Horizon with no mods at the build revision; write target/horizon-status.json")
    parser.add_argument("--status", action="store_true", help="Print target/horizon-status.json; exit 3 if Horizon is broken")
    parser.add_argument("--keep-home", action="store_true", help="Keep each scratch game home for debugging")
    args = parser.parse_args()

    all_mod_dirs = sorted(p for p in MODS_ROOT.iterdir() if p.is_dir() and (p / "Cargo.toml").is_file())

    if args.status:
        st = horizon_status()
        if not st:
            print("No Horizon status yet: make horizon-baseline (make fleet-start runs it).")
            return 0
        good = st.get("built") and st.get("run") == "ok"
        print(f"Horizon {st['rev'][:12]} (checked {st['checked']}): " + ("builds" if st["built"] else "DOES NOT BUILD")
              + f", run with no mods: {st['run']}, {len(st.get('log_lines', []))} log line(s) of its own"
              + (f"; origin/main {st['latest']['rev'][:12]} " + ("builds" if st["latest"]["built"] else "does not build")
                 if "latest" in st else ""))
        for line in st.get("log_lines", [])[:20]:
            print(f"  {line}")
        if st.get("build_error"):
            print("  " + "\n  ".join(st["build_error"].splitlines()[-15:]))
        return 0 if good else 3
    if args.baseline:
        # Horizon alone: at the newest revision the mods pin, without validating or packing any mod,
        # so a broken mod never stops the check of Horizon itself.
        try:
            dirs = selected_mods(args.mod, all_mod_dirs)
            pins = [{"horizon_revision": json.loads((d / "native" / "horizon-native.json").read_text(encoding="utf-8"))["horizon"]}
                    for d in dirs if (d / "native" / "horizon-native.json").is_file()]
            sub = ensure_submodule(fetch=not args.no_fetch)
            rev = build_revision(args.horizon_rev, pins, sub)
            st = run_baseline(rev, args, sub)
            return 0 if st["built"] and st["run"] == "ok" else 3
        except HarnessError as exc:
            print(f"[ERROR] {exc}", file=sys.stderr)
            return 1

    try:
        target_dirs = selected_mods(args.mod, all_mod_dirs)
        # Package and validate every mod the run will load.
        for mdir in target_dirs:
            pack_mod.process_mod(mdir, pack=True)

        all_specs = [discover_mod_spec(mdir) for mdir in all_mod_dirs]
        target_specs = [discover_mod_spec(mdir) for mdir in target_dirs]

        check_cross_mod_compatibility(all_specs)
        print(
            f"[OK] Cross-mod compatibility audit passed across {len(all_specs)} mod(s): "
            f"{', '.join(s['id'] for s in all_specs)}"
        )

        REPORT_DIR.mkdir(parents=True, exist_ok=True)
        E2E_ROOT.mkdir(parents=True, exist_ok=True)
        (E2E_ROOT / "build.log").write_text("", encoding="utf-8")  # this run's output only
        report: dict = {"timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "phases": {}}
        sessions: list[tuple[str, str, list[dict]]] = []
        if args.phase in ("isolated", "both"):
            sessions += [("isolated", f"isolated_{s['id']}", [s]) for s in target_specs]
        if args.phase in ("all-mods", "both"):
            sessions.append(("all_mods", "all_mods", target_specs))

        for phase, name, specs in sessions:
            steps = sum(len(build_action_plan(s, plot_index=i)) for i, s in enumerate(specs))
            print(f"[{phase}] {name}: {', '.join(s['id'] for s in specs)} -> {steps} E2E action steps")

        live = not args.dry_run
        sub = None
        if live:
            sub = ensure_submodule(fetch=not args.no_fetch)
            report["horizon"] = sub
            if not args.setup_only and not args.bin_dir:
                # Fail before a long build when the resources can never be made here.
                rev = build_revision(args.horizon_rev, target_specs, sub)
                resources_for(rev, args)
                if horizon_status().get("rev") != rev or args.rebuild:
                    run_baseline(rev, args, sub)

        failed = horizon_only = False
        for phase, name, specs in sessions:
            if not live:
                entry = {"mods": [s["id"] for s in specs], "mode": "dry-run",
                         "steps_planned": sum(len(build_action_plan(s, plot_index=i)) for i, s in enumerate(specs))}
            else:
                try:
                    bins, rev = prepare_build(specs, args, sub)
                except HarnessError as exc:
                    rev = build_revision(args.horizon_rev, specs, sub)
                    broken = horizon_status().get("rev") == rev and not horizon_status().get("built")
                    entry = {"session": name, "mods": [s["id"] for s in specs], "horizon": rev,
                             "status": "horizon" if broken else "failed",
                             "errors": [{"stage": "build", "error": str(exc)[-3000:], "horizon": broken}]}
                    print(f"[{name}] " + (f"Horizon {rev[:12]} does not build even without mods: not the mod's fault"
                                          if broken else "The build fails with this mod's mixins (Horizon builds without them)"))
                    failed |= not broken
                    horizon_only |= broken
                    report["phases"].setdefault(phase, []).append(entry)
                    continue
                if args.setup_only:
                    entry = {"mods": [s["id"] for s in specs], "mode": "setup-only", "horizon": rev, "bins": str(bins)}
                    print(f"[{name}] Built on Horizon {rev[:12]}: {bins}")
                    if args.prepare:
                        entry["resources"] = str(resources_for(rev, args))
                        print(f"[{name}] Resources ready: {entry['resources']}")
                else:
                    resources = args.resources.resolve() if args.bin_dir and args.resources else resources_for(rev, args)
                    entry = run_live_session(name, specs, bins, resources, args)
                    entry["horizon"] = rev
                    failed |= entry["status"] == "failed"
                    horizon_only |= entry["status"] == "horizon"
            report["phases"].setdefault(phase, []).append(entry)

        # summary.json is the last game run's; a dry run or build alone must not replace it
        # (./setup.sh --check reads it to tell whether the live test has passed).
        report_file = REPORT_DIR / ("summary.json" if live and not args.setup_only else "setup.json")
        report_file.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"[FAIL] Errors found; see {report_file}" if failed else
              f"[HORIZON] Only Horizon's own problems (it has them with no mods too); see {report_file} and "
              f"{HORIZON_STATUS}" if horizon_only else f"[OK] Wrote E2E harness report to {report_file}")
        # 0 ok, 1 the mods failed, 3 only Horizon's own problems (the mods are not held up by them).
        return 1 if failed else 3 if horizon_only else 0
    except (HarnessError, pack_mod.ValidationError) as exc:
        print(f"[ERROR] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
