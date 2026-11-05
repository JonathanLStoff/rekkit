"""Revert 'all' texture additions to cube_all models that have specific face textures."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

reverted = 0
for model_file in models_dir.glob('*.json'):
    with open(model_file) as f:
        data = json.load(f)
    
    if data.get('parent') != 'minecraft:block/cube_all':
        continue
    
    textures = data.get('textures', {})
    
    # Only keep 'all' if it's needed (no specific face textures)
    face_keys = {'north', 'south', 'east', 'west', 'up', 'down'}
    has_face_textures = any(k in textures for k in face_keys)
    
    if has_face_textures and 'all' in textures:
        # Check if 'all' matches any face texture - if so, remove it
        all_tex = textures['all']
        face_teachers = [v for k, v in textures.items() if k in face_keys]
        if all_tex in face_teachers:
            del textures['all']
            data['textures'] = textures
            with open(model_file, 'w') as f:
                json.dump(data, f, indent=2)
            print(f'{model_file.name}: Removed all')
            reverted += 1

print(f'\nReverted {reverted} models')
