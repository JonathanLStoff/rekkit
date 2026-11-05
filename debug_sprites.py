"""Debug sprite index mismatches."""
import sys
sys.path.insert(0, 'scripts')
from contract_check import Mod
from pathlib import Path

mod = Mod('redpower_2')

# Check a few examples
examples = [
    ('alloy_furnace', 'down', 'base1_019'),
    ('assembler', 'south', 'machine1_168'),
]

for block, face, tex_name in examples:
    # Get expected sprite from features
    import json
    features = json.load(open('planning/mods/redpower_2.features.json'))
    for b in features.get('blocks', []):
        if b['id'] == block:
            faces = b.get('faces', {})
            # Find the face
            face_key = None
            for k in ['down', 'south', 'north', 'east', 'west', 'up', 'top', 'bottom', 'front', 'back', 'side']:
                if k in faces:
                    face_key = k
                    break
            
            if face_key:
                ref = faces[face_key]
                print(f'{block}.{face_key}: {ref}')
                
                # Extract what sprite the checker expects
                sheet, idx = ref.split('#')
                expected_sprite = int(idx)
                
                # Check actual sprite in PNG
                tex = mod.texture_pixels(f'redpower_2:block/{tex_name}')
                ref2 = {'sheet': sheet.replace('.png', ''), 'index': expected_sprite}
                # Normalize
                sheet_norm = sheet.split('/')[-1].replace('.png', '')
                ref2 = {'sheet': f'eloraam/{sheet_norm}', 'index': expected_sprite}
                
                try:
                    expected = mod.sheets.sprite(ref2)
                    print(f'  Expected sprite {expected_sprite}')
                    print(f'  Match: {expected == tex}')
                except Exception as e:
                    print(f'  Error: {e}')
            break
