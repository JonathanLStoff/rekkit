"""Fix models with cube_all parent that lack 'all' texture."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

fixed = 0
for model_file in models_dir.glob('*.json'):
    with open(model_file) as f:
        data = json.load(f)
    
    # Only fix cube_all models
    if data.get('parent') != 'minecraft:block/cube_all':
        continue
    
    textures = data.get('textures', {})
    
    # If model has individual face textures but no 'all', add 'all'
    if 'all' not in textures and len(textures) > 1:
        # Use the first available texture as 'all'
        first_key = next(k for k in textures if k != 'particle')
        if first_key:
            textures['all'] = textures[first_key]
            data['textures'] = textures
            with open(model_file, 'w') as f:
                json.dump(data, f, indent=2)
            print(f'{model_file.name}: Added all={first_key}')
            fixed += 1

print(f'\nFixed {fixed} models')
