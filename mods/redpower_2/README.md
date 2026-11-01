# RedPower 2 (`redpower_2`) for Horizon

Rekkit port of Eloraam's **RedPower 2** (targeting Pre-Release 6 / v2.0pr6 for Minecraft 1.4.6/1.4.7 and v2.0pr5b2 for Minecraft 1.2.5).

RedPower 2 is a modular industrial, logistics, and computational mod built on strict mechanical principles: no magical teleportation, no pipe overflow spills, precise fractional microblocks, multi-channel logic wiring, rigid frame machines, and analog 6502-based computing.

---

## Architecture: Dual-Artifact Mod

Following the Horizon mod specification, `redpower_2` is structured as a dual artifact:

1. **Content Mod (`.hmod`, API 0.2)**
   - Declared in `content/horizon-mod.toml` with `capabilities = ["registries"]`.
   - **42 Blocks**: Ores (Ruby, Sapphire, Green Sapphire, Nikolite, Copper, Tin, Silver, Tungsten), Marble, Basalt, Alloy Furnace, Blulectric machines, Logistics blocks (Deployer, Block Breaker, Transposer, Filter, Sorting Machine, Retriever, Regulator, Item Detector, Buffer, Relay), Tubes (Pneumatic, Restrictor, Magtube, Accelerator), Frames, Blutricity generators and storage, Lamps, and 6502 peripherals.
   - **29 Items**: Handsaw, Screwdriver, Paintbrush, Canvas Bag, Sickle, Wool Card, Gems, Ingots, Silicon and Doped Wafers, Floppy Disk, Battery, Logic Tile, Red Alloy & Bundled Wires, Ribbon Cable.
   - **15 Recipes**: 5 custom Alloy Furnace recipes (`redpower_2:alloy_furnace`) and 10 shaped crafting recipes (`minecraft:crafting_shaped`).
   - **Localization**: Complete `content/assets/redpower_2/lang/en_us.json` mapping all blocks and items.

2. **Native Mod (`.hnmod`, `NATIVE_HOOKS_API = 40`)**
   - Declared in `native/horizon-native.json` and `native/mixins/redpower_2/horizon-mixin.toml`.
   - Pure Rust domain crate in `src/lib.rs` implementing all core state machines and the `HorizonNativeMod` trait.
   - Hooks into Horizon engine events for item interaction (`API 13`), block entities and ticking (`API 15-18`), custom containers/screens (`API 19-20`), block breaking/placing (`API 24`), redstone signals (`API 38`), and network packets (`API 40`).

---

## Subsystems

### 1. Microblocks (`§3.1`)
- 1/8th fractional voxel slicing hierarchy: `Slab (4/8)` $\rightarrow$ `Panel (2/8)` $\rightarrow$ `Cover (1/8)` $\rightarrow$ `CornerStrip` $\rightarrow$ `Corner`.
- Hollow variants with center pass-through cutouts for wiring and pneumatic tubes.
- Sub-voxel compound AABB bounding boxes and collision tests.

### 2. Wiring & Logic (`§3.2 - §3.3`)
- **Red Alloy Wire**: Propagates redstone signals (0..=255 internal, mapped to 0..=15 vanilla) up to 256 blocks with zero loss.
- **Insulated Wire**: 16 distinct dye colors to prevent crosstalk.
- **Bundled Cable**: Encapsulates 16 colored channels into a single conduit (`rekkit_core::frequency::BundledSignal`).
- **20 Integrated Logic Gates**: Inverter, AND, OR, NAND, NOR, XOR, XNOR, Buffer, Multiplexer, Repeater, Timer, Counter, Sequencer, Pulse Former, Toggle Latch, RS Latch, Transparent Latch, State Cell, Synchronizer, and Light Sensor.

### 3. Pneumatic Tubes & Machines (`§3.4`)
- **Strict Destination Pathfinding**: Shortest-path Dijkstra routing (`rekkit_core::grid::GridGraph`) to valid inventory destinations.
- **Zero Item Spills**: Items never drop into the world due to full inventories; machines halt extraction or items return to safe buffers.
- **Tube Coloring & Restrictors**: Restrictor tubes add virtual +500 distance weight for routing priorities.

### 4. Support Frames & Frame Motors (`§3.5`)
- Rigid multiblock assembly graph solver (`FrameAssembly`).
- Translates contiguous structures and block entities atomically while respecting bedrock and obstacle limits.

### 5. Blutricity Power System (`§3.6`)
- Low-voltage DC electrical simulation ($V = I \cdot R$ via `rekkit_core::energy::BlutricityNode`).
- 100V nominal operating voltage, 60V minimum machine threshold.
- Solar Panels, Thermopiles (temperature differentials), Wind Turbines, and Battery Boxes.

### 6. RedPower Control (`§3.7`)
- MOS Technology 6502 8-bit CPU emulator with 64 KB memory space.
- Bounded instruction execution (`MAX_CYCLES_PER_TICK = 4096`).
- Memory-mapped I/O Expander driving 16-channel Bundled Cables and 80x25 character terminal monitor.

---

## Verification & Testing

Run unit tests for domain logic and native hooks:
```bash
cargo test --manifest-path mods/redpower_2/Cargo.toml
```

Validate dual-artifact manifests and content paths:
```bash
make check-mods MOD=redpower_2
```
