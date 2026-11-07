#!/usr/bin/env python3
"""Write a mod's mechanical content files from its contract, so an agent writes only what needs judgement.

The contract (planning/mods/<id>.features.json merged with planning/mods/<id>.contract/<name>.json, the
mod-spec skill) already says each block's shape, facing and face sprites and each item's sprite, stack
size and durability. From it, in mods/example_mod's shapes, this writes:

  assets/<id>/blockstates/<name>.json     one variant, or facing=... variants turning the model
  assets/<id>/models/block/<name>.json    the contract's shape, a texture per face it names
  assets/<id>/models/item/<name>.json     items: item/generated with the item's sprite
  assets/<id>/items/<name>.json           the item model (a block's item: its block model)
  data/<id>/loot_table/blocks/<name>.json the block drops itself (Horizon's pool `condition` form)
  assets/<id>/lang/en_us.json             block.<id>.<name> / item.<id>.<name> keys it lacks
  data/<id>/horizon/creative_tab/<id>.json  every block and item it lacks, in the outline's order
  data/<id>/horizon/item/<name>.json      only max_stack_size and minecraft:max_damage, when it exists
  data/<id>/horizon/block/<name>.json     only "shapes" for a thin block, when it exists and has none

Textures are named as the asset-smith slices them: <id>:block/<name>_<face> and <id>:item/<name>
("minecraft:..." sprites stay vanilla). An existing file is kept (--force rewrites the generated kinds);
lang and creative tab only gain what they lack. What needs a person's or agent's choice (a block's or
item's template and properties, recipes, GUIs) it lists under "yours".

Usage:
  make content-gen MOD=<id> [NAME=a,b] [ARGS=--check|--force]
  python3 scripts/content_gen.py --mod <id> [--name a,b] [--check] [--force]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from contract_check import DIRS, FACE_DIRS, HORIZONTAL, load_contract  # noqa: E402

ROTATIONS = {"horizontal": {"north": {}, "south": {"y": 180}, "west": {"y": 270}, "east": {"y": 90}},
             "all": {"north": {}, "south": {"y": 180}, "west": {"y": 270}, "east": {"y": 90},
                     "up": {"x": 270}, "down": {"x": 90}}}


class Out:
    def __init__(self, mod: str, check: bool, force: bool):
        self.base = ROOT / "mods" / mod / "content"
        self.check, self.force = check, force
        self.wrote: list[str] = []
        self.kept: list[str] = []
        self.yours: list[str] = []

    def rel(self, path: Path) -> str:
        return path.relative_to(ROOT).as_posix() if path.is_relative_to(ROOT) else path.as_posix()

    def write(self, rel: str, data: dict, generated_kind: bool = True) -> None:
        path = self.base / rel
        if path.is_file() and not (self.force and generated_kind):
            self.kept.append(self.rel(path))
            return
        self.wrote.append(self.rel(path))
        if not self.check:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8", newline="\n")

    def merge(self, rel: str, update) -> None:
        """Read a JSON file (or {}), let update() change it; write it if it changed."""
        path = self.base / rel
        try:
            data = json.loads(path.read_text(encoding="utf-8")) if path.is_file() else {}
        except ValueError:
            self.yours.append(f"{self.rel(path)}: not JSON, fix it by hand")
            return
        before = json.dumps(data, sort_keys=True)
        update(data)
        if json.dumps(data, sort_keys=True) != before:
            self.wrote.append(self.rel(path))
            if not self.check:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


def texture(mod: str, name: str, face: str, sprite: object) -> str:
    if isinstance(sprite, str) and sprite.startswith("minecraft:"):
        return sprite
    return f"{mod}:block/{name}_{face}"


def face_map(mod: str, name: str, faces: dict) -> tuple[dict[str, str], list[str]]:
    """Model direction -> texture, for the block as placed facing north; and what could not be placed."""
    dirs: dict[str, str] = {}
    for face, targets in FACE_DIRS.items():
        if face in faces:
            for d in targets:
                dirs[d] = texture(mod, name, face, faces[face])
    if "side" in faces:
        for d in HORIZONTAL:
            dirs.setdefault(d, texture(mod, name, "side", faces["side"]))
    if "all" in faces:
        for d in DIRS:
            dirs.setdefault(d, texture(mod, name, "all", faces["all"]))
    missing = [d for d in DIRS if d not in dirs]
    if missing and faces:
        first = next(iter(faces))
        for d in missing:
            dirs[d] = texture(mod, name, first, faces[first])
    return dirs, missing


def block_model(mod: str, entry: dict) -> tuple[dict | None, str | None]:
    name, shape, faces = entry["id"], entry.get("shape"), entry.get("faces") or {}
    if not faces:
        return None, "no faces in its contract"
    if shape == "cross":
        first = next(iter(faces))
        return {"parent": "minecraft:block/cross", "textures": {"cross": texture(mod, name, first, faces[first])}}, None
    dirs, missing = face_map(mod, name, faces)
    textures = {**{d: dirs[d] for d in DIRS}, "particle": dirs.get("up") or dirs["north"]}
    model: dict = {"parent": "minecraft:block/cube", "textures": textures}
    if isinstance(shape, list) and len(shape) == 6 and list(shape) != [0, 0, 0, 16, 16, 16]:
        model["elements"] = [{"from": shape[:3], "to": shape[3:],
                              "faces": {d: {"texture": f"#{d}"} for d in DIRS}}]
    note = f"faces {', '.join(missing)} not in its contract: drawn with its first face's sprite" if missing else None
    return model, note


def blockstate(mod: str, entry: dict) -> dict:
    model = f"{mod}:block/{entry['id']}"
    turns = ROTATIONS.get(entry.get("facing") or "none")
    if not turns:
        return {"variants": {"": {"model": model}}}
    return {"variants": {f"facing={d}": {"model": model, **rot} for d, rot in turns.items()}}


def loot(mod: str, name: str) -> dict:
    return {"type": "minecraft:block",
            "pools": [{"condition": {"type": "minecraft:survives_explosion"},
                       "entries": [{"type": "minecraft:item", "name": f"{mod}:{name}"}], "rolls": 1}],
            "random_sequence": f"{mod}:blocks/{name}"}


def generate(mod: str, names: set[str] | None, check: bool, force: bool) -> Out:
    contract = load_contract(mod)
    out = Out(mod, check, force)
    if contract is None:
        out.yours.append(f"no outline at planning/mods/{mod}.features.json (the spec-writer's, step 1)")
        return out
    blocks = [e for e in contract.get("blocks", []) if isinstance(e, dict) and e.get("id")]
    items = [e for e in contract.get("items", []) if isinstance(e, dict) and e.get("id")]
    block_ids = {e["id"] for e in blocks}
    item_by_id = {e["id"]: e for e in items}
    want = (lambda i: i in names) if names else (lambda i: True)

    for e in blocks:
        name = e["id"]
        if not want(name):
            continue
        model, note = block_model(mod, e)
        if model is None:
            out.yours.append(f"{name}: {note}; write its contract's faces first")
        else:
            out.write(f"assets/{mod}/models/block/{name}.json", model)
            if note:
                out.yours.append(f"{name}: {note}")
        out.write(f"assets/{mod}/blockstates/{name}.json", blockstate(mod, e))
        own = item_by_id.get(name)
        if not (own and own.get("sprite")):  # a block item drawn as its block
            out.write(f"assets/{mod}/items/{name}.json",
                      {"model": {"type": "minecraft:model", "model": f"{mod}:block/{name}"}})
        out.write(f"data/{mod}/loot_table/blocks/{name}.json", loot(mod, name))
        data = out.base / f"data/{mod}/horizon/block/{name}.json"
        if not data.is_file():
            out.yours.append(f"{name}: data/{mod}/horizon/block/{name}.json (its template and properties, "
                             "from the Java: hardness, resistance, light, tool, sounds)")
        elif isinstance(e.get("shape"), list) and e["shape"] != [0, 0, 0, 16, 16, 16]:
            box = [round(v / 16, 4) for v in e["shape"]]
            out.merge(f"data/{mod}/horizon/block/{name}.json",
                      lambda d, box=box: d.setdefault("shapes", {"default": [box]}))
        if e.get("gui"):
            out.yours.append(f"{name}: its GUI ({len(e['gui'].get('slots', []))} slots) is the domain-builder's "
                             "native_screen, not a template")

    for e in items:
        name = e["id"]
        if not want(name):
            continue
        sprite = e.get("sprite")
        if isinstance(sprite, list):
            out.yours.append(f"{name}: {len(sprite)} subtypes, one sprite each: write its item model by hand")
        elif sprite:
            tex = sprite if isinstance(sprite, str) and sprite.startswith("minecraft:") else f"{mod}:item/{name}"
            out.write(f"assets/{mod}/models/item/{name}.json",
                      {"parent": "minecraft:item/generated", "textures": {"layer0": tex}})
            out.write(f"assets/{mod}/items/{name}.json",
                      {"model": {"type": "minecraft:model", "model": f"{mod}:item/{name}"}})
        elif name not in block_ids:
            out.yours.append(f"{name}: no sprite in its contract")
        if name in block_ids:
            continue  # a block's item: its data is the block's ("item": true)
        data = out.base / f"data/{mod}/horizon/item/{name}.json"
        if not data.is_file():
            out.yours.append(f"{name}: data/{mod}/horizon/item/{name}.json (its template, from what it does)")
            continue

        def numbers(d: dict, e=e) -> None:
            if isinstance(e.get("max_stack"), int):
                d["max_stack_size"] = e["max_stack"]
            if isinstance(e.get("durability"), int):
                d.setdefault("components", {})["minecraft:max_damage"] = e["durability"]
        out.merge(f"data/{mod}/horizon/item/{name}.json", numbers)

    chosen = [e for e in blocks + items if want(e["id"])]

    def lang(d: dict) -> None:
        for e in chosen:
            kind = "block" if e["id"] in block_ids else "item"
            d.setdefault(f"{kind}.{mod}.{e['id']}", e.get("name") or e["id"].replace("_", " ").title())
        d.setdefault(f"itemGroup.{mod}.{mod}", mod.replace("_", " ").title())
    out.merge(f"assets/{mod}/lang/en_us.json", lang)

    tabs = sorted((out.base / f"data/{mod}/horizon/creative_tab").glob("*.json"))
    listed: set[str] = set()
    for t in tabs:
        try:
            listed |= set(json.loads(t.read_text(encoding="utf-8")).get("items", []))
        except ValueError:
            continue
    missing = [f"{mod}:{e['id']}" for e in chosen if f"{mod}:{e['id']}" not in listed]
    if missing:
        tab = tabs[0].relative_to(out.base).as_posix() if tabs else f"data/{mod}/horizon/creative_tab/{mod}.json"

        def add(d: dict) -> None:
            d.setdefault("title", f"itemGroup.{mod}.{Path(tab).stem}")
            d.setdefault("icon", missing[0])
            d["items"] = list(d.get("items", [])) + [m for m in missing if m not in d.get("items", [])]
        out.merge(tab, add)
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mod", required=True)
    parser.add_argument("--name", default="", help="Only these blocks and items (comma-separated ids)")
    parser.add_argument("--check", action="store_true", help="Say what it would write; write nothing")
    parser.add_argument("--force", action="store_true", help="Rewrite generated files that exist")
    args = parser.parse_args()
    if not (ROOT / "mods" / args.mod).is_dir():
        print(f"No mods/{args.mod}/ (step 2 Scaffold makes it).", file=sys.stderr)
        return 1
    names = {n.strip() for n in args.name.split(",") if n.strip()} or None
    out = generate(args.mod, names, args.check, args.force)
    verb = "would write" if args.check else "wrote"
    for p in out.wrote[:60]:
        print(f"{verb}: {p}")
    if len(out.wrote) > 60:
        print(f"... and {len(out.wrote) - 60} more")
    print(f"{len(out.wrote)} file(s) {verb}, {len(out.kept)} kept as they are.")
    if out.yours:
        print("Yours to write (needs a choice the contract does not make):")
        for y in out.yours:
            print(f"  - {y}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
