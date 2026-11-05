"""Check and fix sprite index mismatches in features.json."""
import json
from pathlib import Path
import sys
sys.path.insert(0, 'scripts')
from contract_check import Mod

mod = Mod('redpower_2')

# Load features
features = json.load(open('planning/mods/redpower_2.features.json'))

# Get all texture issues from contract checker
import subprocess
result = subprocess.run(['python', 'scripts/contract_check.py', '--mod', 'redpower_2'], capture_output=True, text=True)
lines = result.stdout.splitlines()
issues = [l for l in lines if 'is not sprite' in l and 'contract problem' not in l]

# Parse issues to find swaps
swaps = {}
for l in issues:
    # Extract block and face
    parts = l.split(':')
    block = parts[0].replace('redpower_2:', '')
    face = parts[1].strip().split()[1]  # "face X:"
    
    # Extract expected sprite
    if 'is not sprite' in l:
        import re
        match = re.search(r'is not sprite (\d+) of (\S+)', l)
        if match:
            expected_idx = int(match.group(1))
            expected_sheet = match.group(2)
            
            # Extract actual sprite from file name
            match2 = re.search(r'block/(\w+_\d+)', l)
            if match2:
                actual_name = match2.group(1)
                actual_idx = int(actual_name.split('_')[-1])
                
                key = (block, face)
                swaps[key] = {
                    'expected': expected_idx,
                    'actual': actual_idx,
                    'sheet': expected_sheet,
                    'file': actual_name
                }

print(f'Found {len(swaps)} swaps')

# Show some examples
for k, v in list(swaps.items())[:10]:
    print(f'{k}: expected sprite {v["expected"]}, got {v["file"]} (sprite {v["actual"]})')
