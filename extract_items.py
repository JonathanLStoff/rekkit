from PIL import Image
from pathlib import Path

# Load the sprite sheet
sprite_sheet = Image.open('mods/redpower_2/content/assets/redpower_2/textures/eloraam/base/items1.png')
w, h = sprite_sheet.size
print(f'Sprite sheet: {w}x{h}')

# Item size (16x16 standard for Minecraft items)
ITEM_SIZE = 16

# Items to extract (sprite indices 0-29)
items = [
    'ash', 'bag', 'brick_clay', 'brush', 'emitter', 'empty_cell', 'ender_pouch',
    'energy_cell', 'filled_cell', 'flax_fiber', 'flute', 'glass_box_cutter',
    'gold_nugget', 'indigo_dye', 'jack_hammer', 'knowledge_book', 'logic_processor',
    'mixing_stick', 'paintbrush', 'paper', 'printer', 'programmed_plan', 'receiver',
    'reeder', 'rope_arrow', 'rubber', 'rubber_leaf_blacklist', 'ruby_lens',
    'sapphire_lens', 'wrench'
]

output_dir = Path('mods/redpower_2/content/assets/redpower_2/textures/item')
output_dir.mkdir(parents=True, exist_ok=True)

for i, name in enumerate(items):
    # Calculate sprite position (row-major order)
    col = i % (w // ITEM_SIZE)
    row = i // (w // ITEM_SIZE)
    
    x = col * ITEM_SIZE
    y = row * ITEM_SIZE
    
    # Extract the sprite
    sprite = sprite_sheet.crop((x, y, x + ITEM_SIZE, y + ITEM_SIZE))
    
    # Save as PNG
    output_path = output_dir / f'{name}.png'
    sprite.save(output_path, 'PNG')
    print(f'Extracted {name}: [{x},{y}] -> {output_path}')

print(f'\nExtracted {len(items)} item textures')
