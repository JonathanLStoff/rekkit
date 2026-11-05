"""Extract missing sprites from control1.png."""
import zipfile
import struct
import zlib
from pathlib import Path

texture_dir = Path('mods/redpower_2/content/assets/redpower_2/textures/block')

# Read PNG bytes
with zipfile.ZipFile('assets/direwolf20/mods/redpowermechanical.zip') as z:
    png_data = z.read('eloraam/control/control1.png')

# Parse PNG
class Blob:
    def __init__(self, data):
        self._data = data
        self.pos = 0
    def read(self, n):
        result = self._data[self.pos:self.pos+n]
        self.pos += n
        return result
    def read_bytes(self):
        return self._data

# Find PNG dimensions
pos = 8
w = struct.unpack('>I', png_data[pos:pos+4])[0]
h = struct.unpack('>I', png_data[pos+4:pos+8])[0]
size = w // 16  # sprite size (16x16 or 32x32)
print(f'control1.png: {w}x{h}, sprite size: {size}x{size}')

# Decompress IDAT chunks
idat_data = b''
pos = 8
while pos < len(png_data):
    length = struct.unpack('>I', png_data[pos:pos+4])[0]
    ctype = png_data[pos+4:pos+8].decode('ascii', errors='ignore')
    if ctype == 'IDAT':
        idat_data += png_data[pos+8:pos+8+length]
    pos += 8 + length + 4

# Decompress
raw = zlib.decompress(idat_data)

# Convert to pixel format
# First byte is filter type for each row
pixels = []
row_size = w * 4  # RGBA
for y in range(h):
    filter_type = raw[y * (row_size + 1)]
    row_start = y * (row_size + 1) + 1
    row = bytearray(raw[row_start:row_start + row_size])
    
    # Apply filter (simple pass-through for now)
    for x in range(0, len(row), 4):
        r, g, b, a = int(row[x]), int(row[x+1]), int(row[x+2]), int(row[x+3])
        pixels.append((r, g, b, a))

# Extract sprite 18
idx = 18
col = idx % 16
row = idx // 16
sprite_pixels = []
for sy in range(size):
    for sx in range(size):
        px_idx = (row * size + sy) * w + col * size + sx
        if px_idx < len(pixels):
            sprite_pixels.append(pixels[px_idx])

print(f'Sprite 18 has {len(sprite_pixels)} pixels, non-empty: {sum(1 for p in sprite_pixels if p[3] > 0)}')

# Extract sprite 34
idx = 34
col = idx % 16
row = idx // 16
sprite_pixels = []
for sy in range(size):
    for sx in range(size):
        px_idx = (row * size + sy) * w + col * size + sx
        if px_idx < len(pixels):
            sprite_pixels.append(pixels[px_idx])

print(f'Sprite 34 has {len(sprite_pixels)} pixels, non-empty: {sum(1 for p in sprite_pixels if p[3] > 0)}')

# Check which sprites are non-empty
for idx in range(min(40, 16 * 16)):
    col = idx % 16
    row = idx // 16
    sprite_pixels = []
    for sy in range(size):
        for sx in range(size):
            px_idx = (row * size + sy) * w + col * size + sx
            if px_idx < len(pixels):
                sprite_pixels.append(pixels[px_idx])
    non_empty = sum(1 for p in sprite_pixels if p[3] > 0)
    if non_empty > 0:
        print(f'Sprite {idx}: {non_empty} non-empty pixels')
