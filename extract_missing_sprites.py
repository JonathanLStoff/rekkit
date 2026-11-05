"""Extract missing sprites from source PNG sheets."""
import zipfile
import json
import struct
import zlib
import sys
from pathlib import Path

sys.path.insert(0, 'scripts')
from contract_check import png_bytes, Sheets

texture_dir = Path('mods/redpower_2/content/assets/redpower_2/textures/block')

# Load all needed sprite references from contract files
contract_dir = Path('planning/mods/redpower_2.contract')
sprites_needed = set()
for f in sorted(contract_dir.glob('*.json')):
    data = json.load(open(f))
    for face, ref in data.get('faces', {}).items():
        if '#' in ref and ref.startswith('eloraam/'):
            sheet, idx = ref.split('#')
            sprites_needed.add((sheet, int(idx)))

# Also check features.json for any additional refs
features = json.load(open('planning/mods/redpower_2.features.json'))
for b in features.get('blocks', []):
    for face, ref in b.get('faces', {}).items():
        if '#' in ref and ref.startswith('eloraam/'):
            sheet, idx = ref.split('#')
            sprites_needed.add((sheet, int(idx)))

# Group by sheet
by_sheet = {}
for sheet, idx in sprites_needed:
    by_sheet.setdefault(sheet, set()).add(idx)

# Map sheet names to ZIP locations
SHEET_SOURCES = {
    'eloraam/control/control1.png': ('assets/direwolf20/mods/redpowermechanical.zip', 'eloraam/control/control1.png'),
    'eloraam/machine/machine1.png': ('assets/direwolf20/mods/redpowermechanical.zip', 'eloraam/machine/machine1.png'),
    'eloraam/wiring/redpower1.png': ('assets/direwolf20/mods/redpowerwiring.zip', 'eloraam/wiring/redpower1.png'),
    'eloraam/compat/compat1.png': ('assets/direwolf20/mods/redpowercompat.zip', 'eloraam/compat/compat1.png'),
    'eloraam/base/base1.png': ('assets/direwolf20/mods/redpowercore.zip', 'eloraam/base/base1.png'),
    'eloraam/world/world1.png': ('assets/direwolf20/mods/redpowerworld.zip', 'eloraam/world/world1.png'),
}

def extract_sprite(z_data, idx, out_path):
    """Extract a single sprite as a PNG file."""
    w, h, px = png_bytes(z_data)
    size = w // 16
    
    col = idx % 16
    row = idx // 16
    pixels = []
    for sy in range(size):
        for sx in range(size):
            y = row * size + sy
            x = col * size + sx
            if y < h and x < w:
                pix = px[(y * w + x) * 4:(y * w + x) * 4 + 4]
                pixels.append((int(pix[0]), int(pix[1]), int(pix[2]), int(pix[3])))
    
    # Write PNG
    header = b'\x89PNG\r\n\x1a\n'
    ihdr_data = struct.pack('>IIBBBBB', size, size, 8, 6, 0, 0, 0)
    ihdr_crc = struct.pack('>I', zlib.crc32(b'IHDR' + ihdr_data) & 0xffffffff)
    ihdr = struct.pack('>I', len(ihdr_data)) + b'IHDR' + ihdr_data + ihdr_crc
    
    sprite_raw = b''
    for y in range(size):
        sprite_raw += b'\x00'
        for x in range(size):
            p = pixels[y * size + x]
            sprite_raw += struct.pack('BBBB', p[0], p[1], p[2], p[3])
    compressed = zlib.compress(sprite_raw)
    idat_crc = struct.pack('>I', zlib.crc32(b'IDAT' + compressed) & 0xffffffff)
    idat = struct.pack('>I', len(compressed)) + b'IDAT' + compressed + idat_crc
    
    iend_crc = struct.pack('>I', zlib.crc32(b'IEND') & 0xffffffff)
    iend = struct.pack('>I', 0) + b'IEND' + iend_crc
    
    with open(out_path, 'wb') as f:
        f.write(header + ihdr + idat + iend)

# Process each sheet
for sheet, indices in sorted(by_sheet.items()):
    src = SHEET_SOURCES.get(sheet)
    if not src:
        print(f'WARNING: No source found for {sheet}')
        continue
    
    zip_path, png_path = src
    print(f'Processing {sheet} (need sprites: {sorted(indices)})')
    
    with zipfile.ZipFile(zip_path) as z:
        data = z.read(png_path)
    
    w, h, px = png_bytes(data)
    size = w // 16
    existing = {int(f.stem.split('_')[-1]) for f in texture_dir.glob(f'{Path(png_path).stem.split("/")[-1]}_*.png')}
    
    for idx in sorted(indices):
        if idx in existing:
            print(f'  sprite {idx}: already exists')
            continue
        out_path = texture_dir / f'{Path(png_path).stem.split("/")[-1]}_{idx:03d}.png'
        extract_sprite(data, idx, out_path)
        print(f'  extracted sprite {idx} -> {out_path.name}')

print('\nDone!')