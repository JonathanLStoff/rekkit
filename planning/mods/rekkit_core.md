# `rekkit_core` — Shared Foundation Library Specification

## 1. Overview & Version Target
* **Crate Path:** [`crates/rekkit_core`](file:///Users/jonathanstoff/Desktop/scripts/rekkit/crates/rekkit_core) (`crate-type = ["rlib"]`)
* **Version:** `0.1.0`
* **Target Horizon Engine:** `38a97ea64e816e1f897d1fbc7c7ded3b0ce87f56` (`NATIVE_HOOKS_API = 40`, Content Mod API `0.2`)
* **Inclusion Rule:** A concept lives in `rekkit_core` **only if two or more queued Rekkit mods require it**. `crates/rekkit_core` lives under `crates/` (never under `mods/`) because `scripts/pack_mod.py` and `scripts/e2e_harness.py` treat every `mods/*/Cargo.toml` as a full dual-artifact mod.

---

## 2. Shared Concepts & Consumer Mod Matrix

| Subsystem Module | Shared Types & Functions | Queued Mods Using It ($\ge 2$ required) |
| :--- | :--- | :--- |
| **`energy`** | `EuTier`, `EuPacket`, `EuCableKind`, `EuStorage`, `MjBuffer`, `EngineHeatStage`, `BlutricityNode`, `AeEnergyBuffer`, `EnergyUnit`, `convert_energy`, NBT energy sync helpers | `industrialcraft_2`, `buildcraft`, `redpower_2`, `thermal_expansion`, `applied_energistics`, `railcraft`, `forestry`, `advanced_machines`, `charging_bench`, `compact_solars`, `mffs`, `power_converters`, `charge_pads`, `gravitation_suite`, `modular_powersuits` |
| **`inventory`** | `ItemStack`, `Inventory`, slot insertion/extraction rules, sided slot masks, redstone comparator fill signal (`0..=15`) | All 10 Wave 1 mods (`redpower_2`, `buildcraft`, `industrialcraft_2`, `equivalent_exchange_2`, `computercraft`, `applied_energistics`, `railcraft`, `thermal_expansion`, `forestry`, `enderstorage`) + `iron_chests`, `tubestuff` |
| **`fluid`** | `FluidStack`, `FluidTank` (millibuckets `mB`), fill/drain transfers, NBT serialization (`fluid_id`, `fluid_mb`) | `buildcraft`, `railcraft`, `thermal_expansion`, `forestry`, `enderstorage`, `industrialcraft_2`, `redpower_2`, `additional_pipes`, `xycraft` |
| **`side`** | `Direction` (`Down`, `Up`, `North`, `South`, `West`, `East`), `ConnectionMask` (6-bit neighbor mask for `native_update_shape` API 32), `SideMode` & `SideConfig` (CoFH 6-color IO matrix + auto-output) | `thermal_expansion`, `redpower_2`, `buildcraft`, `industrialcraft_2`, `applied_energistics`, `computercraft`, `omni_tools` |
| **`grid`** | `BlockPos`, `UnionFind`, `GridGraph` (connected components + weighted Dijkstra/BFS pathfinding with color filters and restrictor weights) | `redpower_2` (wires, bundled cables, pneumatic tubes, frames), `buildcraft` (item/fluid/conductive pipes), `industrialcraft_2` (EU cable networks), `applied_energistics` (ME cable bus), `computercraft` (wired modems) |
| **`frequency`** | `DyeColor` (`0..16`), `BundledSignal` (16-bit / 16-channel analog wire bus), `FrequencyKey` ($16^3 = 4,096$ dye triads), `ChannelOwner` (`Public` / `Private(String)`) | `enderstorage`, `redpower_2`, `computercraft`, `wireless_redstone_cbe`, `thermal_expansion` (Tesseracts), `buildcraft` (pipe wires & Diamond Pipe filters), `additional_pipes` |
| **`multiblock`** | `MultiblockSpec`, solid cuboid validator, hollow cuboid validator, variable-size tank/boiler/MAC bounds detector | `railcraft` (Coke Oven, Blast Furnace, Steam Boilers, Iron/Steel Tanks), `forestry` (Alveary, Multi-Farms), `applied_energistics` (Molecular Assembler Chamber, ME Controller), `industrialcraft_2` (Nuclear Reactor chambers) |
| **`hooks` & `util`** | `HorizonNativeMod` trait (`NATIVE_HOOKS_API = 40`), `NativeModInfo`, `ModContainer`, `ModBlockEntity`, `NativeScreen`, `TickTimer`, `XorShift64` deterministic RNG | Every Rekkit mod crate under `mods/*` |

---

## 3. Core Systems & Exact Formulas

### 3.1 Energy Systems & Cross-Mod Conversions (`energy`)

#### A. IndustrialCraft 2 Energy Units (`EU`)
* **Voltage Tiers (`EuTier`):**
  * `LV` (Low Voltage): max `32` EU/packet.
  * `MV` (Medium Voltage): max `128` EU/packet.
  * `HV` (High Voltage): max `512` EU/packet.
  * `EV` (Extreme Voltage): max `2048` EU/packet.
* **Over-Voltage Explosion Rule:**
  $$\text{Explodes} \iff \text{PacketVoltage} > \text{MachineMaxTier.max\_voltage()}$$
  When `EuStorage::receive_packet(packet)` receives a packet whose voltage exceeds `max_tier.max_voltage()`, it returns `Err(OverVoltageError { packet_voltage, max_safe_voltage })` so the machine hook can trigger `native_set_block(..., "minecraft:air")` and explode.
* **Cable Distance Loss (`EuCableKind`):**
  $$\text{Delivered EU} = \max\left(0.0,\ \text{PacketEU} - \text{DistanceBlocks} \times \text{LossPerBlock}\right)$$
  * `Tin`: max `32` EU/p, loss `0.025` EU/block (1 EU every 40 blocks).
  * `Copper`: max `32` EU/p, loss `0.20` EU/block (1 EU every 5 blocks).
  * `Gold`: max `128` EU/p, loss `0.40` EU/block (1 EU every 2.5 blocks).
  * `HvIron`: max `512` EU/p, loss `0.80` EU/block (1 EU every 1.25 blocks).
  * `GlassFibre`: max `512` EU/p, loss `0.025` EU/block (1 EU every 40 blocks).
  * `HvTransformer`: max `2048` EU/p, loss `0.80` EU/block.

#### B. BuildCraft / Thermal Expansion / Forestry / Railcraft Minecraft Joules (`MJ`)
* **`MjBuffer`**: Tracks `stored_mj: f64`, `capacity_mj: f64`, `max_in_mj: f64`, `max_out_mj: f64`.
* **Engine Thermal Stages (`EngineHeatStage`):**
  * Ratio $r = \text{heat} / \text{max\_heat}$:
    * `Blue`: $0.0 \le r < 0.25$
    * `Green`: $0.25 \le r < 0.50$
    * `Yellow`: $0.50 \le r < 0.75$
    * `Red`: $0.75 \le r < 1.00$
    * `Overheated` (Explodes if `can_explode == true`): $r \ge 1.00$
* **Conduit Loss (`RedstoneEnergyConduit`):** Fixed `5%` loss ($0.95\times$ transmitted MJ) regardless of distance.

#### C. RedPower 2 Blutricity (`V`, `I`, `Q`)
* **`BlutricityNode`:**
  * Nominal operating voltage: `100.0 V`.
  * Minimum machine operating threshold: `60.0 V`.
  * Maximum safe peak voltage: `120.0 V`.
  * Ohm's Law charge flow between two adjacent nodes $A$ and $B$ with link resistance $R > 0$:
    $$I = \frac{V_A - V_B}{R}, \qquad \Delta Q = I \cdot \Delta t$$

#### D. Applied Energistics Energy (`AE`)
* **`AeEnergyBuffer`:**
  * Stores `stored_ae: f64`, `capacity_ae: f64`, and `idle_drain_per_tick: f64`.
  * `tick_idle()` deducts `idle_drain_per_tick`; if `stored_ae < idle_drain_per_tick`, the network enters an unpowered (`powered = false`) state.

#### E. Canonical Cross-Mod Energy Conversion Ratios (`Power Converters` & `AE1` Standard)
All conversions route through an exact internal base unit (`1 MJ = 10 RF = 2.5 EU = 5 AE = 6.25 Blutricity Watt-ticks`):
* $1 \text{ MJ} = 2.5 \text{ EU}$ ($2 \text{ MJ} = 5 \text{ EU}$)
* $1 \text{ MJ} = 5.0 \text{ AE}$
* $1 \text{ EU} = 2.0 \text{ AE}$
* $1 \text{ MJ} = 10.0 \text{ RF}$
* $1 \text{ EU} = 2.5 \text{ W (Blutricity)}$

#### F. Standardized `ModBlockEntity.nbt` Energy & Fluid Keys
For cross-mod `native_with_mod_block_entity` (`API 18`) interoperability:
* `eu_stored`, `eu_capacity`, `eu_max_tier`
* `mj_stored`, `mj_capacity`
* `bt_voltage`, `bt_charge`
* `ae_stored`, `ae_capacity`
* `emc_stored`, `emc_capacity`
* `fluid_id`, `fluid_mb`, `fluid_capacity_mb`

---

### 3.2 Item Stacks, Inventories & Comparator Math (`inventory`)
* **`ItemStack`**: `{ item_id: String, count: u32, damage: u16, max_stack_size: u32 }`.
* **`Inventory`**:
  * Fixed slot count $N$.
  * `insert_item(stack)` and `insert_into_slots(stack, allowed_slots)`: first merges into existing matching stacks (`item_id` and `damage` match) up to `max_stack_size` (`1..=99`), then places into empty allowed slots, returning any remainder `Option<ItemStack>`.
  * `extract_from_slots(count, allowed_slots)`: extracts up to `count` items from the first non-empty allowed slot.
  * `redstone_signal() -> i32` (`API 38` `native_block_signal`):
    $$\text{Signal} = \begin{cases} 0 & \text{if all slots empty} \\ 1 + \left\lfloor 14 \times \frac{\sum_i \text{count}_i / \text{max\_stack}_i}{\text{slot\_count}} \right\rfloor & \text{otherwise} \end{cases}$$

---

### 3.3 Fluid Tanks (`fluid`)
* **`FluidStack`**: `{ fluid_id: String, amount_mb: u32 }`.
* **`FluidTank`**:
  * `capacity_mb: u32`, `contents: Option<FluidStack>`.
  * `fill(incoming: FluidStack, do_fill: bool) -> u32`: returns millibuckets accepted (`0` if tank holds a different fluid).
  * `drain(max_drain_mb: u32, do_drain: bool) -> Option<FluidStack>`: drains up to `max_drain_mb`.
  * `write_nbt(&self, nbt: &mut HashMap<String, String>)` and `read_nbt(capacity_mb: u32, nbt: &HashMap<String, String>) -> Self`.

---

### 3.4 Directions, 6-Bit Connection Masks & CoFH Side Config (`side`)
* **`Direction`** (`repr(u8)` matching Horizon `ModContainer.faces` order):
  * `0: Down (0, -1, 0)`
  * `1: Up (0, 1, 0)`
  * `2: North (0, 0, -1)`
  * `3: South (0, 0, 1)`
  * `4: West (-1, 0, 0)`
  * `5: East (1, 0, 0)`
* **`ConnectionMask(u8)`**:
  * 6-bit mask (`bit = 1 << (dir as u8)`) representing connected cable/pipe/tube/wire neighbors for `native_update_shape` (`API 32`).
* **`SideMode`** (Thermal Expansion 6-Color Matrix):
  * `Disabled` (Grey)
  * `InputPrimary` (Blue)
  * `OutputPrimary` (Red)
  * `InputSecondary` (Green)
  * `OutputSecondary` (Yellow)
  * `Omni` (Orange — both input and output)
* **`SideConfig`**:
  * Maps each of the 6 `Direction`s to a `SideMode` and builds Horizon's `faces: [Vec<usize>; 6]` array automatically from primary/secondary input and output slot lists.

---

### 3.5 Connected Grids: Union-Find & Weighted Tube/Pipe Routing (`grid`)
* **`BlockPos`**: `(i32, i32, i32)` with `offset(Direction)`, `neighbors()`, and `manhattan_distance(other)`.
* **`UnionFind`**: Disjoint-set data structure with path compression and rank union for fast cable/wire/frame cluster membership.
* **`GridGraph`**:
  * Stores nodes at `BlockPos` with per-node traversal cost (`1` for normal tubes/pipes, `500` for RedPower `RestrictorTube`) and optional `DyeColor` channel tag.
  * `connected_components() -> Vec<Vec<BlockPos>>` using `UnionFind`.
  * `shortest_path(start, goals, item_color) -> Option<(BlockPos, Vec<BlockPos>, u32)>`: Dijkstra shortest-path solver implementing **RedPower 2 Pneumatic Tube strict pathfinding** (only traverses uncolored or matching-color tubes, respects restrictor +500 cost, and picks the nearest valid recipient).

---

### 3.6 16-Color Dyes, Bundled Signals & Frequency Triads (`frequency`)
* **`DyeColor` (`0..16`)**: White (`0`) through Black (`15`), `from_dye_item("minecraft:*_dye")`, `next()`, `rgb_hex()`.
* **`BundledSignal(u16, [u8; 16])`**:
  * 16-color bundled cable state supporting both ComputerCraft's 16-bit digital bitmask (`1 << color`) and RedPower 2's per-color analog strength (`0..=255`).
* **`FrequencyKey(DyeColor, DyeColor, DyeColor)`**:
  * 12-bit index `0..4096` (`(c0 << 8) | (c1 << 4) | c2`), `cycle_pad(0..3)`, and `ChannelOwner` (`Public` vs `Private(String)`).

---

### 3.7 Multiblock Pattern Validation (`multiblock`)
* **`MultiblockShape`**:
  * `SolidCuboid { width: i32, height: i32, depth: i32, block_id: String }` (e.g. Railcraft 3×3×3 Coke Oven, Forestry 3×3×3 Alveary).
  * `HollowCuboid { width: i32, height: i32, depth: i32, casing_ids: Vec<String>, allow_air_interior: bool }` (e.g. Railcraft 3×3×4 Blast Furnace, Iron/Steel Fluid Tanks 3×3×4..9×9×8, AE1 Molecular Assembler Chamber 3×3×3..16×16×16).
* `validate_at(origin: BlockPos, get_block: impl Fn(BlockPos) -> Option<String>) -> bool`.

---

## 4. Hooks Needed
`rekkit_core` is a pure library crate (`rlib`) and registers no direct mixin bundle, but defines the canonical `HorizonNativeMod` trait (`NATIVE_HOOKS_API = 40`) and mirror types (`NativeModInfo`, `ModContainer`, `ModBlockEntity`, `NativeScreen`) used by all Rekkit mods across Hooks **API 8–40**.

## 5. Open Questions
None. All formulas and types are verified against the 10 Wave 1 specifications in `planning/mods/*.md`.
