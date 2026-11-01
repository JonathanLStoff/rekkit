#!/usr/bin/env python3
"""Read decompiled Java a piece at a time, so an agent never pulls whole classes into its context.

  outline  every class under a folder (or those matching --grep), one line per field, constant (with
           its value), constructor, method and static block, with line ranges and sizes: what a class
           holds and where, at a few percent of its text
  show     one member's source with line numbers (Class#member), every overload; or a line range
           (Class:120-180)
  find     where an identifier or text occurs: file:line and the trimmed line, at most --limit hits

The decompiled code is under assets/decompiled/ (make decompile). Read it this way: outline first,
then show only the members a spec section needs.

Usage:
  make java-outline DIR=assets/decompiled/RedPowerCore-2.0pr5b2 [GREP=Alloy]
  make java-show DIR=assets/decompiled/RedPowerCore-2.0pr5b2 MEMBER=TileAlloyFurnace#updateEntity
  make java-find DIR=assets/decompiled/RedPowerCore-2.0pr5b2 TEXT=cookTime
  python3 scripts/java_outline.py outline|show|find <dir> [...]
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODIFIERS = r"(?:(?:public|protected|private|static|final|abstract|synchronized|native|strictfp|transient|volatile|default)\s+)*"
CLASS = re.compile(rf"^\s*{MODIFIERS}(class|interface|enum|record|@interface)\s+(\w+)([^{{]*)")
METHOD = re.compile(rf"^\s*(?:@\w+(?:\([^)]*\))?\s+)*{MODIFIERS}(?:<[^>]+>\s+)?(?:[\w.$\[\]<>?,]+(?:\s*<[^>]*>)?\s+)?(\w+)\s*\(")
NOT_METHODS = {"if", "for", "while", "switch", "catch", "return", "new", "throw", "synchronized", "else", "do", "try"}


@dataclass
class Member:
    kind: str  # field, const, method, ctor, static, class
    text: str
    start: int
    end: int


@dataclass
class JavaClass:
    name: str
    path: Path
    header: str
    start: int
    end: int = 0
    members: list[Member] = field(default_factory=list)


def code_only(line: str, in_block: bool) -> tuple[str, bool]:
    """The line without strings, chars and comments (so braces in them do not count)."""
    out, i, n = [], 0, len(line)
    while i < n:
        if in_block:
            j = line.find("*/", i)
            if j < 0:
                return "".join(out), True
            i, in_block = j + 2, False
            continue
        c = line[i]
        if line.startswith("//", i):
            break
        if line.startswith("/*", i):
            in_block, i = True, i + 2
            continue
        if c in "\"'":
            j = i + 1
            while j < n and line[j] != c:
                j += 2 if line[j] == "\\" else 1
            out.append(c + c)
            i = j + 1
            continue
        out.append(c)
        i += 1
    return "".join(out), in_block


def parse(path: Path) -> list[JavaClass]:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    classes: list[JavaClass] = []
    stack: list[tuple[JavaClass, int]] = []  # class, depth of its body
    open_member: tuple[Member, int] | None = None  # member whose body is open, depth to return to
    depth, in_block = 0, False
    for no, raw in enumerate(lines, 1):
        code, in_block = code_only(raw, in_block)
        text = raw.strip()
        body = stack[-1][1] if stack else None
        if open_member is None and code.strip():
            if (m := CLASS.match(code)) and (body is None or depth == body):
                cls = JavaClass(m.group(2), path, " ".join(f"{m.group(1)} {m.group(2)}{m.group(3)}".split()), no)
                classes.append(cls)
                if stack:
                    stack[-1][0].members.append(Member("class", cls.header, no, no))
                stack.append((cls, depth + 1))
            elif body is not None and depth == body:
                cls = stack[-1][0]
                head = code.split("{")[0].strip()
                m = METHOD.match(code)
                if text.startswith("static {") or text == "static{":
                    open_member = (Member("static", "static {}", no, no), depth)
                elif m and m.group(1) not in NOT_METHODS and ("=" not in head.split("(")[0]):
                    kind = "ctor" if m.group(1) == cls.name else "method"
                    sig = " ".join(head.rstrip(";").split())
                    member = Member(kind, sig, no, no)
                    if "{" in code:
                        open_member = (member, depth)
                    cls.members.append(member)
                elif code.rstrip().endswith(";") or "=" in head:
                    kind = "const" if re.search(r"\bstatic\b", head) and re.search(r"\bfinal\b", head) else "field"
                    cls.members.append(Member(kind, " ".join(text.rstrip(";").split()), no, no))
            if open_member and open_member[0].kind == "static" and open_member[0] not in stack[-1][0].members:
                stack[-1][0].members.append(open_member[0])
        depth += code.count("{") - code.count("}")
        if open_member and depth <= open_member[1]:
            open_member[0].end = no
            open_member = None
        while stack and depth < stack[-1][1]:
            stack.pop()[0].end = no
    for cls in classes:
        cls.end = cls.end or len(lines)
    return classes


def java_files(root: Path) -> list[Path]:
    if root.is_file():
        return [root]
    return sorted(root.rglob("*.java"))


ACCESS = re.compile(r"\b(?:public|protected|private|final|synchronized)\s+")


def outline(root: Path, grep: str | None, consts_only: bool, classes_only: bool) -> None:
    pattern = re.compile(grep, re.I) if grep else None
    total = shown = 0
    for path in java_files(root):
        for cls in parse(path):
            total += 1
            members = [m for m in cls.members if not consts_only or m.kind == "const"]
            if pattern and not pattern.search(cls.name) and not any(pattern.search(m.text) for m in members):
                continue
            shown += 1
            rel = path.relative_to(root) if root.is_dir() else path.name
            methods = sum(m.kind in ("method", "ctor") for m in cls.members)
            print(f"{ACCESS.sub('', cls.header)[:150]}  [{rel}:{cls.start}-{cls.end}, {methods} methods]")
            if classes_only:
                continue
            for m in members:
                where = f"{m.start}-{m.end}" if m.end > m.start else str(m.start)
                kind = "" if m.kind in ("method", "field") else m.kind + " "
                print(f"  {where:>9} {kind}{ACCESS.sub('', m.text)[:160]}")
    print(f"-- {shown} of {total} classes. Next: show <dir> Class#member (or Class:start-end).", file=sys.stderr)


def show(root: Path, spec: str, context: int) -> int:
    m = re.fullmatch(r"([\w$]+)(?:([#:])(.+))?", spec)
    if not m:
        print("Give Class, Class#member or Class:start-end.", file=sys.stderr)
        return 1
    name, sep, rest = m.group(1), m.group(2) or "", m.group(3) or ""
    hits = 0
    for path in java_files(root):
        for cls in parse(path):
            if cls.name != name:
                continue
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
            if sep == ":":
                a, _, b = rest.partition("-")
                ranges = [(int(a), int(b or a))]
            elif sep == "#":
                ranges = [(m.start, m.end) for m in cls.members if m.kind in ("method", "ctor", "static")
                          and (re.search(rf"\b{re.escape(rest)}\s*\(", m.text) or (rest == "static" and m.kind == "static"))]
            else:
                ranges = [(cls.start, cls.end)]
            for a, b in ranges:
                hits += 1
                print(f"// {path.relative_to(root) if root.is_dir() else path.name}:{a}-{b}")
                for no in range(max(a - context, 1), min(b + context, len(lines)) + 1):
                    print(f"{no:>5} {lines[no - 1]}")
    if not hits:
        print(f"No {spec} under {root} (outline lists the classes and members).", file=sys.stderr)
        return 1
    return 0


def find(root: Path, text: str, limit: int) -> int:
    pattern = re.compile(text if any(c in text for c in "\\[]()|*+?^$") else re.escape(text))
    hits = 0
    for path in java_files(root):
        for no, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            if pattern.search(line):
                hits += 1
                if hits <= limit:
                    print(f"{path.relative_to(root) if root.is_dir() else path.name}:{no}: {line.strip()[:160]}")
    if hits > limit:
        print(f"-- {hits} hits, first {limit} shown; narrow the text or the folder.", file=sys.stderr)
    return 0 if hits else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)
    o = sub.add_parser("outline", help="Classes and their members, with line ranges")
    o.add_argument("dir", type=Path)
    o.add_argument("--grep", help="Only classes whose name or a member matches (regex)")
    o.add_argument("--consts", action="store_true", help="Only constants (static final fields and their values)")
    o.add_argument("--classes", action="store_true", help="Only the classes, one line each (start here)")
    s = sub.add_parser("show", help="One member (Class#member), a range (Class:120-180) or a class (Class)")
    s.add_argument("dir", type=Path)
    s.add_argument("member")
    s.add_argument("--context", type=int, default=0, help="Extra lines around it")
    f = sub.add_parser("find", help="Where a name or text occurs")
    f.add_argument("dir", type=Path)
    f.add_argument("text")
    f.add_argument("--limit", type=int, default=40)
    args = parser.parse_args()
    root = args.dir if args.dir.is_absolute() else (Path.cwd() / args.dir)
    if not root.exists():
        print(f"No such folder: {args.dir} (make decompile JAR=<archive> first)", file=sys.stderr)
        return 1
    if args.cmd == "outline":
        outline(root, args.grep, args.consts, args.classes)
        return 0
    if args.cmd == "show":
        return show(root, args.member, args.context)
    return find(root, args.text, args.limit)


if __name__ == "__main__":
    raise SystemExit(main())
