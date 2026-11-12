#!/usr/bin/env python3
"""A role subagent's whole brief in one block, printed by `make fleet-task BEAD=<id> ROLE=<role>`.

Every turn of an agent re-sends all its context, so a run costs roughly turns x context
(planning/token_audit/fix-plan.md). A ticket names its spec sections, Java methods and files; fetching
each one is a turn. This prints them all at once, bounded (BUDGET_CHARS, about 12k tokens):

  1. the ticket: title, what it covers, what done means, what it is after;
  2. the text of each spec section it names (a long section: the subsections about what it covers);
  3. the source of each Java member it names, from every decompiled version of the mod;
  4. each file it edits: whether it exists, and for a new one the mods/example_mod file of that kind;
  5. the outline entry (merged with its contract file) of each block and item it covers, and, when it
     writes a contract, the contract format from the mod-spec skill;
  6. the checks to run before reporting.

Usage:
  python3 scripts/fleet/brief.py <bead> [--role <role>]          (what fleet-task prints)
  python3 scripts/fleet/brief.py --mod <id> --key <ticket key> [--role <role>]   (no tracker needed)
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import re
import sys
from pathlib import Path

import tracker
import spec as spec_tool

WORK = Path(__file__).resolve().parents[2]  # this checkout: the planning files an agent writes live here
MAIN = tracker.MAIN  # assets/ (decompiled Java, archives) is the main checkout's
sys.path.insert(0, str(WORK / "scripts"))

BUDGET_CHARS = 48_000
SECTION_LINES = 120
JAVA_LINES = 120
EXAMPLE = WORK / "mods" / "example_mod"
# The mods/example_mod file to copy for each kind of content file (by path), so nobody lists example_mod.
TEMPLATES = [
    (r"/horizon/block/", "content/data/example_mod/horizon/block/ender_chest.json"),
    (r"/horizon/item/", "content/data/example_mod/horizon/item/ender_pouch.json"),
    (r"/horizon/creative_tab/", "content/data/example_mod/horizon/creative_tab/example_mod.json"),
    (r"/blockstates/", "content/assets/example_mod/blockstates/ender_chest.json"),
    (r"/models/block/", "content/assets/example_mod/models/block/ender_chest.json"),
    (r"/assets/[^/]+/items/", "content/assets/example_mod/items/ender_chest.json"),
    (r"/loot_table/", "content/data/example_mod/loot_table/blocks/ender_chest.json"),
    (r"/recipe/", "content/data/example_mod/recipe/ender_chest.json"),
]
# Big files a role follows the style of: named, not inlined (read only the part you need).
STYLE = [(r"^mods/[^/]+/src/", "mods/example_mod/src/lib.rs"), (r"^mods/[^/]+/tests/", "mods/example_mod/tests/mod_tests.rs"),
         (r"^mods/[^/]+/native/mixins/", "mods/example_mod/native/mixins/example_mod/")]
CONTENT_GEN = ("`make content-gen MOD={mod} NAME={name}` writes the blockstate, models, item model, loot table, "
               "lang keys and creative-tab entry from the contract; write only what it reports it cannot")
CHECKS = {
    "content-author": ["make content-gen MOD={mod}{names}", "make check-mods MOD={mod}",
                       "make contract-check MOD={mod} OWNER=content-author"],
    "domain-builder": ["cargo test -q --manifest-path mods/{mod}/Cargo.toml",
                       "cargo clippy -q --manifest-path mods/{mod}/Cargo.toml --all-targets -- -D warnings",
                       "make contract-check MOD={mod} OWNER=domain-builder"],
    "domain-builder-pro": ["cargo test -q --manifest-path {crate}/Cargo.toml",
                           "cargo clippy -q --manifest-path {crate}/Cargo.toml --all-targets -- -D warnings"],
    "hook-engineer": ["make check-mods MOD={mod}", "make agent-gate MOD={mod}"],
    "test-smith": ["cargo test -q --manifest-path mods/{mod}/Cargo.toml", "make e2e-gen MOD={mod}",
                   "make e2e-mod MOD={mod} DRY_RUN=1"],
    "asset-smith": ["make contract-check MOD={mod} OWNER=asset-smith", "make check-mods MOD={mod} COMPLETE=1"],
    "scaffolder": ["make check-mods MOD={mod}", "cargo test -q --manifest-path mods/{mod}/Cargo.toml"],
    "spec-writer": ["make fleet-tickets MOD={mod} CHECK=1"],
    "parity-critic": ["make contract-check MOD={mod}", "make agent-gate MOD={mod}"],
    "compat-auditor": ["make e2e-mod MOD={mod}", "make e2e-all-mods"],
    "release-clerk": ["make version-sync", "make check-mods MOD={mod} COMPLETE=1", "make pack-mods MOD={mod}"],
}
# The mod-spec sections a role needs when its ticket writes a contract file.
CONTRACT_SECTIONS = ["Outline file and contracts", "Test levels"]


def ticket_files(mod: str) -> list[dict]:
    out = []
    folder = WORK / "planning" / "mods" / f"{mod}.tickets"
    for f in sorted(folder.glob("*.json")) if folder.is_dir() else []:
        try:
            out += json.loads(f.read_text(encoding="utf-8")).get("tickets", [])
        except ValueError:
            continue
    return out


def find_ticket(mod: str, key: str | None) -> dict | None:
    return next((t for t in ticket_files(mod) if key and t.get("key") == key), None)


def spec_sections(mod: str, wanted: list[str], hints: list[str]) -> list[str]:
    path = WORK / "planning" / "mods" / f"{mod}.md"
    if not path.is_file() or not wanted:
        return []
    lines = path.read_text(encoding="utf-8").splitlines()
    found = spec_tool.sections(lines)
    out = []
    for want in wanted:
        hits = [s for s in found if want.lower() in s[1].lower()]
        hits = [h for h in hits if not any(o is not h and o[2] <= h[2] and h[3] <= o[3] for o in hits)]
        if not hits:
            out.append(f"(no spec heading contains {want!r}; `make spec-toc MOD={mod}` lists them)")
            continue
        level, title, start, end = hits[0]
        if end - start + 1 <= SECTION_LINES:
            out.append(f"<!-- spec {start}-{end} -->\n" + "\n".join(lines[start - 1:end]))
            continue
        subs = [s for s in found if s[0] > level and start < s[2] <= end]
        mine = [s for s in subs if any(h in s[1].lower() for h in hints)]
        mine = [m for m in mine if not any(o is not m and o[2] <= m[2] and m[3] <= o[3] for o in mine)]
        if mine:
            body = [f"<!-- spec {s[2]}-{s[3]} -->\n" + "\n".join(lines[s[2] - 1:min(s[3], s[2] + SECTION_LINES - 1)])
                    for s in mine]
            out.append(f"## {title} (only the parts about this ticket)\n" + "\n\n".join(body))
        else:
            cut = start + SECTION_LINES - 1
            out.append(f"<!-- spec {start}-{cut} of {end} -->\n" + "\n".join(lines[start - 1:cut])
                       + "\n(cut; the rest by subsection: " + "; ".join(s[1] for s in subs[:12]) + ")")
    return out


def decompiled_dirs(mod: str) -> list[Path]:
    import context
    spec = WORK / "planning" / "mods" / f"{mod}.md"
    text = spec.read_text(encoding="utf-8", errors="replace") if spec.is_file() else ""
    dirs = []
    for _, _, _, archive in context.archives(mod, text):
        d = MAIN / "assets" / "decompiled" / Path(archive).stem
        if d.is_dir() and d not in dirs:
            dirs.append(d)
    return dirs


def java_members(mod: str, members: list[str]) -> list[str]:
    if not members:
        return []
    import java_outline
    dirs = decompiled_dirs(mod)
    if not dirs:
        return [f"(no decompiled Java for {mod} yet; `make fleet-context MOD={mod}` names the archives)"]
    out = []
    for member in members:
        shown = ""
        for d in dirs:
            buf, err = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(err):
                try:
                    if "#" in member or ":" in member:
                        java_outline.show(d, member, 0)
                    else:  # a whole class: its members with line ranges, never its source
                        java_outline.outline(d, rf"^{re.escape(member)}$", False, False)
                except Exception as exc:  # noqa: BLE001 (one bad member never spoils the brief)
                    print(f"({member}: {exc})")
            got = buf.getvalue().strip()
            if got:
                lines = got.splitlines()
                if len(lines) > JAVA_LINES:
                    lines = lines[:JAVA_LINES] + [f"... ({len(lines) - JAVA_LINES} more lines: "
                                                  f"`make java-show DIR={d.relative_to(MAIN).as_posix()} MEMBER={member}`)"]
                shown += f"// {member} in {d.relative_to(MAIN).as_posix()}\n" + "\n".join(lines) + "\n"
        out.append(shown.strip() or f"// {member}: not found (`make java-outline DIR=<decompiled> GREP=<class>`)")
    return out


def file_notes(mod: str, files: list[str], role: str | None) -> list[str]:
    out, shown = [], set()
    for rel in files:
        full = WORK / rel
        if full.is_file():
            n = full.read_text(encoding="utf-8", errors="replace").count("\n")
            out.append(f"- {rel}: exists ({n} lines); read it before you change it")
            continue
        note = f"- {rel}: new"
        for pattern, template in TEMPLATES:
            if re.search(pattern, rel):
                note += f"; shape of {template}"
                if template not in shown and (EXAMPLE / template).is_file():
                    shown.add(template)
                    note += ":\n```json\n" + (EXAMPLE / template).read_text(encoding="utf-8").strip() + "\n```"
                break
        else:
            for pattern, style in STYLE:
                if re.search(pattern, rel):
                    note += f"; follow the style of {style} (read only the part you need)"
                    break
        out.append(note)
    return out


def covered_entries(mod: str, covers: list[str]) -> list[str]:
    if not covers:
        return []
    try:
        import contract_check
        contract = contract_check.load_contract(mod, WORK / "planning" / "mods" / f"{mod}.features.json") or {}
    except Exception:  # noqa: BLE001
        return []
    out = []
    for kind in ("blocks", "items", "features"):
        for e in contract.get(kind, []):
            if isinstance(e, dict) and e.get("id") in covers:
                out.append(f"{kind[:-1]} {e['id']}: " + json.dumps(e, separators=(",", ":")))
    scen = [s for s in contract.get("scenarios", []) if set(s.get("for", [])) & set(covers)]
    if scen:
        out.append(f"{len(scen)} scenario(s) already in the contract for these: "
                   + "; ".join(f"L{s.get('level')} {s.get('name')}" for s in scen))
    return out


def skill_sections(skill: str, headings: list[str]) -> str:
    path = WORK / ".agents" / "skills" / skill / "SKILL.md"
    if not path.is_file():
        return ""
    lines = path.read_text(encoding="utf-8").splitlines()
    found = spec_tool.sections(lines)
    parts = []
    for want in headings:
        hit = next((s for s in found if want.lower() in s[1].lower()), None)
        if hit:
            parts.append("\n".join(lines[hit[2] - 1:hit[3]]))
    return "\n\n".join(parts)


def checks(mod: str, role: str | None, covers: list[str]) -> list[str]:
    crate = "crates/rekkit_core" if mod == tracker.CORE else f"mods/{mod}"
    names = f" NAME={','.join(covers)}" if covers else ""
    return [c.format(mod=mod, crate=crate, names=names) for c in CHECKS.get(role or "", [])]


def assemble(head: str, parts: list[tuple[str, list[str] | str]]) -> str:
    out, used = [head], len(head)
    for title, body in parts:
        text = body if isinstance(body, str) else "\n\n".join(body)
        if not text.strip():
            continue
        block = f"\n## {title}\n{text.strip()}\n"
        if used + len(block) > BUDGET_CHARS:
            room = max(BUDGET_CHARS - used - 200, 0)
            out.append(block[:room] + f"\n(cut at the brief's limit of {BUDGET_CHARS // 4:,} tokens; the ticket is too "
                                      "big: say so in your report so the conductor splits it)\n")
            break
        out.append(block)
        used += len(block)
    return "".join(out)


def brief_for(mod: str, ticket: dict | None, role: str | None, title: str = "", description: str = "") -> str:
    t = ticket or {}
    role = role or t.get("role")
    covers = t.get("covers", [])
    hints = [c.replace("_", " ").lower() for c in covers] + [c.lower() for c in covers]
    files = t.get("files", [])
    head = f"# Brief: {t.get('title') or title} ({mod}{', ' + role if role else ''})\n"
    ticket_text = [t.get("description") or description or ""]
    if covers:
        ticket_text.append("Covers: " + ", ".join(covers))
    if t.get("after"):
        ticket_text.append("After (already done): " + ", ".join(t["after"]))
    parts: list[tuple[str, list[str] | str]] = [("The ticket", "\n".join(x for x in ticket_text if x))]
    entries = covered_entries(mod, covers)
    if entries:
        parts.append(("What the outline and contract say about it", entries))
    parts.append(("Files you edit", file_notes(mod, files, role)))
    if role == "content-author" and covers:
        parts.append(("Generated for you", CONTENT_GEN.format(mod=mod, name=",".join(covers)) + "."))
    if any(".contract/" in f for f in files):
        parts.append(("The contract file's format (mod-spec skill)", skill_sections("mod-spec", CONTRACT_SECTIONS)))
    parts.append(("Spec sections", spec_sections(mod, t.get("spec", []), hints)))
    parts.append(("The original's Java", java_members(mod, t.get("java", []))))
    run = checks(mod, role, covers)
    if run:
        parts.append(("Before you report, run", "\n".join(f"- `{c}`" for c in run)))
    return assemble(head, parts)


def brief(bead: str, role: str | None) -> str:
    """The brief for an issue: its ticket's, or (a story, feature or bug) its title and description."""
    try:
        issue = tracker.show(bead)
    except Exception:  # noqa: BLE001 (tracker down: the bead's prompt still says what to do)
        return ""
    mod = tracker.label_value(issue, "mod")
    if not mod:
        return ""
    ticket = find_ticket(mod, tracker.label_value(issue, "key"))
    return brief_for(mod, ticket, role or tracker.label_value(issue, "role"), issue.get("title", ""),
                     issue.get("description", ""))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("bead", nargs="?")
    parser.add_argument("--mod")
    parser.add_argument("--key")
    parser.add_argument("--role")
    args = parser.parse_args()
    if args.mod and args.key:
        ticket = find_ticket(args.mod, args.key)
        if not ticket:
            print(f"No ticket {args.key} in planning/mods/{args.mod}.tickets/.", file=sys.stderr)
            return 1
        text = brief_for(args.mod, ticket, args.role)
    elif args.bead:
        text = brief(args.bead, args.role)
    else:
        parser.print_usage(sys.stderr)
        return 1
    print(text)
    print(f"-- {len(text):,} characters, about {len(text) // 4:,} tokens", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
