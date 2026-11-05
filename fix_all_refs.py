"""Fix all models that reference #all but don't define 'all' texture."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

fixed = 0
for model_file in models_dir.glob('*.json'):
    with open(model_file) as f:
        data = json.load(f)
    
    textures = data.get('textures', {})
    
    # Check if any element references #all
    has_all_ref = False
    for elem in data.get('elements', []):
        for face_data in elem.get('faces', {}).values():
            if isinstance(face_data, dict) and '#all' in face_data.get('texture', ''):
                has_all_ref = True
                break
        if has_all_ref:
            break
    
    if has_all_ref and 'all' not in textures:
        # Add 'all' using first available texture
        if textures:
            first_key = next(k for k in textures if k != 'particle')
            if first_key:
                textures['all'] = textures[first_key]
                data['textures'] = textures
                with open(model_file, 'w') as f:
                    json.dump(data, f, indent=2)
                print(f'{model_file.name}: Added all={first_key}')
                fixed += 1

print(f'\nFixed {fixed} models')
