"""Fix model texture references and add missing texture keys."""
import json
from pathlib import Path

# Load features
features = json.load(open('planning/mods/redpower_2.features.json'))

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')
texture_dir = Path('mods/redpower_2/content/assets/redpower_2/textures/block')

# Build expected texture mapping from features
FACE_MAP = {
    'north': 'north', 'south': 'south', 'east': 'east', 'west': 'west',
    'up': 'top', 'down': 'bottom', 'side': None, 'front': None, 'back': None,
    'all': None, 'particle': None
}

def get_expected(face, ref):
    if '#' in ref and ref.startswith('eloraam/'):
        sheet, idx = ref.split('#')
        sheet_short = sheet.split('/')[-1].replace('.png', '')
        return f'{sheet_short}_{int(idx):03d}'
    return None

fixed = 0
for b in features.get('blocks', []):
    bid = b['id']
    faces = b.get('faces', {})
    if not faces:
        continue
    
    # Build expected textures
    expected = {}
    for face, ref in faces.items():
        tex = get_expected(face, ref)
        if tex:
            expected[face] = tex
    
    if not expected:
        continue
    
    model_file = models_dir / f'{bid}.json'
    if not model_file.exists():
        continue
    
    with open(model_file) as f:
        data = json.load(f)
    
    textures = data.get('textures', {})
    needs_update = False
    
    # Add missing face textures
    for face, exp_name in expected.items():
        model_key = FACE_MAP.get(face, face)
        new_tex = f'redpower_2:block/{exp_name}'
        
        if new_tex not in textures.values() and model_key in textures:
            old = textures[model_key]
            if old != new_tex:
                print(f'{bid}.{face}: {old} -> {new_tex}')
                textures[model_key] = new_tex
                needs_update = True
        
        elif model_key not in textures and new_tex not in textures.values():
            print(f'{bid}.{face}: ADDING {new_tex}')
            textures[model_key] = new_tex
            needs_update = True
    
    # Fix texture name for 'all' face - it should map to a side texture
    if 'all' in expected:
        side_texs = [v for v in textures.values() if v.startswith('redpower_2:block/')]
        all_tex = f"redpower_2:block/{expected['all']}"
        if all_tex not in textures.values():
            for k, v in list(textures.items()):
                if v == f"redpower_2:block/{expected.get('side', '')}":
                    textures['side'] = all_tex
                    break
                elif v.startswith('redpower_2:block/'):
                    textures['side'] = all_tex
                    break
    
    if needs_update:
        data['textures'] = textures
        with open(model_file, 'w') as f:
            json.dump(data, f, indent=2)
        fixed += 1

print(f'\nFixed {fixed} models')
