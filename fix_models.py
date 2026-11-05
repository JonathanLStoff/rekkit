from pathlib import Path
import json

# Update item models to reference textures
models_dir = Path('mods/redpower_2/content/assets/redpower_2/models/item')

missing_items = [
    'ash', 'bag', 'brick_clay', 'brush', 'emitter', 'empty_cell', 'ender_pouch',
    'energy_cell', 'filled_cell', 'flax_fiber', 'flute', 'glass_box_cutter',
    'gold_nugget', 'indigo_dye', 'jack_hammer', 'knowledge_book', 'logic_processor',
    'mixing_stick', 'paintbrush', 'paper', 'printer', 'programmed_plan', 'receiver',
    'reeder', 'rope_arrow', 'rubber', 'rubber_leaf_blacklist', 'ruby_lens',
    'sapphire_lens', 'wrench'
]

for name in missing_items:
    f = models_dir / f'{name}.json'
    if f.exists():
        data = {'parent': 'minecraft:item/generated', 'textures': {'layer0': f'redpower_2:item/{name}'}}
        with open(f, 'w') as out:
            json.dump(data, out, indent=2)
        print(f'Updated: {name}')

print(f'\nUpdated {len(missing_items)} item models')
