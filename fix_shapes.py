"""Fix shape issues for cross-shaped blocks and full cubes."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')

# Cross shape definition
CROSS_SHAPE = [[0, 0, 0], [16, 0, 16], [0, 4, 0], [16, 4, 16], [8, 4, 8], [0, 8, 0], [16, 8, 16], [8, 8, 8], [0, 12, 0], [16, 12, 16]]

def fix_shape(bid, shape):
    model_file = models_dir / f'{bid}.json'
    if not model_file.exists():
        return
    
    with open(model_file) as f:
        data = json.load(f)
    
    if shape == 'cross':
        # Replace with proper cross shape
        data['elements'] = [{
            "from": CROSS_SHAPE[0],
            "to": CROSS_SHAPE[-1],
            "faces": {k: {"texture": f"#{k}"} for k in ['north', 'south', 'east', 'west', 'up', 'down']}
        }]
        data['parent'] = 'minecraft:block/cube'
    elif shape == 'full' or (len(shape) == 6 and all(c == 16 for c in shape)):
        # Full cube - keep as cube_all
        pass
    else:
        print(f'{bid}: Unknown shape {shape}, keeping current model')
        return
    
    with open(model_file, 'w') as f:
        json.dump(data, f, indent=2)
    print(f'Fixed shape for {bid}')

# Fix known cross-shaped blocks
for bid in ['flax_crop', 'indigo_flower', 'rubber_sapling']:
    fix_shape(bid, 'cross')

# Fix ram_module - should be full cube
import json as j
features = j.load(open('planning/mods/redpower_2.features.json'))
for b in features.get('blocks', []):
    if b['id'] == 'ram_module':
        shape = b.get('shape')
        if len(shape) == 6 and all(c == 16 for c in shape):
            fix_shape('ram_module', shape)
        break

print('\nDone!')