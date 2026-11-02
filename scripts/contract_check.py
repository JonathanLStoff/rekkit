#!/usr/bin/env python3
"""Check a mod against its contract: what the original is, block by block and item by item.

The spec-writer's outline (planning/mods/<id>.features.json) lists every block and item with the
attributes a player must find (a solar panel generates in daylight and stores energy). The agent that
ports a block or item writes its contract file, planning/mods/<id>.contract/<name>.json, from the
original's Java: its shape, whether it turns to face the player, the original sprite on each face and
its GUI's slots, or an item's sprite, stack size and durability, and a scenario testing each outlined
attribute (the mod-spec skill has the format). This script compares what the port draws with the
original's own pixels, and holds every untested attribute as a blocker.

Static checks (make contract-check, and make check-mods COMPLETE=1):
  - every block and item of the mod has a complete contract entry, and every entry is in the mod;
  - every attribute the outline gives a block or item is covered by a scenario ("covers");
  - each face's texture, after the blockstate's rotation and the model's parents, is the original's
    sprite pixel for pixel (a vanilla texture, the top on every side, or a swapped sprite fails, and
    the message names the sprite the texture really is);
  - the model's shape is the original's (a full cube for a 2-pixel gate fails), and a block that turns
    to face the player has a blockstate that turns its model;
  - each item's icon is its sprite, its stack size and durability are the original's.
The GUI slots and scenarios are checked live by scripts/e2e_harness.py.

Usage:
  make contract-check MOD=<id>            the problems, one a line
  python3 scripts/contract_check.py --mod <id> [--json]
"""

from __future__ import annotations

import argparse
import io
import json
import re
import sys
import zipfile
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from art import read_png  # noqa: E402

DIRS = ("north", "east", "south", "west", "up", "down")
HORIZONTAL = ("north", "east", "south", "west")
# Contract face names -> model directions, for the block as placed facing north (vanilla's front is north).
FACE_DIRS = {"top": ("up",), "bottom": ("down",), "front": ("north",), "back": ("south",),
             "left": ("west",), "right": ("east",), "end": ("up", "down")}
# Vanilla parents a mod model may use: their faces' texture variables, and their shape.
CUBE = (0, 0, 0, 16, 16, 16)
VANILLA_PARENTS = {
    "block/cube_all": ({d: "#all" for d in DIRS}, CUBE),
    "block/leaves": ({d: "#all" for d in DIRS}, CUBE),
    "block/cube_mirrored_all": ({d: "#all" for d in DIRS}, CUBE),
    "block/cube": ({d: f"#{d}" for d in DIRS}, CUBE),
    "block/cube_column": ({**{d: "#side" for d in HORIZONTAL}, "up": "#end", "down": "#end"}, CUBE),
    "block/cube_column_horizontal": ({**{d: "#side" for d in HORIZONTAL}, "up": "#end", "down": "#end"}, CUBE),
    "block/cube_bottom_top": ({**{d: "#side" for d in HORIZONTAL}, "up": "#top", "down": "#bottom"}, CUBE),
    "block/cube_top": ({**{d: "#side" for d in HORIZONTAL}, "up": "#top", "down": "#side"}, CUBE),
    "block/orientable": ({"north": "#front", "east": "#side", "south": "#side", "west": "#side",
                          "up": "#top", "down": "#top"}, CUBE),
    "block/orientable_with_bottom": ({"north": "#front", "east": "#side", "south": "#side", "west": "#side",
                                      "up": "#top", "down": "#bottom"}, CUBE),
    "block/cross": ({}, "cross"),
    "block/tinted_cross": ({}, "cross"),
    "block/crop": ({}, "cross"),
}


class Sheets:
    """The original's sprite sheets, read from its archives (asset-map.json's source_archives)."""

    def __init__(self, mod_dir: Path, amap: dict):
        self.archives = [ROOT / a for a in amap.get("source_archives", [])]
        self.size = int(amap.get("resolution", 16))

    @lru_cache(maxsize=None)
    def sheet(self, sheet: str, archive: str | None) -> tuple[int, int, list] | None:
        for path in self.archives:
            if archive and path.stem != archive or not path.is_file():
                continue
            with zipfile.ZipFile(path) as z:
                names = [n for n in z.namelist() if n == sheet.lstrip("/") or n.endswith("/" + sheet.lstrip("/"))]
                if names:
                    return png_bytes(z.read(names[0]))
        return None

    def sprite(self, ref: dict) -> list | None:
        got = self.sheet(ref["sheet"], ref.get("archive"))
        if got is None:
            return None
        width, _, px = got
        size = width // 16  # legacy sheets are 16 sprites a row
        col, row = ref["index"] % 16, ref["index"] // 16
        return [px[(row * size + y) * width + col * size + x] for y in range(size) for x in range(size)]

    def which(self, ref: dict, pixels: list) -> int | None:
        """The index of the sprite on ref's sheet that these pixels are, if any."""
        for index in range(256):
            if same(self.sprite({**ref, "index": index}) or [], pixels):
                return index
        return None


def png_bytes(data: bytes) -> tuple[int, int, list]:
    class Blob:
        def read_bytes(self):
            return data

        def __str__(self):
            return "<archive png>"
    return read_png(Blob())  # type: ignore[arg-type]


def same(a: list, b: list) -> bool:
    """Equal sprites, at any resolution (nearest-neighbour), with fully transparent pixels equal."""
    if not a or not b:
        return False
    sa, sb = int(len(a) ** 0.5), int(len(b) ** 0.5)
    if sa * sa != len(a) or sb * sb != len(b):
        return False
    n = min(sa, sb)
    for y in range(n):
        for x in range(n):
            p = a[(y * sa // n) * sa + x * sa // n]
            q = b[(y * sb // n) * sb + x * sb // n]
            if p[3] == 0 and q[3] == 0:
                continue
            if p != q:
                return False
    return True


def parse_ref(ref: object) -> dict | str | None:
    """A sprite reference: "sheet.png#index" or {"sheet", "index", "archive"?}; "minecraft:..." is a
    vanilla texture the original reused (the port must use the same one)."""
    if isinstance(ref, dict) and "sheet" in ref and "index" in ref:
        return {"sheet": ref["sheet"], "index": int(ref["index"]), "archive": ref.get("archive")}
    if isinstance(ref, str):
        if ref.startswith("minecraft:"):
            return ref
        m = re.fullmatch(r"(?:(\w+):)?(.+\.png)#(\d+)", ref)
        if m:
            return {"sheet": m.group(2), "index": int(m.group(3)), "archive": m.group(1)}
    return None


def rotate_box(box: tuple, x_rot: int, y_rot: int) -> tuple:
    x1, y1, z1, x2, y2, z2 = box
    for _ in range((x_rot // 90) % 4):  # about the x axis
        y1, z1, y2, z2 = 16 - z2, y1, 16 - z1, y2
    for _ in range((y_rot // 90) % 4):  # about the y axis, clockwise seen from above
        x1, z1, x2, z2 = 16 - z2, x1, 16 - z1, x2
    return (min(x1, x2), min(y1, y2), min(z1, z2), max(x1, x2), max(y1, y2), max(z1, z2))


def same_shape(model: tuple, contract: tuple) -> bool:
    """The same box in some orientation (a plate on the floor or on a wall is one shape)."""
    return any(rotate_box(contract, x, y) == model for x in (0, 90, 180, 270) for y in (0, 90, 180, 270))


class Mod:
    def __init__(self, mod_id: str):
        self.id = mod_id
        self.dir = ROOT / "mods" / mod_id
        self.assets = self.dir / "content" / "assets" / mod_id
        self.data = self.dir / "content" / "data" / mod_id / "horizon"
        amap_path = self.dir / "asset-map.json"
        self.amap = json.loads(amap_path.read_text(encoding="utf-8")) if amap_path.is_file() else {}
        self.sheets = Sheets(self.dir, self.amap)

    def json(self, path: Path) -> dict:
        try:
            return json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return {}

    def model(self, ref: str, depth: int = 0) -> tuple[dict, dict, object] | None:
        """(face direction -> texture ref, texture variables, shape) of a model with its parents;
        shape is a box in pixels, "cross", or None when unknown. None for a vanilla model."""
        ns, _, rel = ref.partition(":") if ":" in ref else ("minecraft", "", ref)
        if ns == "minecraft":
            if rel in VANILLA_PARENTS:
                faces, shape = VANILLA_PARENTS[rel]
                return dict(faces), {}, shape
            return None
        data = self.json(self.assets / "models" / f"{rel}.json")
        if not data or depth > 8:
            return None
        faces, textures, shape = {}, {}, None
        if isinstance(data.get("parent"), str):
            parent = self.model(data["parent"], depth + 1)
            if parent is None:
                return None
            faces, textures, shape = parent
        textures = {**textures, **data.get("textures", {})}
        if data.get("elements"):
            faces, boxes = {}, []
            for el in data["elements"]:
                f, t = el.get("from", [0, 0, 0]), el.get("to", [16, 16, 16])
                boxes.append((*f, *t))
                for d, face in el.get("faces", {}).items():
                    faces.setdefault(d, [])
                    faces[d] = faces[d] + [face.get("texture", "")]
            shape = tuple(round(v) for v in (min(b[0] for b in boxes), min(b[1] for b in boxes), min(b[2] for b in boxes),
                                             max(b[3] for b in boxes), max(b[4] for b in boxes), max(b[5] for b in boxes)))
        return faces, textures, shape

    def resolve(self, value: str, textures: dict) -> str:
        for _ in range(10):
            if not value.startswith("#"):
                return value
            value = textures.get(value[1:], value)
        return value

    def texture_pixels(self, ref: str) -> list | None:
        ns, _, rel = ref.partition(":") if ":" in ref else ("minecraft", "", ref)
        if ns != self.id:
            return None
        path = self.assets / "textures" / f"{rel}.png"
        if not path.is_file():
            return None
        w, h, px = read_png(path)
        return px[: w * w]  # an animated strip's first frame


def default_variant(blockstate: dict) -> dict | None:
    """The variant a block placed facing north (or with no state) shows."""
    variants = blockstate.get("variants")
    if isinstance(variants, dict) and variants:
        def pick(v):
            return v[0] if isinstance(v, list) else v
        for key, v in variants.items():
            if "facing=north" in key.split(","):
                return pick(v)
        return pick(next(iter(variants.values())))
    parts = blockstate.get("multipart")
    if isinstance(parts, list) and parts:
        return parts[0].get("apply") if isinstance(parts[0].get("apply"), dict) else parts[0].get("apply", [None])[0]
    return None


def turns(blockstate: dict) -> bool:
    """Whether the blockstate turns the model with the block's facing."""
    variants = blockstate.get("variants") or {}
    rotations = {(v[0] if isinstance(v, list) else v).get("y", 0) for k, v in variants.items() if "facing=" in k}
    if len(rotations) > 1:
        return True
    return any("facing" in json.dumps(p.get("when", {})) for p in blockstate.get("multipart", []))


def load_contract(mod_id: str, path: Path | None = None) -> dict | None:
    """The spec-writer's outline (planning/mods/<id>.features.json: every block and item, with the
    attributes a player must find) merged with each block's and item's own contract file,
    planning/mods/<id>.contract/<name>.json, which the agent porting it writes from the Java (its shape,
    facing, faces, GUI, sprite, stack size, and scenarios). One file each, so parallel agents never
    edit the same file. None when there is no outline."""
    path = path or ROOT / "planning" / "mods" / f"{mod_id}.features.json"
    if not path.is_file():
        return None
    contract = json.loads(path.read_text(encoding="utf-8"))
    contract.setdefault("scenarios", [])
    entries = {e["id"]: e for kind in ("blocks", "items") for e in contract.get(kind, []) if isinstance(e, dict)}
    for f in sorted((path.parent / f"{mod_id}.contract").glob("*.json")):
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
        except ValueError as exc:
            contract.setdefault("broken", []).append(f"{f.name}: {exc}")
            continue
        for scenario in data.pop("scenarios", []):
            scenario.setdefault("for", [f.stem])
            contract["scenarios"].append(scenario)
        if f.stem in entries:
            entries[f.stem].update({k: v for k, v in data.items() if k not in ("id", "name", "java")})
    return contract


# The harness's test levels (planning/harness.md). Level 1, placement and hitbox, the harness runs for
# every block itself; 7 is the all-mods phase.
LEVELS = {1: "placement & hitbox", 2: "item use", 3: "block function (energy, fuel, storage, GUI slots)",
          4: "interaction with other blocks", 5: "persistence (chunk reloaded from disk)",
          6: "automation & sides (hoppers, tubes, sided access)"}


def attribute(raw: object) -> tuple[str, int | None]:
    """An outline attribute: {"text": ..., "level": n} or "L<n>: text"."""
    if isinstance(raw, dict):
        return str(raw.get("text", "")), (int(raw["level"]) if str(raw.get("level", "")).isdigit() else None)
    m = re.match(r"\s*L(\d)\s*:\s*(.+)", str(raw))
    return (m.group(2).strip(), int(m.group(1))) if m else (str(raw), None)


def check(mod_id: str, path: Path | None = None) -> list[dict]:
    mod = Mod(mod_id)
    contract = load_contract(mod_id, path)
    if contract is None:
        return [{"for": mod_id, "kind": "contract", "owner": "spec-writer",
                 "why": f"planning/mods/{mod_id}.features.json (the outline) is missing"}]
    problems: list[dict] = []

    def problem(owner_id: str, kind: str, owner: str, why: str) -> None:
        problems.append({"for": f"{mod_id}:{owner_id}" if ":" not in owner_id else owner_id,
                         "kind": kind, "owner": owner, "why": why})

    blocks = {b["id"]: b for b in contract.get("blocks", [])}
    items = {i["id"]: i for i in contract.get("items", [])}
    for broken in contract.get("broken", []):
        problem(mod_id, "contract", "content-author", f"planning/mods/{mod_id}.contract/{broken}")
    # Every attribute the outline gives a block or item is a blocker until a scenario of its level tests it.
    scenarios = contract["scenarios"]
    for sc in scenarios:
        label = f"scenario {sc.get('name', '?')!r}"
        if int(sc.get("level") or 0) not in LEVELS:
            problem(",".join(sc.get("for", [])) or mod_id, "scenario", "content-author",
                    f"{label} has no test level (\"level\": 1-6: {', '.join(f'{k} {v}' for k, v in LEVELS.items())})")
        if not any(k.startswith("expect") or "expect" in st for st in sc.get("steps", []) for k in st):
            problem(",".join(sc.get("for", [])) or mod_id, "scenario", "content-author", f"{label} checks nothing (no expect step)")

    port_blocks = {p.stem for p in (mod.data / "block").glob("*.json")}
    port_items = {p.stem for p in (mod.data / "item").glob("*.json")}
    for name in sorted(port_blocks - set(blocks)):
        problem(name, "contract", "spec-writer", "a block of the port with no contract entry (or no original)")
    for name in sorted(port_items - set(items)):
        problem(name, "contract", "spec-writer", "an item of the port with no contract entry (or no original)")
    for name in sorted(set(blocks) - port_blocks):
        problem(name, "missing", "content-author", "in the contract, not in the port")
    for name in sorted(set(items) - port_items):
        problem(name, "missing", "content-author", "in the contract, not in the port")

    def levels_for(name: str) -> set[int]:
        return {int(sc.get("level") or 0) for sc in scenarios if name in sc.get("for", [])}

    for name, entry in sorted({**items, **blocks}.items()):
        if "attributes" not in entry:
            problem(name, "outline", "spec-writer", 'no "attributes" in the outline: what a player must find it doing, '
                    'each with its test level ([] for a plain block)')
        for raw in entry.get("attributes", []):
            text, level = attribute(raw)
            if level not in LEVELS:
                problem(name, "outline", "spec-writer", f"attribute {text!r} has no test level (1-6)")
                continue
            if not any(name in sc.get("for", []) and text in sc.get("covers", []) and int(sc.get("level") or 0) == level
                       for sc in scenarios):
                problem(name, "attribute", "content-author",
                        f"level {level} ({LEVELS[level]}): attribute {text!r} has no scenario testing it (a level-{level} "
                        f'scenario with "covers": [{text!r}] in planning/mods/{mod_id}.contract/{name}.json)')
        have = levels_for(name)
        needs = []
        if name in items and (entry.get("use") or entry.get("durability")):
            needs.append(2)
        if name in blocks and (isinstance(entry.get("gui"), dict) or entry.get("energy")):
            needs += [3, 5]
        for level in needs:
            if level not in have:
                problem(name, "level", "content-author", f"needs a level-{level} scenario ({LEVELS[level]})")

    def check_face(name: str, label: str, tex: str | None, want: object) -> None:
        ref = parse_ref(want)
        if ref is None:
            problem(name, "contract", "content-author", f"{label}: {want!r} is not a sprite reference (sheet.png#index)")
            return
        if tex is None:
            problem(name, "model", "content-author", f"{label}: the model draws nothing there")
            return
        if isinstance(ref, str):
            if tex != ref:
                problem(name, "texture", "content-author", f"{label} is {tex}; the original uses vanilla's {ref}")
            return
        original = mod.sheets.sprite(ref)
        where = f"sprite {ref['index']} of {ref['sheet']}"
        if original is None:
            problem(name, "contract", "content-author", f"{label}: no {ref['sheet']} in the archives of asset-map.json")
            return
        if tex.startswith("minecraft:"):
            problem(name, "texture", "content-author", f"{label} is vanilla's {tex}; the original is {where}")
            return
        pixels = mod.texture_pixels(tex)
        if pixels is None:
            problem(name, "texture", "asset-smith", f"{label}: {tex} has no texture file; the original is {where}")
        elif not same(pixels, original):
            got = mod.sheets.which(ref, pixels)
            really = f"; that file is sprite {got} of the sheet (swapped?)" if got is not None else ""
            problem(name, "texture", "asset-smith", f"{label}: {tex} is not {where}{really}")

    def check_any(name: str, label: str, texs: list, refs: list) -> None:
        """Fine when one of the textures drawn there is one of the sprites; else the first problem."""
        first = None
        for tex in texs:
            for ref in refs:
                mark = len(problems)
                check_face(name, label, tex, ref)
                if len(problems) == mark:
                    return
                first = first or problems[mark]
                del problems[mark:]
        if first:
            problems.append(first)

    required_block = ("shape", "facing", "gui")
    for name, entry in sorted(blocks.items()):
        if name not in port_blocks:
            continue
        lacking = [k for k in required_block if k not in entry] + ([] if "faces" in entry else ["faces"])
        if lacking:
            problem(name, "contract", "content-author",
                    f"planning/mods/{mod_id}.contract/{name}.json lacks " + ", ".join(lacking) + " (read them from the Java)")
        blockstate = mod.json(mod.assets / "blockstates" / f"{name}.json")
        variant = default_variant(blockstate)
        if not variant or "model" not in variant:
            problem(name, "model", "content-author", "no blockstate model")
            continue
        resolved = mod.model(variant["model"])
        if resolved is None:
            problem(name, "model", "content-author",
                    f"{variant['model']}: a vanilla or unknown model; give the block its own model (the original's shape and sprites)")
            continue
        faces, textures, shape = resolved
        y_rot = int(variant.get("y", 0))

        def world_textures(direction: str) -> list[str]:
            model_dir = direction
            if direction in HORIZONTAL:
                model_dir = HORIZONTAL[(HORIZONTAL.index(direction) - y_rot // 90) % 4]
            got = faces.get(model_dir)
            refs = got if isinstance(got, list) else ([got] if got else [])
            return [mod.resolve(r, textures) for r in refs]

        want_faces = entry.get("faces")
        if isinstance(want_faces, dict):
            given: dict[str, object] = {}
            for key, ref in want_faces.items():
                for d in FACE_DIRS.get(key, ()):
                    given[d] = ref
            for key in ("side", "all"):
                if key in want_faces:
                    for d in (HORIZONTAL if key == "side" else DIRS):
                        given.setdefault(d, want_faces[key])
            for d, ref in given.items():
                check_any(name, f"face {d}", world_textures(d) or [None], [ref])
        want_shape = entry.get("shape")
        if want_shape is not None and shape is not None:
            if want_shape == "full":
                want_shape = CUBE
            if isinstance(want_shape, list):
                want_shape = tuple(want_shape)
            if want_shape != shape if isinstance(want_shape, str) or isinstance(shape, str) \
                    else not same_shape(shape, want_shape):
                drawn = "a full cube" if shape == CUBE else shape
                problem(name, "shape", "content-author", f"the original's shape is {list(want_shape) if isinstance(want_shape, tuple) else want_shape}; the model is {drawn}")
        if entry.get("facing") in ("horizontal", "all") and not turns(blockstate):
            problem(name, "facing", "content-author",
                    "the original turns to face the player; the blockstate does not turn the model (facing=... variants with y/x)")

    for name, entry in sorted(items.items()):
        if name not in port_items:
            continue
        lacking = [k for k in ("sprite", "max_stack") if k not in entry]
        if lacking:
            problem(name, "contract", "content-author",
                    f"planning/mods/{mod_id}.contract/{name}.json lacks " + ", ".join(lacking) + " (read them from the Java)")
        item = mod.json(mod.data / "item" / f"{name}.json")
        if "max_stack" in entry and item.get("max_stack_size", None) not in (entry["max_stack"],) \
                and not (entry["max_stack"] == 64 and "max_stack_size" not in item):
            problem(name, "item", "content-author",
                    f"stacks to {entry['max_stack']} in the original; max_stack_size is {item.get('max_stack_size', 'the template default')}")
        want_damage = entry.get("durability")
        got_damage = (item.get("components") or {}).get("minecraft:max_damage")
        if want_damage and got_damage != want_damage:
            problem(name, "item", "content-author", f"durability {want_damage} in the original; minecraft:max_damage is {got_damage}")
        if "sprite" in entry:
            definition = mod.json(mod.assets / "items" / f"{name}.json")
            model_ref = json.dumps(definition)
            refs = re.findall(r'"model":\s*"([^"]+)"', model_ref)
            texture = None
            for ref in refs:
                ns, _, rel = ref.partition(":")
                model = mod.json(mod.assets / "models" / f"{rel}.json") if ns == mod_id else {}
                layer = (model.get("textures") or {}).get("layer0")
                if layer:
                    texture = layer
                    break
                if model.get("parent", "").startswith(f"{mod_id}:block/"):
                    texture = "block model"
                    break
            if texture == "block model":
                continue  # a block item drawn as its block: the block's faces are checked above
            sprites = entry["sprite"] if isinstance(entry["sprite"], list) else [entry["sprite"]]
            check_any(name, "icon", [texture], sprites)
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mod", required=True)
    parser.add_argument("--json", action="store_true", help="The problems as JSON (make fleet-contract files them)")
    parser.add_argument("--contract", type=Path, help="Another contract file (default planning/mods/<id>.features.json)")
    args = parser.parse_args()
    found = check(args.mod, args.contract)
    if args.json:
        print(json.dumps(found, indent=1))
    else:
        for p in found:
            print(f"{p['for']}: [{p['kind']} -> {p['owner']}] {p['why']}")
        print(f"{len(found)} contract problem(s) in {args.mod}." if found else f"{args.mod} matches its contract.")
    return 1 if found else 0


if __name__ == "__main__":
    raise SystemExit(main())
