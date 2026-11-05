"""Fix ram_module to be a full cube."""
import json
from pathlib import Path

models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/block')
features_file = Path('planning/mods/redpower_2.features.json')

# Update features.json
with open(features_file) as f:
    features = json.load(f)

for b in features.get('blocks', []):
    if b['id'] == 'ram_module':
        b['shape'] = [0, 0, 0, 16, 16, 16]
        print('Updated ram_module shape to full cube')
        break

with open(features_file, 'w') as f:
    json.dump(features, f, indent=2)

# Regenerate model
model_file = models_dir / 'ram_module.json'
with open(model_file) as f:
    data = json.load(f)

# Find texture
textures = data.get('textures', {})
if textures:
    first_key = next(k for k in textures if k != 'particle')
    tex = textures[first_key]
else:
    # Find from features
    for b in features.get('blocks', []):
        if b['id'] == 'ram_module':
            faces = b.get('faces', {})
            if faces:
                ref = next(iter(faces.values()))
                if '#' in ref:
                    sheet, idx = ref.split('#')
                    sheet_short = sheet.split('/')[-1].replace('.png', '')
                    tex = f'redpower_2:block/{sheet_short}_{int(idx):03d}'
                    break

# Create full cube model
model = {
    "parent": "minecraft:block/cube_all",
    "textures": {"all": tex}
}

with open(model_file, 'w') as f:
    json.dump(model, f, indent=2)

print('Fixed ram_module model')
