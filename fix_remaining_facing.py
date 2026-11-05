"""Fix remaining facing blockstates."""
import json
from pathlib import Path

BLOCKSTATES = Path('mods/redpower_2/content/assets/redpower_2/blockstates')

# These 3 blocks need horizontal facing
HORIZONTAL_BLOCKS = ['pump', 'regulator', 'retriever']

for name in HORIZONTAL_BLOCKS:
    path = BLOCKSTATES / f'{name}.json'
    if not path.exists():
        continue
    with open(path) as f:
        data = json.load(f)
    
    variants = data.get('variants', {})
    
    # Build facing variants for each direction
    new_variants = {}
    for side in ['north', 'south', 'west', 'east']:
        rot = {'north': None, 'south': 180, 'west': 270, 'east': 90}[side]
        entry = {'model': f'redpower_2:block/{name}'}
        if rot is not None:
            entry['y'] = rot
        new_variants[f'facing={side}'] = entry
    
    data['variants'] = new_variants
    with open(path, 'w') as f:
        json.dump(data, f, indent=2)
    print(f'Updated {name}')

print('\nDone!')
