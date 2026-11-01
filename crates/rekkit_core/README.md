# `rekkit_core`

Shared foundation crate (`rlib`) for Rekkit mods targeting the **[Horizon](file:///Users/jonathanstoff/Desktop/scripts/rekkit/horizon)** engine (`NATIVE_HOOKS_API = 40`).

## Modules

- **[`energy`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: IndustrialCraft 2 `EU` (voltage tiers `LV`/`MV`/`HV`/`EV`, over-voltage explosion checks, cable loss), BuildCraft/Thermal Expansion `MJ` (engine thermal stages, 5% conduit loss), RedPower 2 Blutricity ($V, I, Q$ Ohm's law), Applied Energistics `AE`, and `Power Converters` cross-mod conversion ratios (`1 MJ = 2.5 EU = 5 AE = 10 RF`).
- **[`inventory`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: `ItemStack`, fixed-size `Inventory`, sided slot insertion/extraction, and redstone comparator fill signal calculation (`0..=15`).
- **[`fluid`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: `FluidStack` and millibucket `FluidTank` with NBT serialization helpers.
- **[`side`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: 6-face `Direction`, 6-bit `ConnectionMask` (`native_update_shape` API 32), and CoFH 6-color `SideMode` / `SideConfig`.
- **[`grid`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: `BlockPos`, `UnionFind`, and `GridGraph` (connected components + RedPower 2 Dijkstra pneumatic tube routing with color tags and 500-block restrictor weights).
- **[`frequency`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: 16-color `DyeColor`, 16-channel `BundledSignal`, 4,096-channel `FrequencyKey`, and `ChannelOwner`.
- **[`multiblock`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: `MultiblockShape` validation for solid and hollow 3D structures.
- **[`hooks`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: `HorizonNativeMod` trait (`NATIVE_HOOKS_API = 40`) and mirror types (`NativeModInfo`, `ModContainer`, `ModBlockEntity`, `NativeScreen`).
- **[`util`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core/src/lib.rs)**: `TickTimer` and deterministic `XorShift64` RNG.
