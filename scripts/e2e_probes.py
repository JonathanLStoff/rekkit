"""What the E2E harness (scripts/e2e_harness.py) tests beyond placing blocks: items, armor and tools,
entities, worldgen (features, structures, templates), recipes crafted and smelted in their station,
and the player-side measurements a scenario can make (damage taken, fall damage, jump height, walking
speed, attack damage, attributes, effects, inventory, containers, menus).

Everything here is generic: it reads the mod's content and contract, never a mod's name.

Plan steps are the harness's own ({"stage", "cmd", "args", ...}) plus:
  "check": {...}  what the reply must show (check() below);
  "save":  {...}  a value of the reply kept as a variable for later checks (save() below);
  "cmd": "c_*"    a compound action run by run_compound() (several horizon-ctl calls), or "none".
"""

from __future__ import annotations

import ast
import gzip
import io
import json
import math
import operator
import re
import struct
import time
from pathlib import Path

import contract_check

ARMOR_SLOTS = {"helmet": "head", "cap": "head", "chestplate": "chest", "tunic": "chest", "leggings": "legs",
               "pants": "legs", "boots": "feet", "elytra": "chest", "turtle_helmet": "head"}
SLOT_ARGS = {"head": "armor.head", "chest": "armor.chest", "legs": "armor.legs", "feet": "armor.feet",
             "offhand": "weapon.offhand", "mainhand": "weapon.mainhand"}
# Entity templates that are not mobs (no AI, no health attributes).
NOT_LIVING = re.compile(r"minecraft:(?:\w+_)?(?:snowball|egg|ender_pearl|arrow|trident|fireball|potion|"
                        r"experience_bottle|wind_charge|firework_rocket|item|tnt|boat|chest_boat|raft|minecart|"
                        r"falling_block|armor_stand|item_frame|painting|lightning_bolt|marker|area_effect_cloud)$")
# Recipe types the harness crafts itself, in their vanilla station: block, input slots, result slot, ticks.
STATIONS = {
    "minecraft:crafting_shaped": ("minecraft:crafting_table", None, 200, 2),
    "minecraft:crafting_shapeless": ("minecraft:crafting_table", None, 200, 2),
    "minecraft:smelting": ("minecraft:furnace", [300], 302, 220),
    "minecraft:blasting": ("minecraft:blast_furnace", [300], 302, 120),
    "minecraft:smoking": ("minecraft:smoker", [300], 302, 120),
    "minecraft:smithing_transform": ("minecraft:smithing_table", [300, 301, 302], 303, 4),
}
FUEL = "minecraft:coal"
COMPOUND = {"none", "c_damage", "c_fall", "c_jump", "c_walk", "c_attack", "c_interact_entity", "c_station",
            "c_entity_attr"}
NUMBER = r"(-?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)"


# ---------------------------------------------------------------- discovery

def _json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def read_nbt(path: Path) -> dict:
    """A (gzipped) NBT file as Python values: a structure template's size and palette."""
    raw = path.read_bytes()
    if raw[:2] == b"\x1f\x8b":
        raw = gzip.decompress(raw)
    buf = io.BytesIO(raw)

    def r(fmt: str):
        return struct.unpack(">" + fmt, buf.read(struct.calcsize(">" + fmt)))[0]

    def string() -> str:
        return buf.read(r("H")).decode("utf-8", "replace")

    def payload(tag: int):
        if tag == 1:
            return r("b")
        if tag == 2:
            return r("h")
        if tag == 3:
            return r("i")
        if tag == 4:
            return r("q")
        if tag == 5:
            return r("f")
        if tag == 6:
            return r("d")
        if tag == 7:
            return buf.read(r("i"))
        if tag == 8:
            return string()
        if tag == 9:
            inner, n = r("b"), r("i")
            return [payload(inner) for _ in range(n)]
        if tag == 10:
            out = {}
            while True:
                t = r("b")
                if t == 0:
                    return out
                name = string()
                out[name] = payload(t)
        if tag == 11:
            return [r("i") for _ in range(r("i"))]
        if tag == 12:
            return [r("q") for _ in range(r("i"))]
        raise ValueError(f"bad NBT tag {tag}")

    if r("b") != 10:
        raise ValueError("not an NBT compound")
    string()
    return payload(10)


def template_info(path: Path) -> dict:
    try:
        data = read_nbt(path)
    except (OSError, ValueError, struct.error, EOFError):
        return {"size": [1, 1, 1], "palette": []}
    palettes = data.get("palette") or (data.get("palettes") or [[]])[0]
    return {"size": [int(v) for v in data.get("size", [1, 1, 1])],
            "palette": sorted({p.get("Name", "") for p in palettes if isinstance(p, dict)})}


def discover(content_dir: Path, mod_id: str, namespaces: list[str], block_ids: set[str], contract: dict) -> dict:
    """The entities, worldgen, structure templates, item tags and full recipes a mod adds."""
    data = content_dir / "data"
    out: dict = {"entities": [], "worldgen": [], "templates": [], "tags": {}, "recipe_data": {}}
    by_name = {c["id"]: c for c in contract.get("entities", []) if isinstance(c, dict) and "id" in c}
    for p in sorted((data / mod_id / "horizon" / "entity_type").glob("*.json")):
        edata = _json(p)
        loot = _json(data / mod_id / "loot_table" / "entities" / f"{p.stem}.json")
        out["entities"].append({"id": f"{mod_id}:{p.stem}", "name": p.stem, "template": edata.get("template", ""),
                                "attributes": edata.get("attributes", {}), "sure_drops": sure_drops(loot),
                                "contract": by_name.get(p.stem, {})})
    for ns in namespaces:
        for kind in ("feature", "configured_feature", "placed_feature", "structure", "template_pool"):
            for p in sorted((data / ns / "worldgen" / kind).rglob("*.json")):
                rel = p.relative_to(data / ns / "worldgen" / kind).with_suffix("").as_posix()
                text = p.read_text(encoding="utf-8", errors="replace")
                out["worldgen"].append({"kind": kind, "id": f"{ns}:{rel}", "file": str(p), "text": text,
                                        "mod_blocks": sorted(b for b in block_ids if f'"{b}"' in text or f'"{b}[' in text)})
        for p in sorted((data / ns / "structure").rglob("*.nbt")):
            rel = p.relative_to(data / ns / "structure").with_suffix("").as_posix()
            info = template_info(p)
            out["templates"].append({"id": f"{ns}:{rel}", **info,
                                     "mod_blocks": [b for b in info["palette"] if b in block_ids]})
    # Item tags of every namespace the mod ships (its own, c:, minecraft: additions).
    if data.is_dir():
        for nsdir in data.iterdir():
            for folder in ("item", "items"):
                root = nsdir / "tags" / folder
                for p in root.rglob("*.json"):
                    rel = p.relative_to(root).with_suffix("").as_posix()
                    values = _json(p).get("values", [])
                    out["tags"].setdefault(f"{nsdir.name}:{rel}", []).extend(
                        v if isinstance(v, str) else v.get("id", "") for v in values)
    for p in sorted((data / mod_id / "recipe").glob("*.json")):
        out["recipe_data"][f"{mod_id}:{p.stem}"] = _json(p)
    # A structure's mod blocks: those of the templates its pools name.
    templates = {t["id"]: t for t in out["templates"]}
    pools = {w["id"]: w for w in out["worldgen"] if w["kind"] == "template_pool"}
    for w in out["worldgen"]:
        if w["kind"] != "structure":
            continue
        names = set(w["mod_blocks"])
        for pool_id in re.findall(r'"start_pool"\s*:\s*"([^"]+)"', w["text"]):
            pool = pools.get(pool_id if ":" in pool_id else f"minecraft:{pool_id}")
            for loc in re.findall(r'"location"\s*:\s*"([^"]+)"', pool["text"] if pool else ""):
                names |= set((templates.get(loc) or {}).get("mod_blocks", []))
        w["mod_blocks"] = sorted(names)
    return out


def sure_drops(loot: dict) -> list[str]:
    """The items a loot table always drops: plain item entries in pools with no conditions."""
    drops = []
    for pool in loot.get("pools", []):
        if pool.get("conditions"):
            continue
        for entry in pool.get("entries", []):
            if entry.get("type") == "minecraft:item" and "name" in entry and not entry.get("conditions"):
                if not any("random_chance" in json.dumps(f) or "looting" in json.dumps(f) for f in entry.get("functions", [])):
                    drops.append(entry["name"])
    return drops


# ---------------------------------------------------------------- plan pieces

def tp(stage: str, x: float, y: float, z: float) -> dict:
    return {"stage": stage, "cmd": "command", "args": {"command": f"tp @s {x} {y} {z}"}}


def raw(stage: str, *words, **extra) -> dict:
    return {"stage": stage, "cmd": "raw", "args": {"words": [str(w) for w in words]}, **extra}


def command(stage: str, text: str, **extra) -> dict:
    return {"stage": stage, "cmd": "command", "args": {"command": text}, **extra}


def wait(stage: str, ticks: int) -> dict:
    return {"stage": stage, "cmd": "wait", "args": {"ticks": ticks}}


def patch(stage: str, x1: int, z1: int, x2: int, z2: int, y: int, height: int = 6,
          floor: str = "minecraft:stone", depth: int = 1) -> list[dict]:
    """A cleared patch far from the plot: kept loaded, a floor `depth` deep under y, air above."""
    return [command(f"{stage}_forceload", f"forceload add {x1} {z1} {x2} {z2}"),
            tp(f"{stage}_go", (x1 + x2) / 2 + 0.5, y + 1, z1 - 1.5),
            wait(f"{stage}_loads", 40),
            {"stage": f"{stage}_floor", "cmd": "fill", "args": {"box": [x1, y - depth, z1, x2, y - 1, z2], "block": floor}},
            {"stage": f"{stage}_clear", "cmd": "fill", "args": {"box": [x1, y, z1, x2, y + height, z2], "block": "minecraft:air"}},
            wait(f"{stage}_ready", 5)]


def armor_slot(item: dict) -> str | None:
    c = item.get("contract", {})
    if c.get("armor_slot") in SLOT_ARGS:
        return c["armor_slot"]
    name = f"{item['template']} {item['name']}".lower()
    for word, slot in ARMOR_SLOTS.items():
        if re.search(rf"(?:^|[_:\s]){word}(?:$|[_\s])", name):
            return slot
    return None


def item_steps(spec: dict) -> list[dict]:
    """Level 1 for every item: it can be given and stacks to the original's size; worn armor gives the
    original's armor, toughness and attributes ("armor", "toughness", "worn": {attribute: value} in its
    contract); held, the contract's "held" attributes (attack damage, speed, reach)."""
    steps: list[dict] = []
    for it in spec["items"]:
        c = it.get("contract", {})
        tag = {"item": it["id"]}
        size = int(c.get("max_stack") or it.get("max_stack_size") or 64)
        steps += [command("1_item_clear", "clear", **tag),
                  {"stage": "1_item_give_stack", "cmd": "give", "args": {"item": it["id"], "count": size + 1}, **tag},
                  wait("1_item_sync", 4) | tag,
                  raw("1_item_stack", "inventory", **tag, check={"type": "stack_max", "item": it["id"], "max": size},
                      what=f"{it['id']} stacks to {size} (the original's)")]
        slot = armor_slot(it)
        worn = dict(c.get("worn") or {})
        if c.get("armor") is not None:
            worn.setdefault("minecraft:armor", c["armor"])
        if c.get("toughness") is not None:
            worn.setdefault("minecraft:armor_toughness", c["toughness"])
        if slot and slot not in ("mainhand", "offhand") and not worn:
            worn["minecraft:armor"] = ">0"
        if slot and worn:
            steps += [command("1_item_clear", "clear", **tag),
                      command("1_item_equip", f"item replace entity @s {SLOT_ARGS[slot]} with {it['id']}", **tag),
                      wait("1_item_equip_sync", 4) | tag]
            for attr, want in sorted(worn.items()):
                steps.append(command("1_item_worn", f"attribute @s {attr} get", **tag,
                                     check=number_check(want), what=f"{it['id']} worn ({slot}): {attr}"))
        held = c.get("held") or {}
        if held:
            steps += [command("1_item_clear", "clear", **tag),
                      {"stage": "1_item_hold", "cmd": "give", "args": {"item": it["id"], "count": 1}, **tag},
                      wait("1_item_hold_sync", 4) | tag,
                      {"stage": "1_item_select", "cmd": "select", "args": {"slot": it["id"]}, **tag},
                      wait("1_item_select_sync", 2) | tag]
            for attr, want in sorted(held.items()):
                steps.append(command("1_item_held", f"attribute @s {attr} get", **tag,
                                     check=number_check(want), what=f"{it['id']} held: {attr}"))
    if spec["items"]:
        steps.append(command("1_item_done", "clear"))
    return steps


def number_check(want) -> dict:
    """A check of the number a command reports ("... is 20.0"): want 20, ">0", ">=3.5", "$var * 2"."""
    op, value = parse_op(want)
    return {"type": "number", "regex": rf"is {NUMBER}|{NUMBER}\s*$", "op": op, "value": value}


def entity_steps(spec: dict, plot_index: int) -> list[dict]:
    """Level 1 for every entity type: summoned, it exists, is drawn (not invisible or the magenta
    missing texture), has the original's attributes, and dies dropping what its loot table always drops."""
    steps: list[dict] = []
    oz, y = 2048 + plot_index * 64, 100
    for n, e in enumerate(spec.get("entities", [])):
        ox = n * 8
        tag = {"entity": e["id"]}
        key = f"{spec['id']}_entity_{e['name']}"
        living = not NOT_LIVING.match(e["template"] or "")
        nbt = "{NoAI:1b,PersistenceRequired:1b,Silent:1b}" if living else "{NoGravity:1b}"
        at = (ox + 0.5, y, oz + 0.5)
        part = patch("11_entity_patch", ox - 3, oz - 4, ox + 3, oz + 3, y, height=5)
        part += [tp("11_entity_stand", ox + 0.5, y, oz - 3.5), wait("11_entity_settle", 5),
                 {"stage": "11_entity_aim_empty", "cmd": "look_at", "args": {"x": at[0], "y": y + 0.6, "z": at[2]}},
                 {"stage": "11_entity_empty", "cmd": "screenshot", "args": {"path": f"{key}_empty.png"}},
                 command("11_entity_summon", f"summon {e['id']} {at[0]} {at[1]} {at[2]} {nbt}"),
                 wait("11_entity_lands", 10),
                 raw("11_entity_exists", "entities", 8, e["id"], check={"type": "count", "item": "*", "op": ">=", "value": 1},
                     what=f"{e['id']} summoned"),
                 {"stage": "11_entity_aim", "cmd": "look_at", "args": {"x": at[0], "y": y + 0.6, "z": at[2]}},
                 {"stage": "11_entity_render", "cmd": "screenshot", "args": {"path": f"{key}_world.png"},
                  "render_check": key}]
        attrs = dict(e.get("attributes") or {})
        attrs.update(e.get("contract", {}).get("attributes_values") or {})
        for attr, want in sorted(attrs.items()) if living else []:
            part.append({"stage": "11_entity_attribute", "cmd": "c_entity_attr", "args": {"type": e["id"], "attribute": attr},
                         "check": {"type": "value_of", "op": "==", "value": float(want)}, "what": f"{e['id']} {attr}"})
        part += [command("11_entity_kill", f"kill @e[type={e['id']}]"), wait("11_entity_drops", 10)]
        for drop in e.get("sure_drops", []) if living else []:
            part.append(raw("11_entity_drop", "entities", 8, "item", check={"type": "count", "item": drop, "op": ">=", "value": 1},
                            what=f"{e['id']} killed drops {drop} (its loot table)"))
        part.append(command("11_entity_cleanup", "kill @e[type=item,distance=..16]"))
        for st in part:
            st.update(tag)
        steps += part
    return steps


def worldgen_steps(spec: dict, plot_index: int) -> list[dict]:
    """Level 1 for every feature, template and structure: /place puts it in the world, and the blocks it
    names (or its templates' palettes name) of the mod are there afterwards."""
    steps: list[dict] = []
    oz, y = 4096 + plot_index * 512, 100
    n = 0
    configured = {w["id"]: w for w in spec.get("worldgen", []) if w["kind"] in ("feature", "configured_feature")}
    for w in spec.get("worldgen", []):
        if w["kind"] == "template_pool":
            continue
        tag = {"feature": w["id"]}
        ox = n * 40
        n += 1
        if w["kind"] == "structure":
            sz = oz + 256
            part = [command("12_structure_forceload", f"forceload add {ox - 16} {sz - 16} {ox + 16} {sz + 16}"),
                    tp("12_structure_go", ox + 0.5, y + 20, sz + 0.5), wait("12_structure_loads", 100),
                    command("12_structure_place", f"place structure {w['id']} {ox} {y} {sz}",
                            what=f"/place structure {w['id']}")]
            tp_at = (ox + 0.5, y, sz + 0.5)
        else:
            feature = w
            if w["kind"] == "placed_feature":
                ref = re.search(r'"feature"\s*:\s*"([^"]+)"', w["text"])
                feature = configured.get(ref.group(1)) if ref else None
                if feature is None:
                    # A placed feature over a vanilla configured feature, or inline: placed as itself.
                    feature = {"id": ref.group(1) if ref else w["id"], "text": w["text"]}
            buried = re.search(r"minecraft:(?:ore|scattered_ore|replace_single_block|replace_blob|disk)", feature["text"])
            origin_y = y - 6 if buried else y
            part = patch("12_feature_patch", ox - 8, oz - 8, ox + 8, oz + 8, y, height=16, depth=12)
            part += [command("12_feature_top", f"fill {ox - 8} {y - 1} {oz - 8} {ox + 8} {y - 1} {oz + 8} minecraft:grass_block"),
                     raw("12_feature_before", "blocks", ox - 8, y - 12, oz - 8, ox + 8, y + 15, oz + 8,
                         save={"name": f"wg_before_{n}", "total": True}),
                     command("12_feature_place", f"place feature {feature['id']} {ox} {origin_y} {oz}",
                             what=f"/place feature {feature['id']} (for {w['id']})")]
            tp_at = (ox + 0.5, y, oz - 9.5)
            if not w["mod_blocks"]:
                part += [wait("12_feature_lands", 5),
                         raw("12_feature_after", "blocks", ox - 8, y - 12, oz - 8, ox + 8, y + 15, oz + 8,
                             save={"name": f"wg_after_{n}", "total": True}),
                         {"stage": "12_feature_changed", "cmd": "none", "args": {},
                          "check": {"type": "value", "expr": f"$wg_after_{n} != $wg_before_{n}"},
                          "what": f"{w['id']} placed changes the world"}]
        part += [wait("12_worldgen_lands", 10), tp("12_worldgen_look", *tp_at), wait("12_worldgen_settle", 10)]
        for block in w["mod_blocks"]:
            part.append(raw("12_worldgen_has", "find-block", block, 64,
                            check={"type": "found", "op": ">=", "value": 1}, what=f"{w['id']} places {block}"))
        for st in part:
            st.update(tag)
        steps += part
    for t in spec.get("templates", []):
        tag = {"feature": t["id"]}
        ox = n * 40
        n += 1
        sx, sy, sz = (max(1, v) for v in t["size"])
        part = patch("12_template_patch", ox - 1, oz - 1, ox + min(sx, 48), oz + min(sz, 48), y, height=min(sy, 48), depth=1)
        part += [command("12_template_place", f"place template {t['id']} {ox} {y} {oz}", what=f"/place template {t['id']}"),
                 wait("12_template_lands", 10)]
        for block in t["mod_blocks"]:
            part.append(raw("12_template_has", "find-block", block, min(64, max(sx, sy, sz) + 8),
                            check={"type": "found", "op": ">=", "value": 1}, what=f"{t['id']} holds {block}"))
        if not t["mod_blocks"] and t["palette"]:
            part.append(raw("12_template_nonempty", "blocks", ox, y, oz, ox + sx - 1, y + sy - 1, oz + sz - 1,
                            check={"type": "total", "op": ">=", "value": 1}, what=f"{t['id']} placed holds blocks"))
        for st in part:
            st.update(tag)
        steps += part
    return steps


def resolve_ingredient(ing, tags: dict, vanilla: dict, depth: int = 0) -> str | None:
    """One concrete item that satisfies a recipe ingredient (an id, "#tag", {"item"}, {"tag"} or a list)."""
    if depth > 8 or ing is None:
        return None
    if isinstance(ing, list):
        for alt in ing:
            got = resolve_ingredient(alt, tags, vanilla, depth + 1)
            if got:
                return got
        return None
    if isinstance(ing, dict):
        if "item" in ing or "id" in ing:
            return ing.get("item") or ing.get("id")
        if "tag" in ing:
            return resolve_ingredient("#" + ing["tag"], tags, vanilla, depth + 1)
        return None
    ing = str(ing)
    if not ing.startswith("#"):
        return ing if ":" in ing else f"minecraft:{ing}"
    tag = ing[1:] if ":" in ing else f"minecraft:{ing[1:]}"
    values = list(tags.get(tag, [])) + list((vanilla.get("tags") or {}).get(f"item:{tag}", []))
    for v in values:
        got = resolve_ingredient(v, tags, vanilla, depth + 1)
        if got:
            return got
    return None


def recipe_steps(spec: dict, all_tags: dict, base_x: int, base_y: int, base_z: int) -> list[dict]:
    """Stage 6: every crafting, smelting and smithing recipe made as a player makes it, in its station,
    with the result checked. Other types (the mod's own machines, stonecutting) are marked skipped:
    a contract scenario covers them."""
    vanilla = contract_check.vanilla_registry()
    steps: list[dict] = []
    station_pos = (base_x - 1, base_y, base_z - 3)
    stand = (base_x - 0.5, base_y, base_z - 1.5)
    for rid, r in sorted(spec.get("recipe_data", {}).items()):
        rtype = r.get("type", "")
        tag = {"recipe": rid}
        if rtype not in STATIONS:
            steps.append({"stage": "6_verify_recipe", "recipe_id": rid, "result": r.get("result"), **tag,
                          "skipped_why": f"type {rtype}: a contract scenario must make it (its machine)"})
            continue
        block, slots, result_slot, ticks = STATIONS[rtype]
        inputs: list[tuple[int, str, int]] = []
        missing = []
        if rtype == "minecraft:crafting_shaped":
            for row, line in enumerate(r.get("pattern", [])):
                for col, ch in enumerate(line):
                    if ch == " ":
                        continue
                    item = resolve_ingredient(r.get("key", {}).get(ch), all_tags, vanilla)
                    (inputs.append((100 + row * 3 + col, item, 1)) if item else missing.append(ch))
        elif rtype == "minecraft:crafting_shapeless":
            for i, ing in enumerate(r.get("ingredients", [])):
                item = resolve_ingredient(ing, all_tags, vanilla)
                (inputs.append((100 + i, item, 1)) if item else missing.append(str(ing)))
        elif rtype == "minecraft:smithing_transform":
            for slot, key in zip(slots, ("template", "base", "addition")):
                item = resolve_ingredient(r.get(key), all_tags, vanilla)
                (inputs.append((slot, item, 1)) if item else missing.append(key))
        else:
            item = resolve_ingredient(r.get("ingredient"), all_tags, vanilla)
            (inputs.append((300, item, 1)) if item else missing.append("ingredient"))
            inputs.append((301, FUEL, 1))
            ticks = int(r.get("cookingtime") or ticks - 20) + 20
        result = r.get("result") or {}
        rid_out = result if isinstance(result, str) else result.get("id") or result.get("item")
        count = 1 if isinstance(result, str) else int(result.get("count", 1))
        if missing or not rid_out:
            steps.append({"stage": "6_verify_recipe", "recipe_id": rid, "result": result, **tag,
                          "skipped_why": f"ingredient(s) {missing} name a tag no loaded mod or vanilla fills"})
            continue
        steps.append({"stage": "6_craft_recipe", "cmd": "c_station", **tag,
                      "args": {"block": block, "pos": list(station_pos), "stand": list(stand), "inputs": inputs,
                               "result_slot": result_slot, "ticks": ticks},
                      "check": {"type": "slot", "slot": result_slot, "item": rid_out, "op": "==", "value": count},
                      "what": f"{rtype} {rid} in a {block} makes {count} {rid_out}"})
    if any(s.get("cmd") == "c_station" for s in steps):
        steps += [{"stage": "6_station_close", "cmd": "close", "args": {}, "allow_kinds": ["state"]},
                  {"stage": "6_station_remove", "cmd": "setblock",
                   "args": {"x": station_pos[0], "y": station_pos[1], "z": station_pos[2], "block": "minecraft:air"}},
                  command("6_clear", "clear")]
    return steps


# ---------------------------------------------------------------- scenario steps

# ---------------------------------------------------------------- builds and multiblocks

DIRS4 = ["north", "east", "south", "west"]
# Scenario step keys holding positions: how many numbers (in threes) each starts with.
POS_KEYS = {"setblock": 3, "expect_block": 3, "expect_data": 3, "insert": 3, "expect_container": 3, "blocks": 6,
            "expect_blocks": 6, "break": 3, "remove": 3, "data": 3, "tp": 3, "interact": 3, "on": 3, "at": 3}


def rotate_state(block: str, turns: int) -> str:
    """A block string turned clockwise (seen from above) `turns` quarter turns: facing, the sides a
    wire connects to, axis and sign rotation turn with it."""
    turns %= 4
    if not turns or "[" not in block:
        return block
    head, _, nbt = block.partition("{")
    base, _, rest = head.partition("[")
    out = []
    for prop in rest.rstrip("]").split(","):
        k, eq, v = prop.partition("=")
        if k in DIRS4:
            k = DIRS4[(DIRS4.index(k) + turns) % 4]
        if v in DIRS4:
            v = DIRS4[(DIRS4.index(v) + turns) % 4]
        if k == "axis" and turns % 2 and v in ("x", "z"):
            v = "z" if v == "x" else "x"
        if k == "rotation" and v.isdigit():
            v = str((int(v) + 4 * turns) % 16)
        out.append(f"{k}={v}" if eq else k)
    return f"{base}[{','.join(out)}]" + (f"{{{nbt}" if nbt else "")


def rotate_xz(x: float, z: float, turns: int, w: int, d: int) -> tuple[float, float]:
    """(x, z) inside a w-by-d footprint, turned clockwise; the footprint keeps its north-west corner."""
    turns %= 4
    if turns == 1:
        return d - 1 - z, x
    if turns == 2:
        return w - 1 - x, d - 1 - z
    if turns == 3:
        return z, w - 1 - x
    return x, z


def cell_spec(spec) -> tuple[str, str, bool]:
    """A layout key's value: "mod:block[state]", or {"block", "item", "by_player"} -> (block, item, by player)."""
    if isinstance(spec, dict):
        block = spec.get("block") or spec.get("item")
        item = spec.get("item") or block.split("[")[0].split("{")[0]
        return block, item, bool(spec.get("by_player", "item" in spec))
    return str(spec), str(spec).split("[")[0].split("{")[0], False


def layout(build: dict) -> tuple[list[tuple[int, int, int, object]], tuple[int, int, int]]:
    """A build's cells as (x, y, z, spec) before turning, and its size (w, h, d). `layers` run bottom to
    top, each a list of rows north to south, each row's characters west to east: a key letter, " " for
    air (cleared) or "." for whatever is there. `blocks` lists [x, y, z, spec] instead."""
    cells = []
    if "blocks" in build:
        cells = [(int(c[0]), int(c[1]), int(c[2]), c[3]) for c in build["blocks"]]
    for y, layer in enumerate(build.get("layers", [])):
        for z, row in enumerate([layer] if isinstance(layer, str) else layer):
            for x, ch in enumerate(row):
                if ch == ".":
                    continue
                if ch != " " and ch not in build.get("key", {}):
                    raise ValueError(f"build layer {y} row {z}: {ch!r} is not in its key")
                cells.append((x, y, z, "minecraft:air" if ch == " " else build["key"][ch]))
    if not cells:
        return [], (1, 1, 1)
    return cells, (max(c[0] for c in cells) + 1, max(c[1] for c in cells) + 1, max(c[2] for c in cells) + 1)


def rotate_step(st: dict, turns: int, size: tuple[int, int, int], origin=(0, 0, 0)) -> dict:
    """A scenario step with every position (relative to the build at `origin`) and block turned."""
    if not turns % 4 or not isinstance(st, dict):
        return st
    w, _, d = size
    out = dict(st)
    for key, n in POS_KEYS.items():
        v = out.get(key)
        if isinstance(v, list) and len(v) >= n and all(isinstance(c, (int, float)) for c in v[:n]):
            v = list(v)
            for i in range(0, n, 3):
                x, z = rotate_xz(v[i] - origin[0], v[i + 2] - origin[2], turns, w, d)
                v[i], v[i + 2] = x + origin[0], z + origin[2]
            if key == "blocks" or key == "expect_blocks":  # keep the box's corners ordered
                for i in (0, 2):
                    v[i], v[i + 3] = min(v[i], v[i + 3]), max(v[i], v[i + 3])
            if key == "setblock" and len(v) > 3:
                v[3] = rotate_state(str(v[3]), turns)
            out[key] = v
    return out


def expand_multiblock(mb: dict, default_for: list[str]) -> list[dict]:
    """A contract's multiblock (a ritual, an altar, a machine built of parts, a powered network) as the
    scenarios that test it: built complete in each of its rotations it forms (after its trigger: the last
    block placed by a player, a right-click, a wait) and `formed` holds; with one block missing,
    `incomplete` holds (nothing summoned, no power); built and then one block broken, `broken` holds."""
    build = mb["build"] if isinstance(mb["build"], dict) else {"blocks": mb["build"]}
    cells, size = layout(build)
    origin = tuple(build.get("at", [0, 0, 0]))
    solid = [c for c in cells if cell_spec(c[3])[0] != "minecraft:air"]
    plain = [c for c in solid if not cell_spec(c[3])[2]] or solid
    omit = list(mb.get("omit") or (plain[0][:3] if plain else [0, 0, 0]))
    gone = list(mb.get("break") or omit)
    base = {"for": mb.get("for", default_for), "level": mb.get("level", 4), "covers": mb.get("covers", []),
            "area": [max(size[0], size[2]) + 7, max(size[0], size[2]) + 7, size[1] + 4], "multiblock": mb.get("name")}
    name = mb.get("name", "multiblock")
    wait_step = [{"wait": int(mb["wait"])}] if mb.get("wait") else []
    out = []
    for rot in mb.get("rotations", [0]):
        turns = (int(rot) // 90) % 4

        def rs(steps):
            return [rotate_step(x, turns, size, origin) for x in steps]
        steps = rs(mb.get("before", [])) + [{"build": {**build, "rotate": rot}}] + rs(mb.get("trigger", [])) + wait_step
        steps += rs(mb.get("formed", []))
        if mb.get("broken"):
            gx, gy, gz = (int(v) for v in gone)
            steps += rs([{"break": [gx + origin[0], gy + origin[1], gz + origin[2]]}]) + (wait_step or [{"wait": 20}])
            steps += rs(mb["broken"])
        out.append({**base, "name": f"{name} (built" + (f", turned {rot}" if turns else "") + ")", "steps": steps})
    if mb.get("incomplete"):
        steps = mb.get("before", []) + [{"build": {**build, "omit": omit}}] + mb.get("trigger", []) + (wait_step or [{"wait": 40}])
        out.append({**base, "name": f"{name} (one block missing)", "steps": steps + mb["incomplete"]})
    return out


def stand_for(pos: tuple[int, int, int], ctx: dict) -> tuple[float, float, float]:
    """Where the player stands to reach `pos` by hand: on the patch floor, out of every built cell,
    within reach of the block."""
    px, py, pz = pos
    oy = ctx["oy"]
    x1, z1, x2, z2 = ctx["bounds"]
    occupied = ctx["occupied"]
    for r in (2, 3, 1, 4):
        for ddx, ddz in ((0, -r), (-r, 0), (r, 0), (0, r), (-r, -r), (r, -r), (-r, r), (r, r)):
            fx, fz = px + ddx, pz + ddz
            if not (x1 <= fx <= x2 and z1 <= fz <= z2):
                continue
            if (fx, oy, fz) in occupied or (fx, oy + 1, fz) in occupied:
                continue
            if math.dist((fx + 0.5, oy + 1.62, fz + 0.5), (px + 0.5, py + 0.5, pz + 0.5)) > 4.4:
                continue
            return fx + 0.5, oy, fz + 0.5
    return ctx["stand"]


def build_steps(build: dict, at, ctx: dict) -> list[dict]:
    """A build placed: every cell by /setblock (neighbour updates included) bottom to top, then the
    cells a player places (`"player": "last"` the last cell, `"all"` every cell, or the key's
    {"by_player": true} cells) by hand with horizon-ctl place, so placement logic runs (the wither's
    skulls, chalk, a ritual's last block, a part that links on placement)."""
    cells, size = layout(build)
    turns = (int(build.get("rotate", 0)) // 90) % 4
    origin = build.get("at", [0, 0, 0])
    omit = tuple(build["omit"]) if build.get("omit") else None
    mode = build.get("player", "marked")
    last = tuple(build["last"]) if build.get("last") else None
    placed = []
    for x, y, z, spec in cells:
        if (x, y, z) == omit:
            continue
        block, item, by_player = cell_spec(spec)
        by_player = by_player or mode == "all"
        rx, rz = rotate_xz(x, z, turns, size[0], size[2])
        rel = [rx + origin[0], y + origin[1], rz + origin[2]]
        placed.append({"rel": rel, "cell": (x, y, z), "block": rotate_state(block, turns), "item": item,
                       "player": by_player and block != "minecraft:air"})
    if mode == "last" and placed:
        target = next((p for p in placed if p["cell"] == last), None) if last else None
        target = target or [p for p in placed if p["block"] != "minecraft:air"][-1]
        target["player"] = True
        placed.remove(target)
        placed.append(target)
    for p in placed:
        a = at(p["rel"])
        if p["block"] != "minecraft:air":
            ctx["occupied"].add((a["x"], a["y"], a["z"]))
        else:
            ctx["occupied"].discard((a["x"], a["y"], a["z"]))
    steps = []
    for p in (p for p in placed if not p["player"]):
        a = at(p["rel"])
        steps.append({"stage": "10_build_setblock", "cmd": "setblock", "args": {**a, "block": p["block"]}})
    steps.append(wait("10_build_lands", 5))
    for p in (p for p in placed if p["player"]):
        a = at(p["rel"])
        sx, sy, sz = stand_for((a["x"], a["y"], a["z"]), ctx)
        steps += [{"stage": "10_build_give", "cmd": "give", "args": {"item": p["item"], "count": 1}},
                  wait("10_build_sync", 4),
                  tp("10_build_stand", sx, sy, sz), wait("10_build_settle", 4),
                  raw("10_build_place", "place", a["x"], a["y"], a["z"], "--item", p["item"],
                      what=f"a player places {p['item']} at {p['rel']}"),
                  wait("10_build_placed", 4)]
    if any(p["player"] for p in placed):
        steps += [tp("10_build_back", *ctx["stand"]), wait("10_build_back_settle", 3)]
    return steps


def scenario_step(st: dict, at, stand, ctx: dict | None = None) -> list[dict] | None:
    """The plan steps for one of the extended scenario step kinds (planning/harness.md, "Scenario
    steps"), or None when `st` is not one of them. `at(rel)` turns [dx, dy, dz] into {"x", "y", "z"};
    `ctx` is the scenario's patch (its floor y, bounds, stand point and the cells built so far)."""
    ctx = ctx if ctx is not None else {"oy": int(stand[1]), "bounds": (-10**9, -10**9, 10**9, 10**9),
                                       "occupied": set(), "stand": stand}
    if "build" in st:
        build = st["build"] if isinstance(st["build"], dict) else {"blocks": st["build"]}
        return build_steps(build, at, ctx)
    if "break" in st or "remove" in st:
        a = at(st.get("break") or st.get("remove"))
        ctx["occupied"].discard((a["x"], a["y"], a["z"]))
        if "remove" in st:
            return [{"stage": "10_scenario_remove", "cmd": "setblock", "args": {**a, "block": "minecraft:air"}},
                    wait("10_scenario_removed", 5)]
        sx, sy, sz = stand_for((a["x"], a["y"], a["z"]), ctx)
        return [tp("10_scenario_break_stand", sx, sy, sz), wait("10_scenario_break_settle", 4),
                raw("10_scenario_break", "break", a["x"], a["y"], a["z"], what=f"a player breaks {st.get('break')}"),
                wait("10_scenario_broken", 5), tp("10_scenario_break_back", *ctx["stand"])]
    if "data" in st:
        d = st["data"]
        if isinstance(d, dict):
            text = f"data get entity {d['entity']} {d.get('path', '')}".rstrip()
        else:
            a = at(d[:3])
            text = f"data get block {a['x']} {a['y']} {a['z']} {d[3] if len(d) > 3 else ''}".rstrip()
        name = st.get("save") if isinstance(st.get("save"), str) else (st.get("save") or {}).get("name", "data")
        return [command("10_scenario_data", text, save={"name": name, "regex": rf"data: {NUMBER}|{NUMBER}[bsLfd]?\s*$"})]
    save = st.get("save")
    if isinstance(save, str):
        save = {"name": save}

    def with_save(step: dict, default_path: str | None = "value") -> dict:
        if save:
            s = dict(save)
            if "path" not in s and "regex" not in s and "total" not in s and default_path:
                s["path"] = default_path
            step["save"] = s
        return step

    if "give" in st:
        return [{"stage": "10_scenario_give", "cmd": "give", "args": {"item": st["give"], "count": st.get("count", 1)}},
                wait("10_scenario_sync", 4)]
    if "equip" in st:
        slot = SLOT_ARGS.get(st.get("slot", "chest"), st.get("slot", "armor.chest"))
        return [command("10_scenario_equip", f"item replace entity @s {slot} with {st['equip']} {st.get('count', 1)}"),
                wait("10_scenario_equip_sync", 4)]
    if "hold" in st:
        return [{"stage": "10_scenario_hold", "cmd": "give", "args": {"item": st["hold"], "count": st.get("count", 1)}},
                wait("10_scenario_sync", 4),
                {"stage": "10_scenario_select", "cmd": "select", "args": {"slot": st["hold"].split("[")[0].split("{")[0]}}]
    if "use_item" in st:
        secs = st["use_item"]
        words = ["use-item"] + (["--hold", secs] if isinstance(secs, (int, float)) and not isinstance(secs, bool) else [])
        return [with_save(raw("10_scenario_use_item", *words), None), wait("10_scenario_after", 5)]
    if "key" in st:
        if st.get("seconds"):
            return [raw("10_scenario_key", "hold", st["key"], st["seconds"])]
        return [raw("10_scenario_key", "key", st["key"], st.get("action", "tap"))]
    if "tp" in st:
        p = at(st["tp"])
        return [tp("10_scenario_tp", p["x"] + 0.5, p["y"], p["z"] + 0.5), wait("10_scenario_tp_settle", 5)]
    if "look" in st:
        return [raw("10_scenario_look", "look", *st["look"])]
    if "summon" in st:
        p = at(st.get("at", [0, 0, 2]))
        return [command("10_scenario_summon", f"summon {st['summon']} {p['x'] + 0.5} {p['y']} {p['z'] + 0.5} {st.get('nbt', '')}".rstrip()),
                wait("10_scenario_summon_lands", 5)]
    if "attack" in st:
        return [with_save({"stage": "10_scenario_attack", "cmd": "c_attack",
                           "args": {"type": st["attack"], "radius": st.get("radius", 8)}})]
    if "interact_entity" in st:
        return [with_save({"stage": "10_scenario_interact_entity", "cmd": "c_interact_entity",
                           "args": {"type": st["interact_entity"], "radius": st.get("radius", 8)}}, None)]
    if "damage" in st:
        return [with_save({"stage": "10_scenario_damage", "cmd": "c_damage",
                           "args": {"amount": st["damage"], "type": st.get("type", "minecraft:mob_attack")}})]
    if "fall" in st:
        return [with_save({"stage": "10_scenario_fall", "cmd": "c_fall", "args": {"height": st["fall"]}})]
    if "jump_height" in st:
        return [with_save({"stage": "10_scenario_jump", "cmd": "c_jump", "args": {}})]
    if "walk_speed" in st:
        return [with_save({"stage": "10_scenario_walk", "cmd": "c_walk",
                           "args": {"seconds": st["walk_speed"], "sprint": bool(st.get("sprint")),
                                    "stand": list(stand)}})]
    if "state" in st:
        return [with_save(raw("10_scenario_state", "state"), st["state"])]
    if "attribute" in st:
        target = st.get("target", "@s")
        step = command("10_scenario_attribute", f"attribute {target} {st['attribute']} get")
        if save:
            step["save"] = {"regex": rf"is {NUMBER}", **save}
        return [step]
    if "insert" in st:
        p = at(st["insert"][:3])
        return [raw("10_scenario_insert", "container", p["x"], p["y"], p["z"], st["insert"][3], st["insert"][4] if len(st["insert"]) > 4 else 1),
                wait("10_scenario_insert_lands", 2)]
    if "click_slot" in st:
        words = ["click-slot", st["click_slot"]] + (["right"] if st.get("right") else []) + (["shift"] if st.get("shift") else [])
        return [raw("10_scenario_click_slot", *words), wait("10_scenario_click_lands", 2)]
    if "place" in st:
        p = at(st.get("at", [0, 0, 0]))
        x, y, z = p["x"], p["y"], p["z"]
        where = f"{x} {z}" if st["place"] == "jigsaw_unused" else f"{x} {y} {z}"
        return [command("10_scenario_place", f"place {st['place']} {st['id']} {where}"), wait("10_scenario_place_lands", 10)]
    if "blocks" in st:
        a, b = at(st["blocks"][:3]), at(st["blocks"][3:6])
        step = raw("10_scenario_blocks", "blocks", a["x"], a["y"], a["z"], b["x"], b["y"], b["z"])
        if save:
            step["save"] = {"total": True, **save} if len(st["blocks"]) < 7 else {"count": st["blocks"][6], **save}
        return [step]
    # Expectations.
    if "expect_value" in st:
        return [{"stage": "10_scenario_expect", "cmd": "none", "args": {}, "check": {"type": "value", "expr": st["expect_value"]}}]
    if "expect_item" in st:
        item, want = st["expect_item"][0], st["expect_item"][1] if len(st["expect_item"]) > 1 else ">=1"
        op, value = parse_op(want)
        return [raw("10_scenario_expect", "inventory", check={"type": "count", "item": item, "op": op, "value": value})]
    if "expect_entity" in st:
        kind, want = st["expect_entity"][0], st["expect_entity"][1] if len(st["expect_entity"]) > 1 else ">=1"
        op, value = parse_op(want)
        return [raw("10_scenario_expect", "entities", st.get("radius", 16), kind,
                    check={"type": "count", "item": "*", "op": op, "value": value})]
    if "expect_container" in st:
        e = st["expect_container"]
        p = at(e[:3])
        op, value = parse_op(e[4] if len(e) > 4 else ">=1")
        return [raw("10_scenario_expect", "container", p["x"], p["y"], p["z"],
                    check={"type": "count", "item": e[3], "op": op, "value": value})]
    if "expect_slot" in st:
        e = st["expect_slot"]
        op, value = parse_op(e[2] if len(e) > 2 else ">=1")
        return [raw("10_scenario_expect", "menu", check={"type": "slot", "slot": e[0], "item": e[1], "op": op, "value": value})]
    if "expect_blocks" in st:
        e = st["expect_blocks"]
        a, b = at(e[:3]), at(e[3:6])
        op, value = parse_op(e[7] if len(e) > 7 else ">=1")
        return [raw("10_scenario_expect", "blocks", a["x"], a["y"], a["z"], b["x"], b["y"], b["z"],
                    check={"type": "count", "item": e[6], "op": op, "value": value})]
    if "expect_effect" in st:
        return [raw("10_scenario_expect", "effects", check={"type": "regex", "regex": st["expect_effect"]})]
    return None


def scenario_area(sc: dict) -> tuple[int, int, int]:
    """The patch a scenario needs, [width, depth, height] (a runway for walk_speed, room for a structure
    or a build)."""
    area = list(sc.get("area") or [6, 6])
    w, d = area[:2]
    h = area[2] if len(area) > 2 else 4
    for st in sc.get("steps", []):
        if isinstance(st.get("build"), (dict, list)):
            build = st["build"] if isinstance(st["build"], dict) else {"blocks": st["build"]}
            _, (bw, bh, bd) = layout(build)
            ax, ay, az = build.get("at", [0, 0, 0])
            side = max(bw + ax, bd + az) + 7
            w, d, h = max(w, side), max(d, side), max(h, bh + ay + 4)
    if any("walk_speed" in st for st in sc.get("steps", [])):
        d = max(d, 48)
    if any(st.get("place") == "structure" for st in sc.get("steps", [])):
        w, d, h = max(w, 48), max(d, 48), max(h, 24)
    return int(w), int(d), int(h)


# ---------------------------------------------------------------- run time

def parse_op(want) -> tuple[str, object]:
    """">=3" -> (">=", 3.0); 5 -> ("==", 5); "$x * 2" -> ("==", "$x * 2"); ">$bare" -> (">", "$bare")."""
    if isinstance(want, bool):
        return "==", float(want)
    if isinstance(want, (int, float)):
        return "==", float(want)
    m = re.match(r"\s*(>=|<=|==|!=|>|<|~)?\s*(.+)", str(want))
    op, rest = (m.group(1) or "=="), m.group(2).strip()
    try:
        return op, float(rest)
    except ValueError:
        return op, rest


_OPS = {ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul, ast.Div: operator.truediv,
        ast.Mod: operator.mod, ast.Pow: operator.pow, ast.USub: operator.neg, ast.UAdd: operator.pos,
        ast.Gt: operator.gt, ast.GtE: operator.ge, ast.Lt: operator.lt, ast.LtE: operator.le,
        ast.Eq: operator.eq, ast.NotEq: operator.ne}


def evaluate(expr, variables: dict):
    """A number or comparison over saved variables: "$jump_boots >= $jump_bare * 2", "abs($a - 4) < 0.1"."""
    if isinstance(expr, (int, float)):
        return expr
    names = {}
    text = re.sub(r"\$(\w+)", lambda m: names.setdefault(m.group(1), m.group(1)), str(expr))
    for n in names:
        if n not in variables:
            raise KeyError(f"${n} was never saved (an earlier step failed, or no step saves it)")
    funcs = {"abs": abs, "min": min, "max": max, "round": round, "sqrt": math.sqrt}

    def ev(node):
        if isinstance(node, ast.Expression):
            return ev(node.body)
        if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
            return node.value
        if isinstance(node, ast.Name):
            if node.id in variables:
                return variables[node.id]
            raise KeyError(f"unknown name {node.id}")
        if isinstance(node, ast.BinOp) and type(node.op) in _OPS:
            return _OPS[type(node.op)](ev(node.left), ev(node.right))
        if isinstance(node, ast.UnaryOp) and type(node.op) in _OPS:
            return _OPS[type(node.op)](ev(node.operand))
        if isinstance(node, ast.BoolOp):
            vals = [ev(v) for v in node.values]
            return all(vals) if isinstance(node.op, ast.And) else any(vals)
        if isinstance(node, ast.Compare):
            left = ev(node.left)
            for op, right in zip(node.ops, node.comparators):
                r = ev(right)
                if not _OPS[type(op)](left, r):
                    return False
                left = r
            return True
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in funcs:
            return funcs[node.func.id](*[ev(a) for a in node.args])
        raise ValueError(f"not allowed in an expression: {ast.dump(node)[:60]}")

    return ev(ast.parse(text, mode="eval"))


def compare(got: float, op: str, want, variables: dict) -> bool:
    want = evaluate(want, variables) if isinstance(want, str) else want
    if op == "~":
        return abs(got - want) <= max(0.05, abs(want) * 0.05)
    if op == "==":
        return abs(got - want) <= 1e-3 * max(1.0, abs(want))
    return _OPS[{">": ast.Gt, ">=": ast.GtE, "<": ast.Lt, "<=": ast.LtE, "!=": ast.NotEq}[op]](got, want)


def walk(obj):
    yield obj
    if isinstance(obj, dict):
        for v in obj.values():
            yield from walk(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from walk(v)


def stacks(reply) -> list[dict]:
    """Every item stack or block entry of a reply ({"item"|"block": id, "count"?})."""
    return [d for d in walk(reply) if isinstance(d, dict) and isinstance(d.get("item") or d.get("block"), str)]


def count_of(reply: dict, item: str) -> float:
    """How many of `item` a reply shows ("*": entries of any kind). A counts map ({"id": n}) wins over
    the stacks, which it sums; an entity list counts its entities."""
    if item == "*":
        for key in ("entities", "items", "blocks"):
            if isinstance(reply.get(key), list):
                return len(reply[key])
        return float(reply.get("count") or 0)
    base = item.split("[")[0]
    maps = [d[base] for d in walk(reply) if isinstance(d, dict) and isinstance(d.get(base), (int, float))]
    if maps:
        return max(maps)
    return sum(float(d.get("count", 1)) for d in stacks(reply) if (d.get("item") or d.get("block")) == base)


def total_of(reply: dict) -> float:
    counts = reply.get("counts")
    if isinstance(counts, dict):
        return sum(v for v in counts.values() if isinstance(v, (int, float)))
    return float(reply.get("total") or reply.get("count") or len(reply.get("blocks") or []))


def pick(reply, path: str):
    cur = reply
    for part in str(path).split("."):
        if isinstance(cur, list) and part.lstrip("-").isdigit():
            cur = cur[int(part)] if -len(cur) <= int(part) < len(cur) else None
        elif isinstance(cur, dict):
            cur = cur.get(part)
        else:
            return None
    return cur


def text_of(reply: dict) -> str:
    parts = [*(reply.get("chat") or []), reply.get("actionbar") or "", reply.get("feedback") or "",
             reply.get("output") or "", reply.get("message") or ""]
    joined = " | ".join(str(p) for p in parts if p)
    return joined or json.dumps(reply)


def number_in(reply: dict, regex: str) -> float | None:
    for source in (text_of(reply), json.dumps(reply)):
        for m in re.finditer(regex, source):
            for g in m.groups() or (m.group(0),):
                if g is not None:
                    try:
                        return float(g)
                    except ValueError:
                        continue
    return None


def save(step: dict, reply: dict, variables: dict) -> str | None:
    """Keep the value `step["save"]` names; a problem when the reply does not have it."""
    s = step.get("save")
    if not s:
        return None
    name = s.get("name") or "last"
    if "state_prop" in s:  # a block state's property, kept as text for a prop_changed check
        variables[name] = prop_of(reply, s["state_prop"])
        return None
    if s.get("total"):
        value = total_of(reply)
    elif "count" in s:
        value = count_of(reply, s["count"])
    elif "regex" in s:
        value = number_in(reply, s["regex"])
    else:
        value = pick(reply, s.get("path", "value"))
    if isinstance(value, bool):
        value = float(value)
    if not isinstance(value, (int, float)):
        return f"could not save ${name}: the reply has no number at {s} ({json.dumps(reply)[:200]})"
    variables[name] = float(value)
    return None


def prop_of(reply: dict, prop: str):
    state = reply.get("state", reply)
    if isinstance(state, dict) and prop in state:
        return str(state[prop])
    for d in walk(state):
        if isinstance(d, dict) and prop in d and not isinstance(d[prop], (dict, list)):
            return str(d[prop])
    m = re.search(rf"\b{re.escape(prop)}=([\w:-]+)", json.dumps(state))
    return m.group(1) if m else None


def check(step: dict, reply: dict, variables: dict) -> str | None:
    """What is wrong with `reply` against `step["check"]`, or None."""
    c = step["check"]
    what = step.get("what") or step.get("stage")
    kind = c["type"]
    try:
        if kind == "value":
            return None if evaluate(c["expr"], variables) else (
                f"{what}: {c['expr']} is false ({', '.join(f'${k}={v:g}' for k, v in sorted(variables.items()) if '$' + k in c['expr'])})")
        if kind == "prop_changed":
            before, after = variables.get(f"state_{c['prop']}"), prop_of(reply, c["prop"])
            if after is None:
                return f"{what}: the block has no {c['prop']} ({json.dumps(reply)[:200]})"
            return None if after != before else f"{what}: {c['prop']} stayed {after}"
        if kind == "regex":
            return None if re.search(c["regex"], json.dumps(reply)) else f"{what}: expected /{c['regex']}/ in {json.dumps(reply)[:300]}"
        if kind == "stack_max":
            sizes = [float(d.get("count", 0)) for d in stacks(reply) if d.get("item") == c["item"]]
            got = max(sizes) if sizes else 0
            return None if got == c["max"] else f"{what}: its fullest stack holds {got:g}, the original's {c['max']}"
        if kind in ("count", "found", "total", "value_of", "number", "slot"):
            if kind == "count":
                got = count_of(reply, c["item"])
            elif kind == "found":
                got = float(reply.get("count") or len(reply.get("blocks") or reply.get("found") or []))
            elif kind == "total":
                got = total_of(reply)
            elif kind == "value_of":
                got = reply.get("value")
            elif kind == "number":
                got = number_in(reply, c["regex"])
            else:
                slot = next((d for d in walk(reply) if isinstance(d, dict) and d.get("slot") == c["slot"]), None)
                if not slot or not slot.get("item"):
                    return f"{what}: slot {c['slot']} is empty ({json.dumps(reply)[:200]})"
                if not re.fullmatch(c["item"], slot["item"]):
                    return f"{what}: slot {c['slot']} holds {slot['item']}, expected {c['item']}"
                got = float(slot.get("count", 1))
            if got is None:
                return f"{what}: no number in the reply ({json.dumps(reply)[:200]})"
            if not compare(float(got), c.get("op", "=="), c.get("value", 1), variables):
                return f"{what}: got {float(got):g}, expected {c.get('op', '==')} {c.get('value')}"
            return None
    except (KeyError, ValueError, ZeroDivisionError, TypeError) as exc:
        return f"{what}: {exc}"
    return f"{what}: unknown check {kind!r}"


def _state(ctl) -> dict:
    return ctl("state")[1]


def _pos(state: dict) -> list[float]:
    p = state.get("position") or (state.get("player") or {}).get("position") or [0, 0, 0]
    return [float(v) for v in (p if isinstance(p, list) else [p.get("x", 0), p.get("y", 0), p.get("z", 0)])]


def _health(state: dict) -> float | None:
    h = state.get("health", (state.get("player") or {}).get("health"))
    return float(h) if isinstance(h, (int, float)) else None


def _on_ground(state: dict) -> bool:
    return bool(state.get("on_ground", (state.get("player") or {}).get("on_ground")))


def _survival(ctl) -> None:
    for c in ("effect clear @s", "gamemode survival", "effect give @s instant_health 1 20 true",
              "effect give @s saturation 1 20 true"):
        ctl("command", c)
    ctl("wait", 10)


def _restore(ctl, state: dict) -> None:
    if (state.get("screen") or "") == "death" or (_health(state) or 1) <= 0:
        ctl("respawn")
        ctl("wait", 20)
    ctl("command", "gamemode creative")
    ctl("command", "effect give @s instant_health 1 20 true")


def _entity(ctl, kind: str, radius: int):
    _, reply = ctl("entities", radius, kind)
    ents = reply.get("entities") or []
    return ents[0] if ents else None


def _number_near(reply, *words) -> float | None:
    for d in walk(reply):
        if isinstance(d, dict):
            for k, v in d.items():
                if isinstance(v, (int, float)) and all(w in k for w in words):
                    return float(v)
    return None


def _menu_slots(menu: dict) -> list[dict]:
    return [d for d in walk(menu) if isinstance(d, dict) and isinstance(d.get("slot"), int)]


def run_compound(ctl, step: dict) -> tuple[int, dict]:
    """Run a compound step (several horizon-ctl calls) and reply with {"ok", "value", ...}."""
    cmd, a = step["cmd"], step.get("args", {})
    if cmd == "none":
        return 0, {"ok": True}
    if cmd == "c_damage":
        _survival(ctl)
        before = _state(ctl)
        code, reply = ctl("command", f"damage @s {a['amount']} {a.get('type', 'minecraft:mob_attack')}")
        ctl("wait", 4)
        after = _state(ctl)
        h0, h1 = _health(before), _health(after)
        _restore(ctl, after)
        if "error" in reply:
            return code, reply
        if h0 is None or h1 is None:
            return 1, {"error": "state has no health", "kind": "unparsed"}
        return 0, {"ok": True, "value": h0 - h1, "health_before": h0, "health_after": h1}
    if cmd == "c_fall":
        _survival(ctl)
        before = _state(ctl)
        x, y, z = _pos(before)
        ctl("command", f"tp @s {x} {y + float(a['height'])} {z}")
        landed = False
        for _ in range(200):
            s = _state(ctl)
            if _pos(s)[1] < y + 0.5 and _on_ground(s):
                landed = True
                break
        ctl("wait", 4)
        after = _state(ctl)
        h0, h1 = _health(before), _health(after)
        _restore(ctl, after)
        if not landed or h0 is None or h1 is None:
            return 1, {"error": "the player never landed (or state has no health)", "kind": "timeout"}
        return 0, {"ok": True, "value": h0 - h1, "health_before": h0, "health_after": h1}
    if cmd == "c_jump":
        ctl("command", "gamemode survival")
        ctl("wait", 10)
        y0 = _pos(_state(ctl))[1]
        top, left = y0, False
        ctl("key", "space", "down")
        started = time.time()
        try:
            while time.time() - started < 6:
                s = _state(ctl)
                y = _pos(s)[1]
                top = max(top, y)
                left = left or y > y0 + 0.05
                if left and _on_ground(s):
                    break
        finally:
            ctl("key", "space", "up")
            ctl("release")
        ctl("wait", 10)
        ctl("command", "gamemode creative")
        if not left:
            return 1, {"error": "the player did not leave the ground", "kind": "state"}
        return 0, {"ok": True, "value": top - y0, "start_y": y0, "top_y": top}
    if cmd == "c_walk":
        ctl("command", "gamemode survival")
        sx, sy, sz = a.get("stand") or _pos(_state(ctl))
        ctl("command", f"tp @s {sx} {sy} {sz} 0 0")
        ctl("wait", 10)
        p0 = _pos(_state(ctl))
        words = ["move", "forward", a.get("seconds", 2)] + (["--sprint"] if a.get("sprint") else [])
        code, reply = ctl(*words, timeout=60)
        p1 = _pos(_state(ctl))
        ctl("command", "gamemode creative")
        if "error" in reply:
            return code, reply
        dist = math.hypot(p1[0] - p0[0], p1[2] - p0[2])
        return 0, {"ok": True, "value": dist / float(a.get("seconds", 2)), "distance": dist}
    if cmd in ("c_attack", "c_interact_entity"):
        ent = _entity(ctl, a["type"], a.get("radius", 8))
        if not ent or "id" not in ent:
            return 1, {"error": f"no {a['type']} within {a.get('radius', 8)} blocks", "kind": "unknown"}
        ctl("look-at-entity", ent["id"])
        ctl("wait", 20)  # the attack cooldown
        if cmd == "c_interact_entity":
            return ctl("interact-entity", ent["id"])
        code, reply = ctl("attack", ent["id"])
        before, after = _number_near(reply, "before"), _number_near(reply, "after")
        if before is not None and after is not None:
            reply = {**reply, "value": before - after}
        return code, reply
    if cmd == "c_entity_attr":
        code, reply = ctl("command", f"attribute @e[type={a['type']},limit=1,sort=nearest] {a['attribute']} get")
        value = number_in(reply, rf"is {NUMBER}")
        return code, {**reply, "value": value}
    if cmd == "c_station":
        x, y, z = a["pos"]
        ctl("close")
        ctl("command", "clear")
        totals: dict[str, int] = {}
        for _, item, n in a["inputs"]:
            totals[item] = totals.get(item, 0) + n
        for item, n in totals.items():
            code, reply = ctl("give", item, n)
            if "error" in reply:
                return code, {**reply, "error": f"give {item}: {reply['error']}"}
        ctl("setblock", x, y, z, a["block"])
        ctl("command", "tp @s {} {} {}".format(*a["stand"]))
        ctl("wait", 6)
        ctl("look-at-block", x, y, z)
        ctl("mouse", "right", "click")
        ctl("wait", 6)
        _, menu = ctl("menu")
        if not _menu_slots(menu):
            return 1, {"error": f"right-clicking the {a['block']} opened no menu ({json.dumps(menu)[:200]})", "kind": "state"}
        for slot, item, n in a["inputs"]:
            for _ in range(n):
                src = next((d for d in _menu_slots(menu) if d["slot"] < 36 and d.get("item") == item), None)
                if src is None:
                    return 1, {"error": f"{item} is not in the inventory to put in slot {slot}", "kind": "state"}
                ctl("click-slot", src["slot"])
                ctl("click-slot", slot, "right")
                ctl("click-slot", src["slot"])
                _, menu = ctl("menu")
        ctl("wait", a.get("ticks", 2))
        _, menu = ctl("menu")
        return 0, {**menu, "ok": True}
    raise ValueError(f"unknown compound step {cmd}")
