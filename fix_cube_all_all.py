"""Add 'all' texture to all cube_all models."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

fixed = 0
for model_file in models_dir.glob('*.json'):
    with open(model_file) as f:
        data = json.load(f)
    
    if data.get('parent') != 'minecraft:block/cube_all':
        continue
    
    textures = data.get('textures', {})
    
    if 'all' not in textures and textures:
        # Add 'all' using the first available non-particle texture
        first_key = next((k for k in textures if k != 'particle'), None)
        if first_key:
            textures['all'] = textures[first_key]
            data['textures'] = textures
            with open(model_file, 'w') as f:
                json.dump(data, f, indent=2)
            print(f'{model_file.name}: Added all={first_key}')
            fixed += 1

print(f'\nFixed {fixed} models')
