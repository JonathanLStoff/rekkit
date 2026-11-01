# RedPower 2 - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** RedPower 2 Pre-Release 6 (v2.0pr6) for Minecraft 1.4.6/1.4.7 (and v2.0pr5b2 for Minecraft 1.2.5).
* **Author:** Eloraam.
* **Core Philosophy:** Modular, elegant automation without magic single-block teleporters or pipe overflows. Precise digital logic, multi-colored wiring, pneumatic item transport with strict pathfinding, microblocks, frames for moving structures, and analog 6502-based computing.

---

## 2. Architecture & Modules
RedPower 2 is split into several sub-modules:
1. **Core:** Fundamental materials, alloys, tools, gems, marble/basalt, microblocks system, and alloy furnaces.
2. **Logic & Wiring:** Red alloy wires, insulated wires, bundled cables, jackets, integrated logic tiles (AND, OR, NOT, Timer, Counter, Sequencer, Latches).
3. **Lighting:** Plain and color-dyed lamps, inverted lamps, powered by redstone or blutricity.
4. **Machine & Tubes:** Pneumatic tubes, machine blocks (Deployer, Breaker, Transposer, Filter, Sorting Machine, Retriever, Regulator, Item Detector, Buffer, Relay), and Frame motors.
5. **Blutricity (Power):** Low-voltage DC energy system (100V nominal), solar panels, thermopiles, wind turbines, battery boxes, transformers, blulectric furnaces, and pumps.
6. **Control:** 6502-based Forth microcomputer, monitor, keyboard, disk drive, ribbon cables, and IO expanders.
7. **World:** World generation (gems, ores, marble, basalt volcanoes, rubber trees, flax plants).

---

## 3. Core Mechanics & Subsystems

### 3.1 Microblocks System
* **Concept:** Regular 1x1x1 blocks cut down into 1/8th fractional slices using a Handsaw.
* **Hierarchy:**
  * Slab / Half-Block (4/8th = 1/2 block)
  * Panel (2/8th = 1/4 block)
  * Cover (1/8th block)
  * Hollow versions (Cover, Panel, Slab with 1/2 width central cutout for wires/tubes passing through)
  * Strips: Corner Strip (1/8 x 1/8 x 1), Strip Panel (1/4 x 1/8 x 1), Strip Slab (1/2 x 1/8 x 1)
  * Corners: Corner (1/8 x 1/8 x 1/8), Corner Panel, etc.
* **Placement:** Surface-anchored to any face of an internal 1x1x1 block voxel. Multiple microblocks (up to 6 covers on 6 faces, plus edges/corners) can occupy a single block space.
* **Collision:** Exact compound axis-aligned bounding boxes (AABB) matching placed microblocks.
* **Occlusion & Masking:** Microblocks isolate wires and block connections between adjacent faces unless hollow covers are used.

### 3.2 Wiring System
* **Red Alloy Wire:**
  * Transmits standard redstone signal (0-255 internal, mapped to 0-15 standard redstone) without signal loss up to 256 blocks.
  * Can be placed on floors, walls, and ceilings. Can turn 90-degree internal and external corners along block edges.
* **Insulated Wire:**
  * 16 distinct dye colors (White, Orange, Magenta, Light Blue, Yellow, Lime, Pink, Gray, Light Gray, Cyan, Purple, Blue, Brown, Green, Red, Black).
  * Only connects to wires of matching color or bare Red Alloy wire. Prevents crosstalk when run side-by-side.
* **Bundled Cable:**
  * Encapsulates all 16 colored signals into a single cable.
  * Connects to insulated wire by splitting out the matching color signal.
  * Connects to Logic Gates and Control IO directly.
* **Jacketed Wire / Cable:**
  * Freestanding rigid wires/cables that do not require attachment to a solid block face.

### 3.3 Integrated Logic Gates
All logic tiles are 1/8-block flat tiles placeable on any face, rotatable in 4 directions.
* **Inverter (NOT):** Inverts 1 input to 1-3 outputs.
* **AND Gate:** Outputs high when all configured inputs are high.
* **OR Gate:** Outputs high when any configured input is high.
* **NAND Gate:** Inverted AND.
* **NOR Gate:** Inverted OR.
* **XOR Gate:** Outputs high when an odd number of inputs are high.
* **XNOR Gate:** Outputs high when inputs match.
* **Buffer:** Delays signal by 1 tick, isolates input from backflow.
* **Multiplexer:** Selects between two inputs based on a control line.
* **Repeater:** Configurable delay (1 to 4 redstone ticks).
* **Timer:** Pulses periodically. Delay configurable from 0.2s to continuous clock cycles via GUI slider.
* **Counter:** Maintains a count value incremented/decremented by left/right pulses; emits pulse on reaching max/min.
* **Sequencer:** Cycles through 4 outputs in clockwise or counterclockwise rotation on pulse.
* **Pulse Former:** Converts continuous high input into a single 1-tick pulse on rising edge.
* **Toggle Latch:** Toggles output state on each input pulse (T flip-flop).
* **RS Latch:** Set/Reset latch with independent inputs.
* **Transparent Latch:** Passes data through while clock is high; latches state when clock drops low.
* **State Cell:** Monostable pulse timer with configurable duration.
* **Synchronizer:** Waits for two independent pulses to arrive before firing joint output pulse.
* **Light Sensor:** Emits analog or digital signal based on ambient skylight/blocklight.

### 3.4 Pneumatic Tubes & Item Logistics
* **Strict Destination Pathfinding:**
  * Items inside tubes **never drop into the world** unless an inventory is broken or completely invalid.
  * When an item enters the tube system (from a Transposer, Filter, Sorting Machine, etc.), the origin calculates the route to the **nearest valid recipient inventory** with available space using Dijkstra / BFS.
  * If no destination has room, the machine will not extract or emit the item.
  * If a path is blocked midway (e.g. tube broken or inventory filled en route), the item turns back or searches for a "safe" overflow buffer / chest.
* **Tube Colors:**
  * Tubes can be painted with a Paintbrush (16 colors).
  * Color-tagged items will only travel down uncolored tubes or tubes matching their tagged color.
* **Restrictor Tube:**
  * Counts as 500 blocks of virtual distance in pathfinding calculations, serving as an overflow priority route.
* **Magtube & Accelerator:**
  * Accelerates item payloads into capsules traveling at ultra-high speed (up to 16 blocks/tick) using Blutricity power.
* **Machines:**
  * **Transposer:** Extracts 1 item from adjacent inventory or sucks items off ground on redstone pulse; injects into tube.
  * **Filter:** Extracts 1 item matching a 9-slot ghost inventory filter on redstone pulse. Can assign color tag to routed items.
  * **Sorting Machine:** Advanced routing with 8 color modes:
    1. Any items matching column color.
    2. Exact stack sizes matching column.
    3. Sequential cycling.
    4. Auto-pulling with Blutricity power.
  * **Retriever:** Pulls items across the tube network from distant inventories to itself when pulsed, requesting matching filtered items.
  * **Regulator:** Compares adjacent inventory contents to a 9-slot template; extracts/requests items to keep inventory exactly stocked.
  * **Item Detector:** Emits redstone pulse whenever an item passes through the internal tube.
  * **Buffer:** 9-slot dual-sided inventory buffer to decouple fast extraction from slow processing.
  * **Deployer:** Simulates right-click of held item onto adjacent block on redstone pulse (places blocks, uses buckets, bones meals, shears sheep).
  * **Block Breaker:** Breaks adjacent block on redstone pulse, drops item into attached tube or inventory behind it.

### 3.5 Frame Motors & Structural Movement
* **Support Frames:**
  * Structural blocks that latch onto adjacent blocks on specified sides (configured with Screwdriver or Covers).
  * Form contiguous rigid multiblock bodies.
* **Frame Motor:**
  * Driven by redstone pulse or continuous signal.
  * Moves the entire connected frame assembly and attached blocks by 1 block in the designated direction.
  * Retains tile entity states, inventory contents, wires, tubes, and machines intact during motion.
  * Physics: Sweeps candidate blocks, checks for immovable blocks (Bedrock, Obsidian depending on config), moves all blocks simultaneously in 1 world tick.

### 3.6 Blutricity (Power System)
* **Model:** Voltage (V), Current (I), and Charge (Q).
  * Nominal operating voltage: 100V.
  * Peak safe voltage: ~120V; minimum machine running threshold: ~60V.
  * Wires have internal electrical resistance ($V = I \cdot R$); voltage drops over distance.
* **Generation:**
  * **Solar Panel:** Generates current during daylight (direct sky exposure).
  * **Thermopile:** Generates current from temperature gradients across opposite faces (Lava vs Water/Ice).
  * **Wind Turbine:** Mounted on tall masts (higher elevation = higher wind power). Requires Wooden or Canvas sails that wear over time.
* **Storage:**
  * **Battery Box:** Holds 8 rechargeable Nikolite/Lead batteries. Stores ~1,200,000 Joules.
* **Consumers:**
  * **Blulectric Alloy Furnace:** Smelts alloy recipes without fuel.
  * **Blulectric Furnace:** Smelts standard ores/smeltables at 2x speed.
  * **Blulectric Pump:** Pumps liquids (lava, water, oil) into pipes/tubes.

### 3.7 RedPower Control (Computer Subsystem)
* **Architecture:** Emulated MOS Technology 6502 8-bit microprocessor with 64KB addressing space.
* **Components:**
  * **Central Processing Unit (CPU):** Executes 6502 machine code and Forth operating system (FORTH-83/RedPower Forth).
  * **Monitor:** Displays 80x25 characters or monochrome raster graphics.
  * **Disk Drive:** Reads/writes 8-inch virtual floppy disks (128 KB storage).
  * **Keyboard:** Real-time text input.
  * **IO Expander:** Memory-mapped 8-bit / 16-bit registers connected to Bundled Cables to sense and drive redstone signals.
  * **Ribbon Cable:** High-density 16-conductor bus connecting CPU to peripherals.

---

## 4. World Generation & Resources

### 4.1 Ores & Minerals
* **Ruby Ore:** Spawns layers 1-48, vein size 3-5, drops Ruby.
* **Sapphire Ore:** Spawns layers 1-48, vein size 3-5, drops Sapphire.
* **Green Sapphire Ore:** Spawns layers 1-48, vein size 3-5, drops Green Sapphire.
* **Nikolite Ore:** Spawns layers 1-16 (deep underground), drops 4-5 Nikolite dust per ore. Essential for all RedPower electronics, logic, batteries, and blue alloy.
* **Copper Ore:** Spawns layers 1-64, vein size 6-10.
* **Tin Ore:** Spawns layers 1-48, vein size 4-8.
* **Silver Ore:** Spawns layers 1-32, vein size 3-6.
* **Tungsten Ore:** Extremely rare, spawns layers 1-16.

### 4.2 Natural Formations & Plants
* **Marble:** Large subterranean strata and white cavern formations. Can be carved into decorative bricks.
* **Basalt:** Volcanic columns and volcanoes with active lava chambers. Basalt Cobblestone, Basalt Stone, and Basalt Pavers.
* **Rubber Tree:** Produces Rubber Saplings and Rubber Wood; leaves yield Latex / Raw Rubber when harvested.
* **Flax:** Wild flax plants in grasslands; harvested for Flax Seeds and String (used for canvas).

---

## 5. Complete Item & Block Registry (`redpower_2`)

### Blocks (`data/redpower_2/horizon/block/*.json`)
1. `redpower_2:ore_ruby`
2. `redpower_2:ore_sapphire`
3. `redpower_2:ore_green_sapphire`
4. `redpower_2:ore_nikolite`
5. `redpower_2:ore_copper`
6. `redpower_2:ore_tin`
7. `redpower_2:ore_silver`
8. `redpower_2:ore_tungsten`
9. `redpower_2:block_marble`
10. `redpower_2:block_basalt`
11. `redpower_2:alloy_furnace`
12. `redpower_2:blulectric_alloy_furnace`
13. `redpower_2:blulectric_furnace`
14. `redpower_2:deployer`
15. `redpower_2:block_breaker`
16. `redpower_2:transposer`
17. `redpower_2:filter`
18. `redpower_2:sorting_machine`
19. `redpower_2:retriever`
20. `redpower_2:regulator`
21. `redpower_2:item_detector`
22. `redpower_2:buffer`
23. `redpower_2:relay`
24. `redpower_2:frame`
25. `redpower_2:frame_motor`
26. `redpower_2:pneumatic_tube`
27. `redpower_2:restrictor_tube`
28. `redpower_2:magtube`
29. `redpower_2:accelerator`
30. `redpower_2:battery_box`
31. `redpower_2:solar_panel`
32. `redpower_2:thermopile`
33. `redpower_2:wind_turbine`
34. `redpower_2:pump`
35. `redpower_2:lamp`
36. `redpower_2:inverted_lamp`
37. `redpower_2:cpu`
38. `redpower_2:monitor`
39. `redpower_2:keyboard`
40. `redpower_2:disk_drive`
41. `redpower_2:io_expander`
42. `redpower_2:microblock`

### Items (`data/redpower_2/horizon/item/*.json`)
1. `redpower_2:handsaw` (`crafting_remainder = "redpower_2:handsaw"`)
2. `redpower_2:screwdriver` (`crafting_remainder = "redpower_2:screwdriver"`)
3. `redpower_2:paint_brush`
4. `redpower_2:canvas_bag`
5. `redpower_2:sickle`
6. `redpower_2:wool_card`
7. `redpower_2:ruby`
8. `redpower_2:sapphire`
9. `redpower_2:green_sapphire`
10. `redpower_2:nikolite`
11. `redpower_2:red_alloy_ingot`
12. `redpower_2:blue_alloy_ingot`
13. `redpower_2:brass_ingot`
14. `redpower_2:copper_ingot`
15. `redpower_2:tin_ingot`
16. `redpower_2:silver_ingot`
17. `redpower_2:tungsten_ingot`
18. `redpower_2:silicon_wafer`
19. `redpower_2:red_doped_wafer`
20. `redpower_2:blue_doped_wafer`
21. `redpower_2:floppy_disk`
22. `redpower_2:canvas`
23. `redpower_2:battery`
24. `redpower_2:logic_tile`
25. `redpower_2:wire_red_alloy`
26. `redpower_2:wire_insulated`
27. `redpower_2:wire_bundled`
28. `redpower_2:wire_jacketed`
29. `redpower_2:ribbon_cable`

---

## 6. Key Crafting, Smelting & Alloy Furnace Recipes

### 6.1 Custom Machine Recipes (`"type": "redpower_2:alloy_furnace"`)
* **Red Alloy Ingot (`redpower_2:alloy_red_alloy`):** 4x `minecraft:redstone` + 1x `redpower_2:copper_ingot` $\rightarrow$ 1x `redpower_2:red_alloy_ingot`.
* **Blue Alloy Ingot (`redpower_2:alloy_blue_alloy`):** 4x `redpower_2:nikolite` + 1x `redpower_2:silver_ingot` $\rightarrow$ 1x `redpower_2:blue_alloy_ingot`.
* **Brass Ingot (`redpower_2:alloy_brass`):** 3x `redpower_2:copper_ingot` + 1x `redpower_2:tin_ingot` $\rightarrow$ 4x `redpower_2:brass_ingot`.
* **Red-Doped Wafer (`redpower_2:alloy_red_doped_wafer`):** 4x `minecraft:redstone` + 1x `redpower_2:silicon_wafer` $\rightarrow$ 1x `redpower_2:red_doped_wafer`.
* **Blue-Doped Wafer (`redpower_2:alloy_blue_doped_wafer`):** 4x `redpower_2:nikolite` + 1x `redpower_2:silicon_wafer` $\rightarrow$ 1x `redpower_2:blue_doped_wafer`.

### 6.2 Standard Crafting (`"type": "minecraft:crafting_shaped"` / `"minecraft:crafting_shapeless"`)
* **Red Alloy Wire (`redpower_2:wire_red_alloy`, x12):** 3x `redpower_2:red_alloy_ingot` in a horizontal row.
* **Insulated Wire (`redpower_2:wire_insulated`):** `redpower_2:wire_red_alloy` + `minecraft:white_wool`.
* **Bundled Cable (`redpower_2:wire_bundled`):** 5x `redpower_2:wire_insulated` + 4x `minecraft:string`.
* **Handsaw (`redpower_2:handsaw`):** 3x `minecraft:stick` + 2x `minecraft:iron_ingot` + 2x `minecraft:diamond`.
* **Screwdriver (`redpower_2:screwdriver`):** 1x `minecraft:iron_ingot` + 1x `minecraft:stick`.
* **Alloy Furnace (`redpower_2:alloy_furnace`):** 8x `minecraft:bricks` in a furnace ring.
* **Block Breaker (`redpower_2:block_breaker`):** 6x `minecraft:cobblestone` + 1x `minecraft:iron_pickaxe` + 1x `minecraft:piston` + 1x `minecraft:redstone`.
* **Deployer (`redpower_2:deployer`):** 6x `minecraft:cobblestone` + 1x `minecraft:chest` + 1x `minecraft:piston` + 1x `minecraft:redstone`.
* **Transposer (`redpower_2:transposer`):** 6x `minecraft:cobblestone` + 1x `minecraft:OAK_TRAPDOOR` + 1x `minecraft:piston` + 1x `minecraft:redstone`.
* **Pneumatic Tube (`redpower_2:pneumatic_tube`, x8):** `redpower_2:brass_ingot` + `minecraft:glass` + `redpower_2:brass_ingot`.

---

## 7. Hazards & Safety Limits
* **6502 CPU Cycle Runaway:** Unbounded Forth / 6502 loops could stall server ticks; bounded to `MAX_CYCLES_PER_TICK = 4,096` instructions per world tick per CPU.
* **Frame Motor Mass Limit & Unmovable Blocks:** Frame Motors refuse to move Bedrock (`minecraft:bedrock`), Obsidian (unless configured), or assemblies exceeding `MAX_FRAME_BLOCKS = 1,024` blocks.
* **Pneumatic Tube Backpressure:** Items never spill onto the ground when destinations are full; extraction halts or items bounce back to an overflow buffer.

---

## 8. Recreation Implementation Notes for Rekkit (Rust)
1. **Multi-Part / Voxel Model:** Use a sub-voxel bitmask (8 octants, 12 edges, 6 faces) inside block entities to support multi-part microblocks, wires, and tiles in a single coordinate.
2. **Tube Pathfinding:** Delegate tube graph connectivity and Dijkstra routing (with color filtering and Restrictor +500 weight) to `rekkit_core::grid::GridGraph`.
3. **Frame Movement Physics:** Collect all rigidly connected blocks via BFS (`FrameAssembly`), validate destination bounding volume, and move atomically via `Game::native_set_block` (`API 24`).
4. **6502 Emulator:** Pure Rust 6502 CPU state machine (`Cpu6502`) with 64 KB RAM, memory-mapped I/O Expander (`BundledSignal`), and bounded cycle execution.

---

## 9. Hooks Needed (`NATIVE_HOOKS_API = 40`)

| Horizon Hook API | Entrypoint in `native_hooks.rs` | RedPower 2 Feature |
| :--- | :--- | :--- |
| **API 13** | `Game::native_use_item_on` | Screwdriver gate rotation/mode cycling, Paintbrush tube coloring, Handsaw microblock slicing. |
| **API 15** | `Game::native_block_entity`, `Game::native_block_entity_ticks` | Attach and tick block entities for Alloy Furnace, Blulectric machines, Logic Gates, Tubes, Frame Motor, and 6502 CPU. |
| **API 16–17** | `Game::native_block_entity_container`, `Game::native_block_entity_tick` | Alloy Furnace (9-slot 3×3 grid + fuel + output), Buffer (9 slots), Filter (9 slots), Sorting Machine, Battery Box. |
| **API 18** | `Game::native_with_mod_block_entity`, `Game::native_set_block_property` | Blutricity charge transfer ($V = I \cdot R$), Pneumatic Tube item delivery, Lamp `lit` state updates. |
| **API 19–20** | `Game::native_screen`, `Game::native_screen_click`, `Game::native_screen_tooltip`, `Game::native_menu_button`, `Game::native_insert_item` | Alloy Furnace GUI, Timer interval buttons, Filter/Buffer screens, Monitor terminal, tube item insertion. |
| **API 24** | `Game::native_set_block` | Block Breaker mining adjacent blocks, Deployer placing blocks, Frame Motor translating structures. |
| **API 32** | `Game::native_update_shape`, `Game::native_schedule_block_tick` | Dynamic wire/cable/tube 6-face neighbor shape connections and logic gate delay scheduling. |
| **API 33–34** | `Game::native_random_tick`, `Game::native_tick_block` | Flax/Rubber tree growth and Logic Gate scheduled tick evaluation (Timer, Repeater, Pulse Former, State Cell, Sequencer). |
| **API 36** | `Game::native_recipes_of_type` | Custom `"type": "redpower_2:alloy_furnace"` recipes loaded from `content/data/redpower_2/recipe/*.json`. |
| **API 38** | `Game::native_block_signal`, `Game::native_is_signal_source`, `Game::native_update_signal_neighbors` | Red Alloy Wire (`0..=255` mapped to `0..=15`), Logic Gate outputs, Item Detector pulses. |
| **API 40** | `Game::native_net_from_client`, `Game::native_net_send_to_player` | 6502 Monitor/Keyboard keystrokes and 80×25 screen buffer synchronization on `horizon:mod_net`. |

---

## 10. Uses from `rekkit_core`
* `rekkit_core::energy::BlutricityNode` ($100\text{V}$ nominal, $60\text{V}$ minimum operating threshold, Ohm's law $I = (V_a - V_b)/R$).
* `rekkit_core::frequency::{DyeColor, BundledSignal}` (16-color Insulated Wires, 16-channel Bundled Cables, painted Pneumatic Tubes, and 6502 I/O Expander).
* `rekkit_core::grid::{BlockPos, GridGraph, GridNode}` (Pneumatic Tube & Restrictor Tube `+500` Dijkstra shortest-path routing and wire cluster union-find).
* `rekkit_core::side::{Direction, ConnectionMask}` (6-face wire/tube connection masks and machine orientation).
* `rekkit_core::inventory::{Inventory, ItemStack}` (Alloy Furnace, Buffer, Filter, Sorting Machine, and Canvas Bag inventories).
* `rekkit_core::hooks::{HorizonNativeMod, ModBlockEntity, ModContainer, NativeModInfo, NativeScreen}`.
* `rekkit_core::util::TickTimer` (Logic Gate Timer / State Cell / Sequencer intervals and Alloy Furnace smelting progress).

---

## 11. Open Questions
* Legacy sprite sheet indices (`eloraam/*` in `RedPowerCore`/`RedPowerWiring`/`RedPowerMachine`/`RedPowerControl`/`RedPowerWorld`) are recorded in `mods/redpower_2/asset-map.json` and marked `UNVERIFIED` until extracted from the Tekkit/DW20 archive via `make extract-jar`.
