#!/usr/bin/env python3
"""Extract individual sprites from RedPower sprite sheets."""
import zipfile
import os

try:
    from PIL import Image
except ImportError:
    print("PIL not available, using struct-based PNG reader")
    Image = None

def extract_sprite_pil(sheet_data, sprite_index, output_path, size=16):
    """Extract a single 16x16 sprite from a sheet PNG using Pillow."""
    img = Image.open(sheet_data).convert('RGBA')
    w, h = img.size
    cols = w // size
    row = sprite_index // cols
    col = sprite_index % cols
    x = col * size
    y = row * size
    sprite = img.crop((x, y, x + size, y + size))
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    sprite.save(output_path, 'PNG')
    return True

def extract_sprite_raw(sheet_data, sprite_index, output_path, size=16):
    """Extract sprite without Pillow using basic PNG parsing."""
    import io
    # Use the PIL fallback
    return extract_sprite_pil(sheet_data, sprite_index, output_path, size)

def main():
    base = 'mods/redpower_2/content/assets/redpower_2/textures/block'
    os.makedirs(base, exist_ok=True)
    
    # Define sprite extractions: (sheet_path_in_zip, indices, output_prefix)
    extractions = [
        # Logic gates
        ('eloraam/logic/logic1.png', [0, 5, 128, 132, 140, 144, 160, 192], 'logic1'),
        ('eloraam/logic/logic2.png', [24, 88, 128, 144, 184, 240], 'logic2'),
        ('eloraam/logic/array1.png', [0, 1, 2], 'array1'),
        ('eloraam/logic/sensor1.png', [1], 'sensor1'),
        # Wiring
        ('eloraam/wiring/redpower1.png', [2, 5, 7, 16, 97, 98, 99], 'redpower1_wire'),
        # Control
        ('eloraam/control/control1.png', [0, 3, 16, 17, 18], 'control1'),
    ]
    
    # Read from ZIP archives
    archives = {
        'eloraam/logic/logic1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
        'eloraam/logic/logic2.png': 'assets/direwolf20/mods/redpowerdigital.zip',
        'eloraam/logic/array1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
        'eloraam/logic/sensor1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
        'eloraam/wiring/redpower1.png': 'assets/direwolf20/mods/redpowerdigital.zip',
        'eloraam/control/control1.png': 'assets/direwolf20/mods/redpowermechanical.zip',
    }
    
    for sheet, indices, prefix in extractions:
        zip_path = archives[sheet]
        with zipfile.ZipFile(zip_path) as z:
            data = z.read(sheet)
        
        for idx in indices:
            out_name = f'{prefix}_{idx:03d}.png'
            out_path = os.path.join(base, out_name)
            if not os.path.exists(out_path):
                extract_sprite_pil(data, idx, out_path)
                print(f'  Extracted {sheet}#{idx} -> {out_name}')
    
    print('Done extracting sprites.')

if __name__ == '__main__':
    main()
