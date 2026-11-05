#!/usr/bin/env python3
"""Add shapes to block JSONs based on contract definitions."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_DIR = ROOT / "planning" / "mods" / "redpower_2.contract"
BLOCK_DIR = ROOT / "mods" / "redpower_2" / "content" / "data" / "redpower_2" / "horizon" / "block"


def pixel_to_normalized(box):
    """Convert pixel coords [0-16] to normalized [0-1]."""
    return [x / 16.0 for x in box]


def main():
    count = 0
    for contract_file in sorted(CONTRACT_DIR.glob("*.json")):
        block_name = contract_file.stem
        contract = json.load(open(contract_file))

        shape = contract.get("shape")
        if not shape or shape == "full":
            continue

        # Normalize the shape
        if isinstance(shape, list) and len(shape) == 6:
            normalized = pixel_to_normalized(shape)
            block_file = BLOCK_DIR / f"{block_name}.json"
            if not block_file.exists():
                print(f"WARNING: Block file not found: {block_file}")
                continue

            block_data = json.load(open(block_file))

            # Add shapes if not present
            if "shapes" not in block_data:
                block_data["shapes"] = {"default": [normalized]}
                with open(block_file, "w") as f:
                    json.dump(block_data, f, indent=2)
                count += 1
                print(f"Added shapes to {block_name}: {normalized}")
            else:
                print(f"SKIP {block_name}: already has shapes")
        else:
            print(f"WARNING: Unknown shape format for {block_name}: {shape}")

    print(f"\nTotal blocks updated: {count}")


if __name__ == "__main__":
    main()
