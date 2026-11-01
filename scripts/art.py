#!/usr/bin/env python3
"""Build a mod's new art: mods/<id>/art/ -> PNGs in its content mod, at the original's size and palette.

Only for assets the original never had (the New assets step). Sources mirror where the PNG goes under
content/assets/<id>/:  art/textures/block/lamp.svg -> content/assets/<id>/textures/block/lamp.png.

  *.svg      drawn as vector art; rasterised with rsvg-convert to the size
  *.pixels   pixel art as text: a palette, then one row of characters per pixel row

      palette:
        k #1a1a1a
        r #c02020ff
      grid:
        ..kkkk..
        .krrrrk.

    `.` is transparent. The grid's width and height must equal the size.

Every PNG is then snapped to the mod's palette (every colour in its original textures: the asset-map
entries whose origin is not generated), with alpha made fully opaque or fully clear unless the
originals use partial alpha, so new art matches the old (16 px, no anti-aliasing, same colours).
Size: asset-map.json "resolution" (default 16), but 18 for effect icons (textures/mob_effect/), as
vanilla's are. Each output is recorded in asset-map.json
"generated" with origin "generated" and its source, for the human's review.

Usage: make art MOD=<id>      python3 scripts/art.py --mod <id> [--check]
"""

from __future__ import annotations

import argparse
import json
import shutil
import struct
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Folders whose vanilla textures have a fixed size, whatever the mod's resolution.
FIXED_SIZE = {"textures/mob_effect": 18}


# ---------------------------------------------------------------- PNG (8-bit, non-interlaced)

def read_png(path: Path) -> tuple[int, int, list[tuple[int, int, int, int]]]:
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"{path}: not a PNG")
    pos, idat, palette, trns = 8, b"", [], b""
    width = height = depth = ctype = interlace = 0
    while pos < len(data):
        length, kind = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            width, height, depth, ctype, _, _, interlace = struct.unpack(">IIBBBBB", chunk)
        elif kind == b"PLTE":
            palette = [tuple(chunk[i:i + 3]) for i in range(0, len(chunk), 3)]
        elif kind == b"tRNS":
            trns = chunk
        elif kind == b"IDAT":
            idat += chunk
    if depth != 8 or interlace:
        raise ValueError(f"{path}: only 8-bit, non-interlaced PNGs are supported (bit depth {depth})")
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[ctype]
    raw = zlib.decompress(idat)
    stride = width * channels
    rows, prev = [], bytearray(stride)
    for y in range(height):
        f = raw[y * (stride + 1)]
        line = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for x in range(stride):
            a = line[x - channels] if x >= channels else 0
            b, c = prev[x], prev[x - channels] if x >= channels else 0
            if f == 1:
                line[x] = (line[x] + a) & 255
            elif f == 2:
                line[x] = (line[x] + b) & 255
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line)
        prev = line
    pixels = []
    for line in rows:
        for x in range(width):
            px = line[x * channels:(x + 1) * channels]
            if ctype == 0:
                pixels.append((px[0], px[0], px[0], 255))
            elif ctype == 2:
                pixels.append((px[0], px[1], px[2], 255))
            elif ctype == 3:
                r, g, b = palette[px[0]]
                pixels.append((r, g, b, trns[px[0]] if px[0] < len(trns) else 255))
            elif ctype == 4:
                pixels.append((px[0], px[0], px[0], px[1]))
            else:
                pixels.append(tuple(px))
    return width, height, pixels


def write_png(path: Path, width: int, height: int, pixels: list[tuple[int, int, int, int]]) -> None:
    raw = b"".join(b"\x00" + bytes(c for px in pixels[y * width:(y + 1) * width] for c in px) for y in range(height))

    def chunk(kind: bytes, body: bytes) -> bytes:
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
                     + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


# ---------------------------------------------------------------- sources

def from_pixels(path: Path, size: int) -> list[tuple[int, int, int, int]]:
    palette = {".": (0, 0, 0, 0)}
    grid, section = [], None
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") and section != "palette":
            continue
        if stripped in ("palette:", "grid:"):
            section = stripped[:-1]
        elif section == "palette":
            char, colour = stripped.split()
            colour = colour.lstrip("#")
            palette[char] = tuple(int(colour[i:i + 2], 16) for i in (0, 2, 4)) + (int(colour[6:8], 16) if len(colour) == 8 else 255,)
        elif section == "grid":
            grid.append(stripped)
    if len(grid) != size or any(len(row) != size for row in grid):
        raise ValueError(f"{path}: the grid must be {size}x{size} (it is {len(grid[0]) if grid else 0}x{len(grid)})")
    unknown = {c for row in grid for c in row} - set(palette)
    if unknown:
        raise ValueError(f"{path}: characters not in the palette: {''.join(sorted(unknown))}")
    return [palette[c] for row in grid for c in row]


def from_svg(path: Path, size: int) -> list[tuple[int, int, int, int]]:
    if not shutil.which("rsvg-convert"):
        sys.exit("rsvg-convert is needed for SVG art: brew install librsvg (Linux: sudo apt install librsvg2-bin)")
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "art.png"
        subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(out), str(path)], check=True)
        w, h, pixels = read_png(out)
    if (w, h) != (size, size):
        raise ValueError(f"{path}: rendered {w}x{h}, expected {size}x{size}")
    return pixels


# ---------------------------------------------------------------- palette

def mod_palette(mod_dir: Path, amap: dict) -> tuple[list[tuple[int, int, int]], bool]:
    """Colours of the mod's original textures, and whether they use partial transparency."""
    generated = {g["out"] for g in amap.get("generated", [])}
    colours, partial = set(), False
    for png in (mod_dir / "content" / "assets" / mod_dir.name / "textures").rglob("*.png"):
        if png.relative_to(mod_dir).as_posix() in generated:
            continue
        try:
            for r, g, b, a in read_png(png)[2]:
                if a:
                    colours.add((r, g, b))
                partial |= 0 < a < 255
        except (ValueError, KeyError):
            continue
    return sorted(colours), partial


def snap(pixels, palette, partial):
    out = []
    for r, g, b, a in pixels:
        if not partial:
            a = 255 if a >= 128 else 0
        if a == 0:
            out.append((0, 0, 0, 0))
            continue
        if palette:
            r, g, b = min(palette, key=lambda c: (c[0] - r) ** 2 + (c[1] - g) ** 2 + (c[2] - b) ** 2)
        out.append((r, g, b, a))
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mod", required=True)
    parser.add_argument("--check", action="store_true", help="Exit 1 if a PNG is missing or out of date; write nothing")
    args = parser.parse_args()

    mod_dir = ROOT / "mods" / args.mod
    art = mod_dir / "art"
    map_path = mod_dir / "asset-map.json"
    amap = json.loads(map_path.read_text(encoding="utf-8")) if map_path.is_file() else {"mod": args.mod}
    sources = sorted(p for p in art.rglob("*") if p.suffix in (".svg", ".pixels")) if art.is_dir() else []
    if not sources:
        print(f"No art sources in {art.relative_to(ROOT)}.")
        return 0
    resolution = int(amap.get("resolution", 16))
    palette, partial = mod_palette(mod_dir, amap)
    entries = {g["out"]: g for g in amap.get("generated", [])}
    stale = []
    for src in sources:
        rel = src.relative_to(art).with_suffix(".png")
        size = next((v for k, v in FIXED_SIZE.items() if rel.as_posix().startswith(k + "/")), resolution)
        out = mod_dir / "content" / "assets" / args.mod / rel
        pixels = from_svg(src, size) if src.suffix == ".svg" else from_pixels(src, size)
        pixels = snap(pixels, palette, partial)
        key = out.relative_to(mod_dir).as_posix()
        entry = {"source": src.relative_to(mod_dir).as_posix(), "out": key, "origin": "generated", "size": size,
                 "palette": f"{len(palette)} colours from the originals" if palette else "none yet (no original textures)"}
        if args.check:
            if not out.is_file() or read_png(out)[2] != pixels or entries.get(key) != entry:
                stale.append(key)
            continue
        write_png(out, size, size, pixels)
        entries[key] = entry
        print(f"  {entry['source']} -> {key} ({size}x{size}, {entry['palette']})")
    if args.check:
        if stale:
            print("Out of date (make art MOD=%s): %s" % (args.mod, ", ".join(stale)), file=sys.stderr)
            return 1
        print(f"{len(sources)} art file(s) up to date.")
        return 0
    amap["generated"] = sorted(entries.values(), key=lambda g: g["out"])
    map_path.write_text(json.dumps(amap, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
