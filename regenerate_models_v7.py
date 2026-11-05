"""Extract missing sprites and regenerate models from contract files."""
import zipfile
import json
import struct
import zlib
import sys
from pathlib import Path

sys.path.insert(0, 'scripts')
from contract_check import png_bytes

texture_dir = Path('mods/redpower_2/content/assets/redpower_2/textures/block')
models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')
contract_dir = Path('planning/mods/redpower_2.contract')

# Map sheet names to (zip_path, png_path_in_zip)
SHEET_SOURCES = {
    'eloraam/control/control1.png': ('assets/direwolf20/mods/redpowermechanical.zip', 'eloraam/control/control1.png'),
    'eloraam/machine/machine1.png': ('assets/direwolf20/mods/redpowermechanical.zip', 'eloraam/machine/machine1.png'),
    'eloraam/wiring/redpower1.png': ('assets/direwolf20/mods/redpowerdigital.zip', 'eloraam/wiring/redpower1.png'),
    'eloraam/world/world1.png': ('assets/direwolf20/mods/redpowermechanical.zip', 'eloraam/world/world1.png'),
    'eloraam/compat/compat1.png': ('assets/direwolf20/mods/redpowercompat.zip', 'eloraam/compat/compat1.png'),
    'eloraam/base/base1.png': ('assets/direwolf20/mods/redpowercore.zip', 'eloraam/base/base1.png'),
}

def get_sheet_data(sheet_name):
    """Load PNG bytes from the correct ZIP."""
    src = SHEET_SOURCES.get(sheet_name)
    if not src:
        return None, None, None
    zip_path, png_path = src
    with zipfile.ZipFile(zip_path) as z:
        data = z.read(png_path)
    return png_bytes(data)

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
                pix = px[y * w + x]
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

# Step 1: Find all needed sprites from contracts
sprites_needed = set()
for f in sorted(contract_dir.glob('*.json')):
    data = json.load(open(f))
    for face, ref in data.get('faces', {}).items():
        if '#' in ref and ref.startswith('eloraam/'):
            sheet, idx = ref.split('#')
            sprites_needed.add((sheet, int(idx)))

# Also check features.json
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

# Step 2: Extract missing sprites
print("=== Extracting missing sprites ===")
for sheet, indices in sorted(by_sheet.items()):
    src = SHEET_SOURCES.get(sheet)
    if not src:
        print(f'WARNING: No source found for {sheet}')
        continue
    
    zip_path, png_path = src
    sheet_short = png_path.split('/')[-1].replace('.png', '')
    
    with zipfile.ZipFile(zip_path) as z:
        data = z.read(png_path)
    
    w, h, px = png_bytes(data)
    existing = {int(f.stem.split('_')[-1]) for f in texture_dir.glob(f'{sheet_short}_*.png')}
    
    missing = sorted(indices - existing)
    if not missing:
        print(f'{sheet_short}: all sprites present')
        continue
    
    print(f'{sheet_short}: extracting {missing}')
    for idx in missing:
        out_path = texture_dir / f'{sheet_short}_{idx:03d}.png'
        extract_sprite(data, idx, out_path)
        print(f'  extracted sprite {idx}')

# Step 3: Regenerate models from contract files
print("\n=== Regenerating models ===")

def parse_ref(ref):
    """Parse 'sheet.png#index' into (sheet_short, index)."""
    if '#' not in ref:
        return None, None
    sheet, idx = ref.split('#')
    sheet_short = sheet.split('/')[-1].replace('.png', '')
    return sheet_short, int(idx)

def shape_to_elements(shape):
    """Convert shape spec to from/to and parent type."""
    if shape == 'cross':
        return 'minecraft:block/cross', None
    if isinstance(shape, list) and len(shape) == 6:
        from_z, from_y, from_x = shape[0:3]
        to_z, to_y, to_x = shape[3:6]
        is_full = (from_z == 0 and from_y == 0 and from_x == 0 and 
                   to_z == 16 and to_y == 16 and to_x == 16)
        if is_full:
            return 'minecraft:block/cube_all', None
        # Thin shape
        return 'minecraft:block/cube', [(from_x, from_y, from_z, to_x, to_y, to_z)]
    return 'minecraft:block/cube_all', None

# Process each contract
for f in sorted(contract_dir.glob('*.json')):
    name = f.stem
    data = json.load(open(f))
    
    shape = data.get('shape')
    faces = data.get('faces', {})
    
    if not faces:
        continue
    
    # Determine parent and shape
    parent, elem_boxes = shape_to_elements(shape)
    
    # Build texture keys from face mappings
    # FACE mapping: top->up, bottom->down, front->north, back->south, side->east/west
    FACE_MAP = {
        'top': 'up', 'bottom': 'down',
        'front': 'north', 'back': 'south',
        'side': None, 'all': None,
    }
    
    textures = {}
    element_faces = {}
    
    # First handle specific face mappings
    for face_name, ref in faces.items():
        sheet_short, idx = parse_ref(ref)
        if sheet_short is None:
            continue
        
        tex_key = FACE_MAP.get(face_name)
        tex_name = f'redpower_2:block/{sheet_short}_{idx:03d}'
        
        if tex_key:
            textures[tex_key] = tex_name
            if elem_boxes:
                element_faces[tex_key] = {"texture": f"#{tex_key}"}
        elif face_name == 'side':
            # side applies to all horizontal faces
            for d in ['north', 'south', 'east', 'west']:
                if d not in textures:
                    textures[d] = tex_name
                    if elem_boxes:
                        element_faces[d] = {"texture": f"#{d}"}
        elif face_name == 'all':
            # 'all' applies to all 6 faces
            for d in ['north', 'south', 'east', 'west', 'up', 'down']:
                if d not in textures:
                    textures[d] = tex_name
                    if elem_boxes:
                        element_faces[d] = {"texture": f"#{d}"}
    
    # For cube_all, add 'all' key pointing to first texture
    if parent == 'minecraft:block/cube_all' and textures:
        first_tex = next(iter(textures.values()))
        textures['all'] = first_tex
    
    model = {"parent": parent, "textures": textures}
    
    if elem_boxes:
        # Use first box for elements
        box = elem_boxes[0]
        model["elements"] = [{
            "from": [box[0], box[1], box[2]],
            "to": [box[3], box[4], box[5]],
            "faces": element_faces
        }]
    
    out_path = models_dir / f'{name}.json'
    with open(out_path, 'w') as f:
        json.dump(model, f, indent=2)
    print(f'Generated {name}: parent={parent}, textures={list(textures.keys())}')

print("\nDone!")