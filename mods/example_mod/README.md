# Example Mod (`example_mod`)

Reference Rust mod template demonstrating Rekkit's **Dual-Artifact Architecture** for porting classic Java Minecraft mods (Tekkit Classic 1.2.5, Direwolf20 1.4.7) to the **Horizon** engine (`horizon/`).

## Architecture Overview

Horizon separates mods into two distinct formats (the legacy WebAssembly runtime has been removed):
1. **Content Mod (`.hmod`, API 0.2)**: Pure data and assets located in [`content/`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/content) (`horizon-mod.toml` with `capabilities = ["registries"]`, `data/`, `assets/`). No `Cargo.toml` is placed in `content/` because `cargo horizon-mod build` rejects directories containing code manifests.
2. **Native Mod (`.hnmod`, `NATIVE_HOOKS_API = 40`)**: Source-level mixin bundle located in [`native/`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/mods/example_mod/native) (`horizon-native.json` and `mixins/example_mod/horizon-mixin.toml` + `hooks.rs`) hooking into Horizon's stable native hook points (`crates/horizon-client/src/game/native_hooks.rs`).
3. **Pure Rust Domain Crate (`Cargo.toml` + `src/lib.rs`)**: Headless, 100% unit-testable state machines, math formulas, inventory logic, and `HorizonNativeMod` trait implementations.

## Directory Structure

```text
mods/example_mod/
├── Cargo.toml                                  # Rust domain crate manifest
├── horizon-registry.toml                       # Horizon Mod Registry page metadata
├── README.md                                   # Mod overview
├── src/
│   └── lib.rs                                  # Domain logic & HorizonNativeMod (API 40) implementation
├── tests/
│   └── mod_tests.rs                            # Unit, native hook lifecycle & manifest validation tests
├── content/                                    # Content Mod source -> builds example_mod.hmod
│   ├── horizon-mod.toml                        # Content manifest (api = "0.2", capabilities = ["registries"])
│   ├── assets/
│   │   └── example_mod/
│   │       ├── blockstates/ender_chest.json
│   │       ├── items/{ender_chest,ender_pouch}.json
│   │       ├── lang/en_us.json
│   │       └── models/block/ender_chest.json
│   └── data/
│       ├── example_mod/
│       │   ├── horizon/
│       │   │   ├── block/ender_chest.json      # Declares example_mod:ender_chest (with item components)
│       │   │   ├── item/ender_pouch.json       # Declares example_mod:ender_pouch (with crafting_remainder)
│       │   │   └── mob_effect/ender_lock.json  # Declares example_mod:ender_lock status effect
│       │   ├── loot_table/blocks/ender_chest.json
│       │   └── recipe/{ender_chest,ender_pouch,ender_tune}.json
│       └── minecraft/
│           └── tags/block/mineable/pickaxe.json
└── native/                                     # Native Mod source -> builds example_mod.hnmod
    ├── horizon-native.json                     # Pinned Horizon/mixins revisions & hooks = 40
    └── mixins/
        └── example_mod/
            ├── horizon-mixin.toml              # horizon-mixins bundle config
            └── hooks.rs                        # Native hook entrypoints
```

## Running Tests & Packaging

```bash
# Run unit and native hook integration tests
cargo test --manifest-path mods/example_mod/Cargo.toml

# Validate and package both .hmod and .hnmod artifacts
make check-mods
make pack-mods
```
