"""Fix wind_turbine and similar models by removing 'all' when specific faces exist."""
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
    
    # Remove 'all' if it conflicts with specific face textures
    face_keys = {'north', 'south', 'east', 'west', 'up', 'down'}
    has_face_textures = any(k in textures for k in face_keys)
    
    if has_face_textures and 'all' in textures:
        del textures['all']
        data['textures'] = textures
        with open(model_file, 'w') as f:
            json.dump(data, f, indent=2)
        print(f'{model_file.name}: Removed all')
        fixed += 1

print(f'\nFixed {fixed} models')
