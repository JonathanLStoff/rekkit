"""Add proper facing variants to all horizontal-facing blockstate files."""
import json
import os
from pathlib import Path
from collections import OrderedDict

BLOCKSTATES = Path('mods/redpower_2/content/assets/redpower_2/blockstates')
HORIZONTAL_BLOCKS = [
    'accelerator', 'and_gate', 'assembler', 'backplane', 'battery_box',
    'blulectric_alloy_furnace', 'blulectric_furnace', 'buffer', 'buffer_chest',
    'buffer_gate', 'bus_transceiver', 'cage_lamp', 'cage_lamp_inverted',
    'cpu', 'display', 'disk_drive', 'io_expander', 'keyboard',
    'light_sensor', 'manager', 'multiplexer', 'monitor', 'project_table',
    'pulse_former', 'ram_module', 'solar_panel', 'wind_turbine',
    'wood_windmill', 'transformer', 'comparator', 'repeater', 'sorting_machine',
    'sortron', 'state_cell', 'synchronizer', 'timer', 'toggle_latch',
    'transparent_latch', 'frame', 'frame_tube', 'frame_motor',
    'redstone_tube', 'pneumatic_tube', 'pipe', 'magtube',
]

for name in HORIZONTAL_BLOCKS:
    path = BLOCKSTATES / f'{name}.json'
    if not path.exists():
        continue
    with open(path) as f:
        data = json.load(f)
    
    variants = data.get('variants', {})
    if '' in variants:
        # Remove default variant, add facing variants
        default = variants.pop('')
    
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
