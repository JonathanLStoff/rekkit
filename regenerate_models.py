"""Regenerate all block models from features.json. Clean and correct."""
import json
from pathlib import Path

# Load features
features = json.load(open('planning/mods/redpower_2.features.json'))

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

# Build expected texture mapping
def get_texture(ref):
    """Convert feature face ref to expected texture name."""
    if '#' in ref and ref.startswith('eloraam/'):
        sheet, idx = ref.split('#')
        sheet_short = sheet.split('/')[-1].replace('.png', '')
        return f'{sheet_short}_{int(idx):03d}'
    return None

# Face name mapping from feature to Minecraft model faces
# feature 'top' -> minecraft 'up', 'bottom' -> 'down', 'side' -> all cardinal
FACE_MAP = {
    'up': 'up', 'down': 'down',
    'north': 'north', 'south': 'south', 'east': 'east', 'west': 'west',
    'top': 'up', 'bottom': 'down',
    'side': None,  # special - maps to all cardinal
}

# Process each block
for b in features.get('blocks', []):
    bid = b['id']
    shape = b.get('shape')
    faces = b.get('faces', {})
    
    if not faces:
        continue
    
    # Build expected textures
    expected = {}
    for face, ref in faces.items():
        tex = get_texture(ref)
        if tex:
            expected[face] = tex
    
    if not expected:
        continue
    
    # Determine if thin shape
    is_thin = False
    if shape and len(shape) == 6:
        from_z, from_y, from_x = shape[0:3]
        to_z, to_y, to_x = shape[3:6]
        is_thin = to_y < 16 or from_y > 0 or from_z > 0 or to_z < 16 or from_x > 0 or to_x < 16
    
    parent = "minecraft:block/cube" if is_thin else "minecraft:block/cube_all"
    
    textures = {}
    element_faces = {}
    
    for face, tex_name in expected.items():
        mc_key = FACE_MAP.get(face, face)
        new_tex = f'redpower_2:block/{tex_name}'
        
        if mc_key is None and face == 'side':
            # 'side' maps to all cardinal directions
            for dir in ['north', 'south', 'east', 'west']:
                textures[dir] = new_tex
                element_faces[dir] = {"texture": f"#{dir}"}
        elif mc_key is not None:
            textures[mc_key] = new_tex
            element_faces[mc_key] = {"texture": f"#{mc_key}"}
    
    model = {"parent": parent, "textures": textures}
    
    if is_thin:
        # Fill in missing faces with first available texture
        default_tex = next(iter(textures.values())) if textures else "redpower_2:block/machine1_000"
        default_key = "north"
        for dir in ['north', 'south', 'east', 'west', 'up', 'down']:
            if dir not in textures:
                textures[dir] = default_tex
                element_faces[dir] = {"texture": f"#{dir}"}
        
        model["elements"] = [{
            "from": list(shape[0:3]),
            "to": list(shape[3:6]),
            "faces": element_faces
        }]
    
    with open(models_dir / f'{bid}.json', 'w') as f:
        json.dump(model, f, indent=2)
    print(f'Generated {bid}')

print('\nDone!')
