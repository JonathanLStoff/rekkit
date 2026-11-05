"""Fix model textures and shapes based on features.json."""
import json
from pathlib import Path

# Load features
features = json.load(open('planning/mods/redpower_2.features.json'))

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

# Build expected texture mapping
def get_expected_texture(face, ref):
    """Convert feature face ref to expected texture name."""
    if '#' in ref and ref.startswith('eloraam/'):
        sheet, idx = ref.split('#')
        sheet_short = sheet.split('/')[-1].replace('.png', '')
        return f'{sheet_short}_{int(idx):03d}'
    return None

# Face name mapping from feature to model
FACE_MAP = {
    'north': 'north', 'south': 'south', 'east': 'east', 'west': 'west',
    'up': 'top', 'down': 'bottom',
}

# Blocks that need thin plate shapes (height 2)
THIN_PLATES = {'and_gate', 'nand_gate', 'nor_gate', 'or_gate', 'not_gate', 
               'xor_gate', 'xnor_gate', 'pulse_former', 'buffer_gate',
               'backplane', 'logic_gate', 'timer', 'rs_latch', 'state_cell',
               'transparent_latch', 'toggle_latch', 'repeater', 'synchronizer',
               'multiplexer', 'randomizer'}

# Blocks that need thin plate shapes (height 4)
THIN_TUBES = {'pipe', 'pneumatic_tube', 'redstone_tube', 'restrictor_tube', 'magtube',
              'frame', 'frame_tube', 'frame_redstone_tube', 'frame_motor'}

# Blocks that need specific thin shapes
THIN_SHAPES = {
    'wire': [0, 0, 0, 16, 1, 16],
    'wire_red_alloy': [0, 0, 0, 16, 1, 16],
    'wire_insulated': [0, 0, 0, 16, 2, 16],
    'wire_blutricity': [0, 0, 0, 16, 3, 16],
    'wire_bundled': [0, 0, 0, 16, 4, 16],
    'wire_blutricity_insulated': [0, 0, 0, 16, 5, 16],
    'wire_jacketed': [0, 0, 0, 16, 1, 16],
    'wire_jacketed_blutricity': [0, 0, 0, 16, 1, 16],
    'ribbon_cable': [0, 0, 0, 16, 1, 16],
    'cable_jacketed': [0, 0, 0, 16, 1, 16],
}

def build_model(block_id, shape, faces):
    """Build a proper model JSON."""
    # Determine parent
    if len(shape) == 6:
        # Thin shape - use cube with elements
        parent = "minecraft:block/cube"
    else:
        parent = "minecraft:block/cube_all"
    
    # Build textures dict
    textures = {}
    for face, tex_name in faces.items():
        model_face = FACE_MAP.get(face, face)
        if model_face not in textures:
            textures[model_face] = f"redpower_2:block/{tex_name}"
    
    # Build elements for thin shapes
    elements = []
    if len(shape) == 6:
        from_coords = shape[0:3]
        to_coords = shape[3:6]
        faces_def = {}
        for face, tex_name in faces.items():
            model_face = FACE_MAP.get(face, face)
            faces_def[model_face] = {"texture": f"#{model_face}"}
        elements = [{
            "from": from_coords,
            "to": to_coords,
            "faces": faces_def
        }]
        textures["particle"] = list(textures.values())[0] if textures else "redpower_2:block/machine1_000"
    
    model = {
        "parent": parent,
        "textures": textures,
        "elements": elements
    }
    
    return model

# Process each block
fixed = 0
for b in features.get('blocks', []):
    bid = b['id']
    shape = b.get('shape')
    faces = b.get('faces', {})
    
    if not faces:
        continue
    
    # Get expected textures
    expected = {}
    for face, ref in faces.items():
        tex = get_expected_texture(face, ref)
        if tex:
            expected[face] = tex
    
    if not expected:
        continue
    
    # Check existing model
    model_file = models_dir / f'{bid}.json'
    if not model_file.exists():
        # Create new model
        model = build_model(bid, shape, expected)
        with open(model_file, 'w') as f:
            json.dump(model, f, indent=2)
        print(f'Created {bid}')
        fixed += 1
        continue
    
    # Read existing model
    with open(model_file) as f:
        data = json.load(f)
    
    # Update textures
    old_textures = data.get('textures', {})
    needs_update = False
    
    for face, tex_name in expected.items():
        model_face = FACE_MAP.get(face, face)
        new_tex = f'redpower_2:block/{tex_name}'
        
        # Find the right key in textures
        found = False
        for k in list(old_textures.keys()):
            if k.lower() == model_face.lower() or (face == 'side' and k in ['north', 'south', 'east', 'west']):
                if old_textures[k] != new_tex:
                    old = old_textures[k]
                    data['textures'][k] = new_tex
                    print(f'{bid}.{face}: {old} -> {new_tex}')
                    needs_update = True
                    found = True
                    break
        
        if not found and model_face in old_textures:
            if old_textures[model_face] != new_tex:
                old = old_textures[model_face]
                data['textures'][model_face] = new_tex
                print(f'{bid}.{face}: {old} -> {new_tex}')
                needs_update = True
    
    # Fix shape for thin blocks
    if shape and len(shape) == 6 and shape != [0, 0, 0, 16, 16, 16]:
        # Check if it's a thin shape that should have elements
        has_elements = len(data.get('elements', [])) > 0
        if not has_elements and data.get('parent') == 'minecraft:block/cube_all':
            print(f'{bid}: Adding thin shape elements')
            from_coords = shape[0:3]
            to_coords = shape[3:6]
            
            faces_def = {}
            for face, tex_name in expected.items():
                model_face = FACE_MAP.get(face, face)
                faces_def[model_face] = {"texture": f"#{model_face}"}
            
            data['parent'] = 'minecraft:block/cube'
            data['elements'] = [{
                "from": from_coords,
                "to": to_coords,
                "faces": faces_def
            }]
            needs_update = True
    
    if needs_update:
        data['textures'] = data.get('textures', {})
        with open(model_file, 'w') as f:
            json.dump(data, f, indent=2)
        fixed += 1

print(f'\nFixed {fixed} models')
