"""Fix all model texture references to match features.json expectations."""
import json
from pathlib import Path

# Load features
features = json.load(open('planning/mods/redpower_2.features.json'))

# Build expected texture mapping from features
expected = {}
for b in features.get('blocks', []):
    bid = b['id']
    faces = b.get('faces', {})
    if not faces:
        continue
    
    tex_map = {}
    for face, ref in faces.items():
        if '#' in ref and ref.startswith('eloraam/'):
            sheet, idx = ref.split('#')
            sheet_short = sheet.split('/')[-1].replace('.png', '')
            tex_map[face] = f'{sheet_short}_{int(idx):03d}'
    
    if tex_map:
        expected[bid] = tex_map

# Also handle items
for i in features.get('items', []):
    iid = i['id']
    faces = i.get('faces', {})
    if not faces:
        continue
    
    tex_map = {}
    for face, ref in faces.items():
        if '#' in ref and ref.startswith('eloraam/'):
            sheet, idx = ref.split('#')
            sheet_short = sheet.split('/')[-1].replace('.png', '')
            tex_map[face] = f'{sheet_short}_{int(idx):03d}'
    
    if tex_map:
        expected[iid] = tex_map

# Fix models
models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')
updated = 0

for bid, exp_tex in expected.items():
    model = models_dir / f'{bid}.json'
    if not model.exists():
        continue
    
    with open(model) as f:
        data = json.load(f)
    
    textures = data.get('textures', {})
    fixed = False
    
    for face, exp_name in exp_tex.items():
        # Find the corresponding model texture key
        # The face names in features might differ from model texture keys
        # Common mappings:
        face_map = {
            'north': 'north', 'south': 'south', 'east': 'east', 'west': 'west',
            'up': 'top', 'down': 'bottom', 'side': None, 'front': None, 'back': None,
            'all': None
        }
        
        model_key = face_map.get(face)
        if model_key is None:
            # Try to find by matching face name in textures
            for k in textures:
                if k.lower() == face.lower():
                    model_key = k
                    break
        
        if model_key and model_key in textures:
            old_tex = textures[model_key]
            new_tex = f'redpower_2:block/{exp_name}'
            if old_tex != new_tex:
                print(f'{bid}.{face}: {old_tex} -> {new_tex}')
                textures[model_key] = new_tex
                fixed = True
    
    if fixed:
        data['textures'] = textures
        with open(model, 'w') as f:
            json.dump(data, f, indent=2)
        updated += 1

print(f'\nUpdated {updated} model files')
