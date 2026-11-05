"""Extract all sprites from the original ZIP files using the correct formula."""
from scripts.art import read_png, write_png
from pathlib import Path
import zipfile, tempfile

base = Path('mods/redpower_2/content/assets/redpower_2/textures/block')

# All sprite references from features.json, grouped by sheet
sprite_refs = {
    'eloraam/base/base1.png': [16, 17, 19, 32, 33, 34, 35],
    'eloraam/compat/compat1.png': [0],
    'eloraam/control/control1.png': [0, 1, 2, 19, 20, 21, 22, 23, 24, 28, 29],
    'eloraam/lighting/lighting1.png': [0, 16, 32, 48],
    'eloraam/logic/array1.png': [0, 1, 2],
    'eloraam/logic/logic1.png': [0, 3, 5, 16, 25, 48, 128, 132, 140, 144, 160, 192, 208, 224],
    'eloraam/logic/logic2.png': [24, 88, 128, 144, 184, 237, 240],
    'eloraam/logic/sensor1.png': [1],
    'eloraam/machine/machine1.png': [2, 21, 22, 23, 24, 25, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 61, 80, 81, 83, 84, 87, 88, 89, 90, 91, 93, 96, 102, 103, 104, 106, 112, 113, 117, 120, 124, 125, 128, 138, 139, 140, 150, 151, 155, 156, 157, 158, 160, 161, 163, 164, 166, 167, 168, 170, 171, 172, 176, 178, 180, 181, 197, 198, 199, 201, 208, 209, 210, 225, 226, 227, 228, 230, 231],
    'eloraam/wiring/redpower1.png': [2, 3, 5, 6, 7, 16, 97, 98, 99],
    'eloraam/world/world1.png': [0, 1, 2, 3, 4, 5, 6, 7, 16, 17, 18, 19, 20, 21, 22, 50, 51, 52, 53, 54, 64, 80, 81, 82, 83, 84, 85],
}

# Map of which zip file contains each sheet
sheet_to_zip = {
    'eloraam/base/base1.png': 'assets/direwolf20/mods/redpowercore.zip',
    'eloraam/compat/compat1.png': 'assets/direwolf20/mods/redpowercompat.zip',
    'eloraam/control/control1.png': 'assets/direwolf20/mods/redpowermechanical.zip',
    'eloraam/lighting/lighting1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
    'eloraam/logic/array1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
    'eloraam/logic/logic1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
    'eloraam/logic/logic2.png': 'assets/direwolf20/mods/redpowerdigital.zip',
    'eloraam/logic/sensor1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
    'eloraam/machine/machine1.png': 'assets/direwolf20/mods/redpowermechanical.zip',
    'eloraam/wiring/redpower1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
    'eloraam/world/world1.png': 'assets/direwolf20/mods/redpowermechanical.zip',
}

def extract_sheet(zip_path, sheet_name, indices):
    with zipfile.ZipFile(zip_path) as z:
        data = z.read(sheet_name)
    tmp = Path(tempfile.mktemp(suffix='.png'))
    tmp.write_bytes(data)
    w, h, px = read_png(tmp)
    tmp.unlink()
    
    size = w // 16
    for idx in indices:
        # Use the SAME formula as contract_check.py's sprite() function
        col = idx % 16
        row = idx // 16
        # Formula: [(row * size + y) * width + col * size + x]
        sprite = [px[(row * size + y) * w + col * size + x] for y in range(size) for x in range(size)]
        
        # Generate output filename based on sheet and index
        sheet_short = sheet_name.split('/')[-1].replace('.png', '')
        out_name = f'{sheet_short}_{idx:03d}.png'
        write_png(base / out_name, 16, 16, sprite)
        print(f'  {sheet_name}#{idx} -> {out_name}')

print('Extracting sprites...')
for sheet, indices in sprite_refs.items():
    zip_path = sheet_to_zip[sheet]
    print(f'\n{sheet}:')
    extract_sheet(zip_path, sheet, indices)

print('\nDone!')
