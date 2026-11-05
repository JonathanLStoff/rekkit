from pathlib import Path
import json

# Update contract files with proper fields
contract_dir = Path('planning/mods/redpower_2.contract')

missing_items = [
    'ash', 'bag', 'brick_clay', 'brush', 'emitter', 'empty_cell', 'ender_pouch',
    'energy_cell', 'filled_cell', 'flax_fiber', 'flute', 'glass_box_cutter',
    'gold_nugget', 'indigo_dye', 'jack_hammer', 'knowledge_book', 'logic_processor',
    'mixing_stick', 'paintbrush', 'paper', 'printer', 'programmed_plan', 'receiver',
    'reeder', 'rope_arrow', 'rubber', 'rubber_leaf_blacklist', 'ruby_lens',
    'sapphire_lens', 'wrench'
]

for name in missing_items:
    f = contract_dir / f'{name}.json'
    if f.exists():
        data = json.load(open(f))
        # Add required fields
        data['sprite'] = f'eloraam/{name}/{name}'  # placeholder sprite reference
        data['max_stack'] = 64
        data['type'] = 'item'
        with open(f, 'w') as out:
            json.dump(data, out, indent=2)
        print(f'Updated: {name}')

print(f'\nUpdated {len(missing_items)} contracts')
