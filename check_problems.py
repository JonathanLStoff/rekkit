import json
from pathlib import Path

features = json.load(open('planning/mods/redpower_2.features.json'))

names = ['brass_gear', 'cable_jacketed', 'ribbon_cable', 'wire', 'wire_blutricity_insulated',
         'wire_bundled', 'wire_insulated', 'wire_jacketed', 'wire_jacketed_blutricity', 'wire_red_alloy']

for name in names:
    found = False
    for b in features.get('blocks', []):
        if b['id'] == name:
            print(f'BLOCK {name}:')
            print(f'  attrs: {b.get("attributes")}')
            print(f'  gui: {b.get("gui")}')
            print(f'  energy: {b.get("energy")}')
            found = True
            break
    if not found:
        for i in features.get('items', []):
            if i['id'] == name:
                print(f'ITEM {name}:')
                print(f'  attrs: {i.get("attributes")}')
                print(f'  use: {i.get("use")}')
                print(f'  durability: {i.get("durability")}')
                found = True
                break
    if not found:
        print(f'{name}: NOT FOUND')
    print()
