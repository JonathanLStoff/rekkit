"""Regenerate all block models with correct face mappings."""
import json
from pathlib import Path

# Load features
features = json.load(open('planning/mods/redpower_2.features.json'))

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

# Face mapping from features to Minecraft model faces
# features 'top' -> minecraft 'up'
# features 'bottom' -> minecraft 'down'
# features 'front' -> minecraft 'north'
# features 'back' -> minecraft 'south'
# features 'side' -> minecraft 'north', 'south', 'east', 'west'
FACE_MAP = {
    'up': 'up', 'down': 'down',
    'north': 'north', 'south': 'south', 'east': 'east', 'west': 'west',
    'top': 'up', 'bottom': 'down',
    'front': 'north', 'back': 'south',
    'side': None,  # special - maps to all cardinal
}

def get_texture(ref):
    """Convert feature face ref to expected texture name."""
    if '#' in ref and ref.startswith('eloraam/'):
        sheet, idx = ref.split('#')
        sheet_short = sheet.split('/')[-1].replace('.png', '')
        return f'{sheet_short}_{int(idx):03d}'
    return None

# Process each block
for b in features.get('blocks', []):
    bid = b['id']
    shape = b.get('shape')
    faces = b.get('faces', {})
    
    if not faces:
        continue
    
    # Build expected textures with proper face mapping
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
    
    # Process each face
    for face, tex_name in expected.items():
        mc_key = FACE_MAP.get(face)
        new_tex = f'redpower_2:block/{tex_name}'
        
        if mc_key is None and face in ('front', 'back', 'side'):
            # 'front' -> north, 'back' -> south, 'side' -> all cardinal
            if face == 'front':
                textures['north'] = new_tex
                element_faces['north'] = {"texture": f"#{'north'}"}
            elif face == 'back':
                textures['south'] = new_tex
                element_faces['south'] = {"texture": f"#{'south'}"}
            elif face == 'side':
                for dir in ['north', 'south', 'east', 'west']:
                    textures[dir] = new_tex
                    element_faces[dir] = {"texture": f"#{dir}"}
        elif mc_key is not None:
            textures[mc_key] = new_tex
            element_faces[mc_key] = {"texture": f"#{mc_key}"}
    
    # For cube_all, add 'all' texture (use first available)
    if parent == "minecraft:block/cube_all":
        if textures:
            first_key = next(k for k in textures if k != 'particle')
            textures['all'] = textures[first_key]
    
    # Fill missing cardinal faces for thin shapes
    if is_thin:
        for dir in ['north', 'south', 'east', 'west', 'up', 'down']:
            if dir not in textures:
                default_tex = next(iter(textures.values())) if textures else "redpower_2:block/machine1_000"
                textures[dir] = default_tex
                element_faces[dir] = {"texture": f"#{dir}"}
    
    model = {"parent": parent, "textures": textures}
    
    if is_thin:
        model["elements"] = [{
            "from": list(shape[0:3]),
            "to": list(shape[3:6]),
            "faces": element_faces
        }]
    
    with open(models_dir / f'{bid}.json', 'w') as f:
        json.dump(model, f, indent=2)
    print(f'Generated {bid}')

print('\nDone!')