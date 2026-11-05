"""Fix ram_module to be a full cube."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')
texture_dir = Path('mods/redpower_2/content/assets/redpower_2/textures/block')

path = models_dir / 'ram_module.json'
with open(path) as f:
    data = json.load(f)

# Change to full cube
data['parent'] = 'minecraft:block/cube_all'
# Keep existing textures but ensure 'all' exists
textures = data.get('textures', {})
if textures:
    first_key = next(k for k in textures if k != 'particle')
    textures['all'] = textures[first_key]
    data['textures'] = textures

with open(path, 'w') as f:
    json.dump(data, f, indent=2)

print('Fixed ram_module to full cube')
