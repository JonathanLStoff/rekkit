#!/usr/bin/env python3
"""
Validate and package Rekkit mods into Horizon's Dual-Artifact formats:
  1. Content Mod (.hmod): data/ and assets/ with horizon-mod.toml (API 0.2, registries).
  2. Native Mod (.hnmod): horizon-native.json + mixins/<id>/ source bundle (NATIVE_HOOKS_API).

Implements the exact validation rules of:
  - horizon/crates/horizon-mods/src/manifest.rs
  - horizon/crates/horizon-mods/src/content.rs
  - horizon/crates/horizon-mods/src/index/native.rs
  - horizon/crates/horizon-mods/src/index/registry.rs
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tomllib
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import contract_check  # noqa: E402

ID_RE = re.compile(r"^[a-z0-9_]{1,64}$")
NS_RE = re.compile(r"^[a-z0-9_.-]{1,64}$")
PATH_PART_RE = re.compile(r"^[a-z0-9_.-]+$")
VANILLA_ID_RE = re.compile(r"^minecraft:[a-z0-9_./-]+$")
HEX40_RE = re.compile(r"^[0-9a-f]{40}$")
RESERVED_NAMESPACES = {"minecraft", "c", "horizon"}

MANIFEST_KEYS = {
    "id",
    "name",
    "version",
    "description",
    "authors",
    "api",
    "side",
    "capabilities",
    "namespaces",
    "experimental",
}
NATIVE_JSON_KEYS = {"schema", "id", "version", "game", "horizon", "mixins", "hooks"}
REGISTRY_TOML_KEYS = {"name", "summary", "description", "authors", "license"}
BLOCK_FILE_KEYS = {"template", "properties", "shapes", "collision_shapes", "default_state", "state_properties", "connects_to", "thickness", "wood_type", "wall_block", "item", "components"}
BLOCK_PROP_KEYS = {
    "hardness",
    "explosion_resistance",
    "light",
    "requires_tool",
    "friction",
    "map_color",
    "sounds",
}
ITEM_FILE_KEYS = {"template", "max_stack_size", "rarity", "crafting_remainder", "components"}
MOB_EFFECT_KEYS = {"category", "color"}
CREATIVE_TAB_KEYS = {"title", "icon", "items"}
# Content mods' own creative tabs (data/<id>/horizon/creative_tab/) arrived in this Horizon commit.
CREATIVE_TABS_SINCE = "74ff545e60fbda2377255effa9c52fc0c346ec5d"
HORIZON_GIT = Path(__file__).resolve().parents[1] / "horizon"
EFFECT_CATEGORIES = {"beneficial", "harmful", "neutral"}
RARITIES = {"common", "uncommon", "rare", "epic"}
ITEM_ID_RE = re.compile(r"^[a-z0-9_.-]+:[a-z0-9_./-]+$")


# Keys horizon-mixins accepts in horizon-mixin.toml (horizon-mixins-core/src/manifest.rs, rev 3f99a5c).
MIXIN_TOP_KEYS = {"mod", "compatibility", "injection", "bridge", "module", "state"}
MIXIN_MOD_KEYS = {"id", "version", "priority"}


class ValidationError(Exception):
    pass


def check_content_path(rel_path: str) -> None:
    parts = rel_path.split("/")
    if len(parts) < 3 or parts[0] not in ("assets", "data"):
        raise ValidationError(f"{rel_path!r} is not a plain assets/ or data/ path")
    for part in parts[1:]:
        if not part or part in (".", "..") or not PATH_PART_RE.match(part):
            raise ValidationError(f"{rel_path!r} contains invalid resource path segment {part!r}")


def validate_components(rel_path: str, components: object, allow_stack_and_rarity: bool) -> None:
    if not isinstance(components, dict):
        raise ValidationError(f"{rel_path}: components must be a JSON object")
    for k, v in components.items():
        removed = k.startswith("!")
        kind = k[1:] if removed else k
        if not VANILLA_ID_RE.match(kind):
            raise ValidationError(f"{rel_path}: component {k!r} is not a vanilla type (minecraft:...)")
        short = kind.removeprefix("minecraft:")
        if not allow_stack_and_rarity and short in ("max_stack_size", "rarity"):
            raise ValidationError(f"{rel_path}: component {k!r}: set {short} on the item directly")
        if removed:
            if v != {}:
                raise ValidationError(f"{rel_path}: removal component {k!r} must have value {{}}")
            continue
        if short == "max_stack_size":
            if not isinstance(v, int) or isinstance(v, bool) or not (1 <= v <= 99):
                raise ValidationError(f"{rel_path}: component {k!r} must be 1..99")
        elif short == "rarity":
            if not isinstance(v, str) or v not in RARITIES:
                raise ValidationError(f"{rel_path}: component {k!r} must be one of {sorted(RARITIES)}")


def validate_block_json(rel_path: str, data: dict) -> bool:
    unknown = set(data.keys()) - BLOCK_FILE_KEYS
    if unknown:
        raise ValidationError(f"{rel_path}: unknown keys {sorted(unknown)}")
    template = data.get("template", "")
    if not isinstance(template, str) or not VANILLA_ID_RE.match(template):
        raise ValidationError(f"{rel_path}: template {template!r} is not a vanilla block (minecraft:...)")
    props = data.get("properties", {})
    if not isinstance(props, dict):
        raise ValidationError(f"{rel_path}: properties must be a JSON object")
    unknown_props = set(props.keys()) - BLOCK_PROP_KEYS
    if unknown_props:
        raise ValidationError(f"{rel_path}: unknown block properties {sorted(unknown_props)}")
    if "hardness" in props and props["hardness"] < 0 and props["hardness"] != -1:
        raise ValidationError(f"{rel_path}: hardness must be >= 0 or -1")
    if "light" in props and not (0 <= int(props["light"]) <= 15):
        raise ValidationError(f"{rel_path}: light must be 0..15")
    if "sounds" in props and not VANILLA_ID_RE.match(str(props["sounds"])):
        raise ValidationError(f"{rel_path}: sounds must name a vanilla block (minecraft:...)")
    has_item = bool(data.get("item", True))
    components = data.get("components", {})
    if components and not has_item:
        raise ValidationError(f"{rel_path}: components requires item = true")
    validate_components(rel_path, components, allow_stack_and_rarity=True)
    sp = data.get("state_properties", {})
    if sp and not isinstance(sp, dict):
        raise ValidationError(f"{rel_path}: state_properties must be an object")
    for name, prop in sp.items():
        if not isinstance(name, str) or not re.match(r"^[a-z0-9_]+$", name):
            raise ValidationError(f"{rel_path}: state property name {name!r} is invalid")
        if not isinstance(prop, dict) or set(prop.keys()) - {"type", "values"}:
            raise ValidationError(f"{rel_path}: state property {name!r} must have type and optional values")
        if prop.get("type") == "boolean":
            continue
        elif prop.get("type") == "enum":
            values = prop.get("values", [])
            if not values or not all(isinstance(v, str) and re.match(r"^[a-z0-9_]+$", v) for v in values):
                raise ValidationError(f"{rel_path}: state property {name!r} enum values must be non-empty list of valid names")
        else:
            raise ValidationError(f"{rel_path}: state property {name!r} unknown type {prop.get('type')!r}")
    return has_item


def validate_item_json(rel_path: str, data: dict) -> str | None:
    unknown = set(data.keys()) - ITEM_FILE_KEYS
    if unknown:
        raise ValidationError(f"{rel_path}: unknown keys {sorted(unknown)}")
    template = data.get("template", "")
    if not isinstance(template, str) or not VANILLA_ID_RE.match(template):
        raise ValidationError(f"{rel_path}: template {template!r} is not a vanilla item (minecraft:...)")
    if "max_stack_size" in data:
        ms = data["max_stack_size"]
        if not isinstance(ms, int) or isinstance(ms, bool) or not (1 <= ms <= 99):
            raise ValidationError(f"{rel_path}: max_stack_size must be 1..99")
    if "rarity" in data and data["rarity"] not in RARITIES:
        raise ValidationError(f"{rel_path}: rarity must be one of {sorted(RARITIES)}")
    remainder = data.get("crafting_remainder")
    if remainder is not None:
        if not isinstance(remainder, str) or not ITEM_ID_RE.match(remainder):
            raise ValidationError(f"{rel_path}: crafting_remainder {remainder!r} is not a valid item id")
        if remainder == "minecraft:air":
            raise ValidationError(f"{rel_path}: crafting_remainder cannot be minecraft:air")
    validate_components(rel_path, data.get("components", {}), allow_stack_and_rarity=False)
    return remainder


def horizon_has(commit: str, pin: str | None) -> bool | None:
    """Whether Horizon at `pin` includes `commit` (from the horizon submodule); None when it cannot tell."""
    if not pin or not (HORIZON_GIT / ".git").exists():
        return None
    r = subprocess.run(["git", "merge-base", "--is-ancestor", commit, pin], cwd=HORIZON_GIT, capture_output=True)
    return {0: True, 1: False}.get(r.returncode)


def validate_creative_tab_json(rel_path: str, data: dict) -> list[str]:
    """A creative tab as Horizon reads it: title, icon and items, nothing else. Returns its items."""
    unknown = set(data.keys()) - CREATIVE_TAB_KEYS
    if unknown:
        raise ValidationError(f"{rel_path}: unknown keys {sorted(unknown)} (a tab has title, icon, items)")
    if not isinstance(data.get("title"), str) or not data["title"]:
        raise ValidationError(f"{rel_path}: title must be text or a lang key")
    items = data.get("items")
    if not isinstance(data.get("icon"), str) or not ITEM_ID_RE.match(data["icon"]):
        raise ValidationError(f"{rel_path}: icon must be an item id (namespace:path)")
    if not isinstance(items, list) or not items or not all(isinstance(i, str) and ITEM_ID_RE.match(i) for i in items):
        raise ValidationError(f"{rel_path}: items must be a non-empty list of item ids (namespace:path)")
    return items


def validate_mob_effect_json(rel_path: str, data: dict) -> None:
    unknown = set(data.keys()) - MOB_EFFECT_KEYS
    if unknown:
        raise ValidationError(f"{rel_path}: unknown keys {sorted(unknown)}")
    category = data.get("category")
    if category not in EFFECT_CATEGORIES:
        raise ValidationError(f"{rel_path}: category must be one of {sorted(EFFECT_CATEGORIES)}")
    color = data.get("color")
    if not isinstance(color, int) or isinstance(color, bool) or not (0 <= color <= 0xFF_FFFF):
        raise ValidationError(f"{rel_path}: color must be 0..=16777215 (0xFFFFFF)")


def validate_content_dir(mod_id: str, content_dir: Path,
                         horizon_pin: str | None = None) -> tuple[str, dict, list[tuple[str, bytes]]]:
    if (content_dir / "Cargo.toml").exists():
        raise ValidationError(
            f"{content_dir}/Cargo.toml exists: cargo-horizon-mod refuses folders with Cargo.toml (code is native)"
        )
    if (content_dir / "mod.wasm").exists():
        raise ValidationError(f"{content_dir}/mod.wasm exists: WebAssembly mod runtime is removed in Horizon")

    manifest_path = content_dir / "horizon-mod.toml"
    if not manifest_path.is_file():
        raise ValidationError(f"missing {manifest_path}")
    manifest_text = manifest_path.read_text(encoding="utf-8")
    manifest = tomllib.loads(manifest_text)

    unknown = set(manifest.keys()) - MANIFEST_KEYS
    if unknown:
        raise ValidationError(f"{manifest_path}: unknown keys {sorted(unknown)}")
    if manifest.get("id") != mod_id or not ID_RE.match(mod_id):
        raise ValidationError(f"{manifest_path}: id {manifest.get('id')!r} must match mod directory {mod_id!r}")
    if manifest.get("api") != "0.2":
        raise ValidationError(f"{manifest_path}: content mods must set api = '0.2'")
    if manifest.get("side", "client") not in ("both", "client"):
        raise ValidationError(f"{manifest_path}: content mod side must be 'both' or 'client'")
    caps = manifest.get("capabilities", [])
    if "registries" not in caps:
        raise ValidationError(f"{manifest_path}: content mod must include 'registries' in capabilities")

    extra_ns = manifest.get("namespaces", [])
    seen_ns: set[str] = set()
    for ns in extra_ns:
        if not NS_RE.match(ns):
            raise ValidationError(f"{manifest_path}: invalid namespace {ns!r}")
        if ns in RESERVED_NAMESPACES:
            raise ValidationError(f"{manifest_path}: reserved namespace {ns!r} cannot be claimed")
        if ns == mod_id or ns in seen_ns:
            raise ValidationError(f"{manifest_path}: duplicate namespace {ns!r}")
        seen_ns.add(ns)

    owned_ns = {mod_id, *extra_ns}
    files: list[tuple[str, bytes]] = []
    for top in ("assets", "data"):
        top_dir = content_dir / top
        if not top_dir.exists():
            continue
        for p in sorted(top_dir.rglob("*")):
            if p.is_symlink():
                raise ValidationError(f"{p}: symlinks are not allowed in content mods")
            if p.is_file():
                rel = p.relative_to(content_dir).as_posix()
                check_content_path(rel)
                files.append((rel, p.read_bytes()))

    files.sort(key=lambda item: item[0])
    declared_ids: set[str] = set()
    declared_effects: set[str] = set()
    remainders: list[tuple[str, str]] = []
    tab_items: list[tuple[str, str]] = []

    for rel, raw in files:
        parts = rel.split("/", 2)
        top, ns, rest = parts[0], parts[1], parts[2]
        allowed = (ns in owned_ns) or (top == "data" and ns in ("minecraft", "c") and rest.startswith("tags/"))
        if not allowed:
            raise ValidationError(f"{rel}: file outside owned namespaces {sorted(owned_ns)} and shared tags")
        if ns != mod_id and top == "data" and rest.startswith("horizon/"):
            raise ValidationError(f"{rel}: blocks/items/mob_effects must be declared under data/{mod_id}/horizon/")
        if ns == mod_id and top == "data" and rest.startswith("horizon/"):
            sub = rest.removeprefix("horizon/")
            if sub.startswith("block/") and sub.endswith(".json"):
                name = sub.removeprefix("block/").removesuffix(".json")
                if "/" in name:
                    raise ValidationError(f"{rel}: nested block declarations are not allowed")
                block_id = f"{mod_id}:{name}"
                has_item = validate_block_json(rel, json.loads(raw.decode("utf-8")))
                if has_item:
                    declared_ids.add(block_id)
            elif sub.startswith("item/") and sub.endswith(".json"):
                name = sub.removeprefix("item/").removesuffix(".json")
                if "/" in name:
                    raise ValidationError(f"{rel}: nested item declarations are not allowed")
                item_id = f"{mod_id}:{name}"
                remainder = validate_item_json(rel, json.loads(raw.decode("utf-8")))
                if item_id in declared_ids:
                    raise ValidationError(f"{rel}: {item_id} collides with a block item")
                declared_ids.add(item_id)
                if remainder is not None:
                    remainders.append((rel, remainder))
            elif sub.startswith("mob_effect/") and sub.endswith(".json"):
                name = sub.removeprefix("mob_effect/").removesuffix(".json")
                if "/" in name:
                    raise ValidationError(f"{rel}: nested mob_effect declarations are not allowed")
                effect_id = f"{mod_id}:{name}"
                validate_mob_effect_json(rel, json.loads(raw.decode("utf-8")))
                declared_effects.add(effect_id)
            elif sub.startswith("creative_tab/") and sub.endswith(".json"):
                if "/" in sub.removeprefix("creative_tab/"):
                    raise ValidationError(f"{rel}: nested creative tabs are not allowed")
                if horizon_has(CREATIVE_TABS_SINCE, horizon_pin) is False:
                    raise ValidationError(
                        f"{rel}: content mods' creative tabs need Horizon {CREATIVE_TABS_SINCE[:9]} (2026-10-30) or "
                        f"later; this mod pins {horizon_pin[:12]}. Repin native/horizon-native.json and the mixin "
                        "bundle's horizon_revision to a newer Horizon.")
                for item in validate_creative_tab_json(rel, json.loads(raw.decode("utf-8"))):
                    tab_items.append((rel, item))
            else:
                raise ValidationError(
                    f"{rel}: data/{mod_id}/horizon/ holds block/, item/, mob_effect/ and creative_tab/*.json only"
                )

    for rel, item in tab_items:
        # Horizon silently leaves out items nobody registers: a typo would just vanish from the tab.
        if item.startswith(f"{mod_id}:") and item not in declared_ids:
            raise ValidationError(f"{rel}: {item} is not one of this mod's items or block items")
    for rel, rem in remainders:
        if not VANILLA_ID_RE.match(rem) and rem not in declared_ids:
            raise ValidationError(
                f"{rel}: crafting_remainder {rem!r} is neither a vanilla item nor declared by {mod_id}"
            )

    return manifest_text, manifest, files


def validate_native_dir(
    mod_id: str, native_dir: Path, src_dir: Path
) -> tuple[bytes, dict, list[tuple[str, bytes]]]:
    manifest_path = native_dir / "horizon-native.json"
    if not manifest_path.is_file():
        raise ValidationError(f"missing {manifest_path}")
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes.decode("utf-8"))

    unknown = set(manifest.keys()) - NATIVE_JSON_KEYS
    if unknown:
        raise ValidationError(f"{manifest_path}: unknown keys {sorted(unknown)}")
    if manifest.get("schema") != 1:
        raise ValidationError(f"{manifest_path}: schema must be 1")
    if manifest.get("id") != mod_id or not ID_RE.match(mod_id):
        raise ValidationError(f"{manifest_path}: id must be {mod_id!r}")
    if not manifest.get("version") or not manifest.get("game"):
        raise ValidationError(f"{manifest_path}: version and game are required")
    if not HEX40_RE.match(str(manifest.get("horizon", ""))):
        raise ValidationError(f"{manifest_path}: horizon must be a 40-digit lowercase hex commit SHA")
    if not HEX40_RE.match(str(manifest.get("mixins", ""))):
        raise ValidationError(f"{manifest_path}: mixins must be a 40-digit lowercase hex commit SHA")
    hooks = manifest.get("hooks")
    if hooks is not None and (not isinstance(hooks, int) or hooks < 1):
        raise ValidationError(f"{manifest_path}: hooks must be a positive integer")

    mixin_dir = native_dir / "mixins" / mod_id
    mixin_toml_path = mixin_dir / "horizon-mixin.toml"
    if not mixin_toml_path.is_file():
        raise ValidationError(f"missing {mixin_toml_path}")
    mixin_toml = tomllib.loads(mixin_toml_path.read_text(encoding="utf-8"))

    mod_sec = mixin_toml.get("mod", {})
    compat_sec = mixin_toml.get("compatibility", {})
    # horizon-mixins reads this file with deny_unknown_fields (horizon-mixins-core manifest.rs).
    unknown_top = set(mixin_toml) - MIXIN_TOP_KEYS
    if unknown_top:
        raise ValidationError(f"{mixin_toml_path}: unknown table(s) {sorted(unknown_top)}; horizon-mixins allows {sorted(MIXIN_TOP_KEYS)}")
    unknown_mod = set(mod_sec) - MIXIN_MOD_KEYS
    if unknown_mod:
        raise ValidationError(f"{mixin_toml_path}: [mod] has {sorted(unknown_mod)}; horizon-mixins allows only {sorted(MIXIN_MOD_KEYS)}")
    if not any(mixin_toml.get(k) for k in ("injection", "bridge", "module", "state")):
        raise ValidationError(f"{mixin_toml_path}: declares no [[injection]], [[bridge]], [[module]] or [[state]]; horizon-mixins refuses an empty bundle")
    unknown_compat = set(compat_sec) - {"horizon_revision"}
    if unknown_compat:
        raise ValidationError(f"{mixin_toml_path}: [compatibility] has {sorted(unknown_compat)}; horizon-mixins allows only ['horizon_revision']")
    if mod_sec.get("id") != mod_id:
        raise ValidationError(f"{mixin_toml_path}: [mod].id must be {mod_id!r}")
    if mod_sec.get("version") != manifest["version"]:
        raise ValidationError(
            f"{mixin_toml_path}: [mod].version ({mod_sec.get('version')!r}) != horizon-native.json ({manifest['version']!r})"
        )
    if compat_sec.get("horizon_revision") != manifest["horizon"]:
        raise ValidationError(
            f"{mixin_toml_path}: [compatibility].horizon_revision != horizon-native.json horizon"
        )

    bundle_files: list[tuple[str, bytes]] = []
    for p in sorted(mixin_dir.rglob("*")):
        if p.is_symlink():
            raise ValidationError(f"{p}: symlinks are not allowed in native bundles")
        if p.is_file():
            rel = p.relative_to(mixin_dir).as_posix()
            if not (rel.endswith(".rs") or rel.endswith(".toml") or rel.startswith("data/")):
                raise ValidationError(
                    f"{p}: native bundle only allows .rs, .toml, or data/* inside mixins/{mod_id}/"
                )
            bundle_files.append((f"mixins/{mod_id}/{rel}", p.read_bytes()))

    # Also bundle pure domain Rust sources from src/ under mixins/<id>/src/ if not already present
    existing_names = {name for name, _ in bundle_files}
    if src_dir.is_dir():
        for p in sorted(src_dir.rglob("*.rs")):
            rel = p.relative_to(src_dir).as_posix()
            entry_name = f"mixins/{mod_id}/src/{rel}"
            if entry_name not in existing_names:
                bundle_files.append((entry_name, p.read_bytes()))

    bundle_files.sort(key=lambda item: item[0])
    return manifest_bytes, manifest, bundle_files


def validate_registry_toml(mod_dir: Path) -> None:
    reg_path = mod_dir / "horizon-registry.toml"
    if not reg_path.is_file():
        raise ValidationError(f"missing {reg_path}")
    try:
        reg = tomllib.loads(reg_path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        raise ValidationError(f"{reg_path}: {exc}") from exc
    unknown = set(reg.keys()) - REGISTRY_TOML_KEYS
    if unknown:
        raise ValidationError(f"{reg_path}: unknown keys {sorted(unknown)}")
    summary = reg.get("summary", "")
    if not summary or len(summary) > 160 or "\n" in summary:
        raise ValidationError(f"{reg_path}: summary must be 1..160 chars on a single line")
    for req in ("name", "description", "authors", "license"):
        if not reg.get(req):
            raise ValidationError(f"{reg_path}: missing required key {req!r}")


def asset_gaps(mod_dir: Path) -> list[dict]:
    """What Minecraft 26.3 needs for every registered block, item and effect to load and render, and
    is missing: blockstate, block and item models, item model definition (items/*.json, new since
    1.21.4), the textures those models name, a particle texture for models with their own elements,
    mob effect icons, lang names, the sound files sounds.json names, and every item and block item in one
    of the mod's creative tabs (with its title translated). A block or item drawn only
    with vanilla models is a placeholder gap, unless asset-map.json lists it in "vanilla_ok" (it
    borrows a vanilla look on purpose). Each gap is {path, for, why}; the New assets step makes one
    feature per gap."""
    mod_id = mod_dir.name
    assets = mod_dir / "content" / "assets" / mod_id
    data = mod_dir / "content" / "data" / mod_id / "horizon"
    gaps: list[dict] = []
    seen: set[str] = set()
    try:
        vanilla_ok = set(json.loads((mod_dir / "asset-map.json").read_text(encoding="utf-8")).get("vanilla_ok", []))
    except (OSError, ValueError):
        vanilla_ok = set()

    def gap(path: str, owner: str, why: str) -> None:
        if path not in seen:
            seen.add(path)
            gaps.append({"path": path, "for": owner, "why": why})

    def load(path: Path) -> dict:
        try:
            return json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return {}

    def own(ref: str) -> str | None:
        """The path inside this mod's namespace that `ref` (ns:path) names, or None for another namespace."""
        ns, _, rest = ref.partition(":") if ":" in ref else ("minecraft", "", ref)
        return rest if ns == mod_id else None

    def check_model(ref: str, owner: str, depth: int = 0) -> None:
        rel = own(ref)
        if rel is None or depth > 8:
            return
        path = assets / "models" / f"{rel}.json"
        if not path.is_file():
            gap(f"assets/{mod_id}/models/{rel}.json", owner, "model named by a blockstate or item definition")
            return
        model = load(path)
        textures = model.get("textures", {})
        for value in textures.values():
            if isinstance(value, str) and not value.startswith("#") and (tex := own(value)) is not None:
                if not (assets / "textures" / f"{tex}.png").is_file():
                    gap(f"assets/{mod_id}/textures/{tex}.png", owner, "texture named by a model")
        if model.get("elements") and "particle" not in textures:
            gap(f"assets/{mod_id}/models/{rel}.json#particle", owner, "a model with its own elements needs a particle texture")
        if isinstance(model.get("parent"), str):
            check_model(model["parent"], owner, depth + 1)

    def models_in(node: object) -> list[str]:
        found = []
        if isinstance(node, dict):
            for key, value in node.items():
                if key == "model" and isinstance(value, str):
                    found.append(value)
                else:
                    found += models_in(value)
        elif isinstance(node, list):
            for item in node:
                found += models_in(item)
        return found

    def check_refs(refs: list[str], name: str, kind: str, owner: str) -> None:
        for ref in refs:
            check_model(ref, owner)
        if refs and all(own(r) is None for r in refs) and name not in vanilla_ok:
            gap(f"assets/{mod_id}/textures/{kind}/{name}.png", owner,
                "placeholder: drawn only with vanilla models (or list it in asset-map.json vanilla_ok)")

    def check_item_definition(name: str, owner: str) -> None:
        path = assets / "items" / f"{name}.json"
        if not path.is_file():
            gap(f"assets/{mod_id}/items/{name}.json", owner, "item model definition (required since 1.21.4)")
            return
        check_refs(models_in(load(path)), name, "item", owner)

    lang = load(assets / "lang" / "en_us.json")
    for block in sorted((data / "block").glob("*.json")) if (data / "block").is_dir() else []:
        name, owner = block.stem, f"{mod_id}:{block.stem}"
        states = assets / "blockstates" / f"{name}.json"
        if not states.is_file():
            gap(f"assets/{mod_id}/blockstates/{name}.json", owner, "blockstate definition")
        else:
            check_refs(models_in(load(states)), name, "block", owner)
        if load(block).get("item", True) and (assets / "items" / f"{name}.json").is_file():
            # A block's item usually shows the block model: no separate placeholder gap for it.
            for ref in models_in(load(assets / "items" / f"{name}.json")):
                check_model(ref, owner)
        elif load(block).get("item", True):
            check_item_definition(name, owner)
        if f"block.{mod_id}.{name}" not in lang:
            gap(f"assets/{mod_id}/lang/en_us.json#block.{mod_id}.{name}", owner, "display name")
    for item in sorted((data / "item").glob("*.json")) if (data / "item").is_dir() else []:
        name, owner = item.stem, f"{mod_id}:{item.stem}"
        check_item_definition(name, owner)
        if f"item.{mod_id}.{name}" not in lang:
            gap(f"assets/{mod_id}/lang/en_us.json#item.{mod_id}.{name}", owner, "display name")
    # Every item and block item in one of the mod's creative tabs (without one, an item is only in search).
    in_tabs: set[str] = set()
    for tab in sorted((data / "creative_tab").glob("*.json")) if (data / "creative_tab").is_dir() else []:
        spec = load(tab)
        in_tabs |= {i for i in spec.get("items", []) if isinstance(i, str)}
        title = spec.get("title", "")
        key = f"itemGroup.{mod_id}.{tab.stem}"
        if key not in lang and (not isinstance(title, str) or "." in title and title not in lang):
            gap(f"assets/{mod_id}/lang/en_us.json#{key}", f"{mod_id}:{tab.stem}", "creative tab title")
    tab_file = f"data/{mod_id}/horizon/creative_tab/{mod_id}.json"
    for kind in ("block", "item"):
        for f in sorted((data / kind).glob("*.json")) if (data / kind).is_dir() else []:
            if kind == "block" and not load(f).get("item", True):
                continue
            if f"{mod_id}:{f.stem}" not in in_tabs:
                gap(f"{tab_file}#{mod_id}:{f.stem}", f"{mod_id}:{f.stem}", "not in any creative tab of the mod")
    for effect in sorted((data / "mob_effect").glob("*.json")) if (data / "mob_effect").is_dir() else []:
        name, owner = effect.stem, f"{mod_id}:{effect.stem}"
        if not (assets / "textures" / "mob_effect" / f"{name}.png").is_file():
            gap(f"assets/{mod_id}/textures/mob_effect/{name}.png", owner, "effect icon")
        if f"effect.{mod_id}.{name}" not in lang:
            gap(f"assets/{mod_id}/lang/en_us.json#effect.{mod_id}.{name}", owner, "display name")
    for event, spec in load(assets / "sounds.json").items():
        for sound in spec.get("sounds", []) if isinstance(spec, dict) else []:
            ref = sound.get("name") if isinstance(sound, dict) else sound
            if isinstance(ref, str) and (rel := own(ref)) is not None and not (assets / "sounds" / f"{rel}.ogg").is_file():
                gap(f"assets/{mod_id}/sounds/{rel}.ogg", f"{mod_id}:{event}", "sound named by sounds.json")
    return gaps


def write_deterministic_zip(out_path: Path, entries: list[tuple[str, bytes]]) -> str:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    partial = out_path.with_suffix(out_path.suffix + ".partial")
    with zipfile.ZipFile(partial, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for name, data in entries:
            info = zipfile.ZipInfo(filename=name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            zf.writestr(info, data)
    partial.replace(out_path)
    return hashlib.sha256(out_path.read_bytes()).hexdigest()


def process_mod(mod_dir: Path, pack: bool) -> None:
    mod_id = mod_dir.name
    cargo_path = mod_dir / "Cargo.toml"
    if not cargo_path.is_file():
        raise ValidationError(f"missing {cargo_path}")
    cargo_toml = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
    cargo_version = cargo_toml.get("package", {}).get("version")

    validate_registry_toml(mod_dir)
    try:
        pin = json.loads((mod_dir / "native" / "horizon-native.json").read_text(encoding="utf-8")).get("horizon")
    except (OSError, ValueError):
        pin = None
    manifest_text, content_manifest, content_files = validate_content_dir(mod_id, mod_dir / "content", pin)
    native_bytes, native_manifest, native_files = validate_native_dir(
        mod_id, mod_dir / "native", mod_dir / "src"
    )

    if not (cargo_version == content_manifest["version"] == native_manifest["version"]):
        raise ValidationError(
            f"{mod_id}: version mismatch across Cargo.toml ({cargo_version}), "
            f"content/horizon-mod.toml ({content_manifest['version']}), "
            f"and native/horizon-native.json ({native_manifest['version']})"
        )

    print(
        f"[OK] Validated {mod_id} v{cargo_version} "
        f"(content files: {len(content_files)}, native sources: {len(native_files)}, "
        f"hooks API: {native_manifest.get('hooks')})"
    )

    if pack:
        out_dir = mod_dir / "target" / "horizon-mod"
        hmod_path = out_dir / f"{mod_id}.hmod"
        hmod_entries = [("horizon-mod.toml", manifest_text.encode("utf-8")), *content_files]
        hmod_sha = write_deterministic_zip(hmod_path, hmod_entries)
        print(f"     -> Built {hmod_path} ({hmod_path.stat().st_size} bytes, sha256={hmod_sha[:16]}...)")

        hnmod_path = out_dir / f"{mod_id}.hnmod"
        hnmod_entries = [("horizon-native.json", native_bytes), *native_files]
        hnmod_sha = write_deterministic_zip(hnmod_path, hnmod_entries)
        print(f"     -> Built {hnmod_path} ({hnmod_path.stat().st_size} bytes, sha256={hnmod_sha[:16]}...)")


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate and package Rekkit mods (.hmod + .hnmod).")
    parser.add_argument("--mod", help="Specific mod ID under ./mods/ to process (default: all)")
    parser.add_argument("--check-only", action="store_true", help="Only validate manifests and files")
    parser.add_argument("--complete", action="store_true",
                        help="Also fail when an asset 26.3 needs is missing (the New assets step's check)")
    parser.add_argument("--gaps", action="store_true",
                        help="Print the missing assets as JSON (exit 1 if any) and nothing else")
    args = parser.parse_args()

    mods_root = Path("mods")
    if args.mod:
        mod_dirs = [mods_root / args.mod]
    else:
        mod_dirs = sorted(p for p in mods_root.iterdir() if p.is_dir() and (p / "Cargo.toml").is_file())

    if not mod_dirs:
        print("No mods found under ./mods/", file=sys.stderr)
        return 1

    if args.gaps:
        found = [{"mod": d.name, **g} for d in mod_dirs for g in asset_gaps(d)]
        print(json.dumps(found, indent=2))
        return 1 if found else 0

    try:
        for mod_dir in mod_dirs:
            process_mod(mod_dir, pack=not args.check_only)
            if args.complete and (found := asset_gaps(mod_dir)):
                listed = "\n".join(f"  {g['path']} ({g['for']}: {g['why']})" for g in found)
                raise ValidationError(f"{mod_dir.name}: {len(found)} asset(s) missing:\n{listed}")
            if args.complete and mod_dir.name != "example_mod" and (found := contract_check.check(mod_dir.name)):
                listed = "\n".join(f"  {p['for']}: [{p['kind']} -> {p['owner']}] {p['why']}" for p in found[:60])
                more = f"\n  ... and {len(found) - 60} more (make contract-check MOD={mod_dir.name})" if len(found) > 60 else ""
                raise ValidationError(f"{mod_dir.name}: {len(found)} difference(s) from the original (its contract):\n{listed}{more}")
    except ValidationError as exc:
        print(f"[ERROR] {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
