from pathlib import Path
import json

# Add missing items to features.json
features = Path('planning/mods/redpower_2.features.json')
data = json.load(open(features))

missing_items = [
    'ash', 'bag', 'brick_clay', 'brush', 'emitter', 'empty_cell', 'ender_pouch',
    'energy_cell', 'filled_cell', 'flax_fiber', 'flute', 'glass_box_cutter',
    'gold_nugget', 'indigo_dye', 'jack_hammer', 'knowledge_book', 'logic_processor',
    'mixing_stick', 'paintbrush', 'paper', 'printer', 'programmed_plan', 'receiver',
    'reeder', 'rope_arrow', 'rubber', 'rubber_leaf_blacklist', 'ruby_lens',
    'sapphire_lens', 'wrench'
]

existing_ids = {i.get('id') for i in data.get('items', [])}
new_items = []
for name in missing_items:
    if name not in existing_ids:
        new_items.append({'id': name, 'type': 'item', 'attributes': []})

if new_items:
    data['items'].extend(new_items)
    with open(features, 'w') as f:
        json.dump(data, f, indent=2)
    print(f'Added {len(new_items)} items to features.json')
    print(f'Total items: {len(data["items"])}')
