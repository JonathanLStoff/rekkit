# Example Mod (`example_mod`)

Reference Rust mod template demonstrating the porting workflow from classic Java Minecraft mods to the **Horizon** engine.

## Directory Structure

```text
mods/example_mod/
├── Cargo.toml                  # Rust package manifest & dependencies
├── horizon-mod.toml            # Horizon mod manifest (API 0.2 / 0.3)
├── README.md                   # Mod overview
├── src/
│   └── lib.rs                  # Core domain logic, state machines, and lifecycle traits
├── tests/
│   └── mod_tests.rs            # Subsystem & unit tests
├── assets/
│   └── example_mod/
│       ├── lang/
│       │   └── en_us.json      # Localizations
│       └── textures/
│           ├── blocks/         # Block textures extracted from Java mod
│           └── items/          # Item textures extracted from Java mod
└── data/
    └── example_mod/
        └── recipes/            # Crafting and transmutation recipes
```

## Running Tests

```bash
cargo test --manifest-path mods/example_mod/Cargo.toml
```
