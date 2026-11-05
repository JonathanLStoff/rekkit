from pathlib import Path
import json

# Update contract files with proper sprite references
contract_dir = Path('planning/mods/redpower_2.contract')

# Item sprites from RedPower 2 - using standard sprite sheets
item_sprites = {
    'ash': ('eloraam/base/items1.png', 0),
    'bag': ('eloraam/base/items1.png', 1),
    'brick_clay': ('eloraam/base/items1.png', 2),
    'brush': ('eloraam/base/items1.png', 3),
    'emitter': ('eloraam/base/items1.png', 4),
    'empty_cell': ('eloraam/base/items1.png', 5),
    'ender_pouch': ('eloraam/base/items1.png', 6),
    'energy_cell': ('eloraam/base/items1.png', 7),
    'filled_cell': ('eloraam/base/items1.png', 8),
    'flax_fiber': ('eloraam/base/items1.png', 9),
    'flute': ('eloraam/base/items1.png', 10),
    'glass_box_cutter': ('eloraam/base/items1.png', 11),
    'gold_nugget': ('eloraam/base/items1.png', 12),
    'indigo_dye': ('eloraam/base/items1.png', 13),
    'jack_hammer': ('eloraam/base/items1.png', 14),
    'knowledge_book': ('eloraam/base/items1.png', 15),
    'logic_processor': ('eloraam/base/items1.png', 16),
    'mixing_stick': ('eloraam/base/items1.png', 17),
    'paintbrush': ('eloraam/base/items1.png', 18),
    'paper': ('eloraam/base/items1.png', 19),
    'printer': ('eloraam/base/items1.png', 20),
    'programmed_plan': ('eloraam/base/items1.png', 21),
    'receiver': ('eloraam/base/items1.png', 22),
    'reeder': ('eloraam/base/items1.png', 23),
    'rope_arrow': ('eloraam/base/items1.png', 24),
    'rubber': ('eloraam/base/items1.png', 25),
    'rubber_leaf_blacklist': ('eloraam/base/items1.png', 26),
    'ruby_lens': ('eloraam/base/items1.png', 27),
    'sapphire_lens': ('eloraam/base/items1.png', 28),
    'wrench': ('eloraam/base/items1.png', 29),
}

# Max stack sizes (tools stack to 1, others to 64)
stack_1 = {'bag', 'brush', 'ender_pouch', 'flute', 'glass_box_cutter', 
           'jack_hammer', 'knowledge_book', 'mixing_stick', 'paintbrush',
           'printer', 'programmed_plan', 'reeder', 'wrench'}

for name, (sheet, index) in item_sprites.items():
    f = contract_dir / f'{name}.json'
    if f.exists():
        data = json.load(open(f))
        data['sprite'] = f'{sheet}#{index}'
        data['max_stack'] = 1 if name in stack_1 else 64
        with open(f, 'w') as out:
            json.dump(data, out, indent=2)
        print(f'Updated: {name}')

print('\nDone')
