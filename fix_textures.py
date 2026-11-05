"""Fix all model texture references to match existing PNG filenames."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')
base = Path('mods/redpower_2/content/assets/redpower_2/textures/block')

# Get all available textures
all_textures = set()
for f in base.glob('*.png'):
    name = f.stem  # e.g., 'logic_gates_005'
    all_textures.add(name)
    # Also add without suffixes like _gates, _cells, etc.
    short = name.split('_')[0] if '_' in name else name
    if short not in all_textures:
        all_textures.add(short)

# Fix texture references
updated_files = 0
for model_file in models_dir.glob('*.json'):
    with open(model_file) as f:
        data = json.load(f)
    
    textures = data.get('textures', {})
    fixed = False
    
    for key, tex in list(textures.items()):
        if not isinstance(tex, str) or not tex.startswith('redpower_2:block/'):
            continue
        
        tex_name = tex.replace('redpower_2:block/', '')
        tex_prefix = tex_name.split('_')[0]  # 'cpu_000' -> 'cpu'
        
        # Try to find a matching file
        matches = []
        for fname in all_textures:
            fprefix = fname.split('_')[0] if '_' in fname else fname
            if fprefix == tex_prefix and len(fname.split('_')) <= len(tex_name.split('_')):
                matches.append(fname)
        
        if matches:
            # Use the full name that matches the pattern
            # Prefer the one with the most underscores to match better
            best = max(matches, key=lambda m: m.count('_'))
            textures[key] = f'redpower_2:block/{best}'
            fixed = True
    
    if fixed:
        data['textures'] = textures
        with open(model_file, 'w') as f:
            json.dump(data, f, indent=2)
        print(f'{model_file.name}: fixed')
        updated_files += 1

print(f'\nUpdated {updated_files} model files')
