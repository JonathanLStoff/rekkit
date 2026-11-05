"""Fix cross-shaped plant models."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')
texture_dir = Path('mods/redpower_2/content/assets/redpower_2/textures/block')

# Cross-shaped plants that need minecraft:block/cross parent
CROSS_PLANTS = ['flax_crop', 'indigo_flower', 'rubber_sapling']

for bid in CROSS_PLANTS:
    model_file = models_dir / f'{bid}.json'
    if not model_file.exists():
        print(f'{bid}: model not found')
        continue
    
    # Find the texture
    import json
    features = json.load(open('planning/mods/redpower_2.features.json'))
    tex_name = None
    for b in features.get('blocks', []):
        if b['id'] == bid:
            faces = b.get('faces', {})
            if 'all' in faces:
                ref = faces['all']
                if '#' in ref:
                    sheet, idx = ref.split('#')
                    sheet_short = sheet.split('/')[-1].replace('.png', '')
                    tex_name = f'redpower_2:block/{sheet_short}_{int(idx):03d}'
                    break
    
    if not tex_name:
        # Try to find from existing model
        with open(model_file) as f:
            data = json.load(f)
        textures = data.get('textures', {})
        for k, v in textures.items():
            if k != 'particle':
                tex_name = v
                break
    
    if tex_name:
        # Create cross model
        model = {
            "parent": "minecraft:block/cross",
            "textures": {
                "cross": tex_name
            }
        }
        with open(model_file, 'w') as f:
            json.dump(model, f, indent=2)
        print(f'Fixed {bid} to cross shape')
    else:
        print(f'{bid}: no texture found')

print('\nDone!')
