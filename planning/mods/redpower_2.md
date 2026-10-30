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

## 5. Complete Item & Block Registry

### Blocks
1. `redpower:ore_ruby`
2. `redpower:ore_sapphire`
3. `redpower:ore_green_sapphire`
4. `redpower:ore_nikolite`
5. `redpower:ore_copper`
6. `redpower:ore_tin`
7. `redpower:ore_silver`
8. `redpower:ore_tungsten`
9. `redpower:block_marble` (and brick variants)
10. `redpower:block_basalt` (cobble, brick, paver variants)
11. `redpower:alloy_furnace`
12. `redpower:blulectric_alloy_furnace`
13. `redpower:blulectric_furnace`
14. `redpower:deployer`
15. `redpower:block_breaker`
16. `redpower:transposer`
17. `redpower:filter`
18. `redpower:sorting_machine`
19. `redpower:retriever`
20. `redpower:regulator`
21. `redpower:item_detector`
22. `redpower:buffer`
23. `redpower:relay`
24. `redpower:frame` (Support Frame)
25. `redpower:frame_motor`
26. `redpower:pneumatic_tube`
27. `redpower:restrictor_tube`
28. `redpower:magtube`
29. `redpower:accelerator`
30. `redpower:battery_box`
31. `redpower:solar_panel`
32. `redpower:thermopile`
33. `redpower:wind_turbine`
34. `redpower:pump`
35. `redpower:lamp` (16 dyed colors + plain)
36. `redpower:inverted_lamp` (16 dyed colors + plain)
37. `redpower:cpu`
38. `redpower:monitor`
39. `redpower:keyboard`
40. `redpower:disk_drive`
41. `redpower:io_expander`
42. `redpower:microblock` (Multi-part container block for all covers/panels/strips/corners)

### Items
1. `redpower:handsaw` (Wood, Stone, Iron, Gold, Diamond, Ruby, Green Sapphire, Sapphire)
2. `redpower:screwdriver` (Rotates logic gates and configures machines/tubes)
3. `redpower:paint_brush` (16 dye colors; colors tubes and bags)
4. `redpower:canvas_bag` (16 dyed colors, 27-slot handheld portable inventory)
5. `redpower:sickle` (Wood, Stone, Iron, Gold, Diamond, Ruby, Green Sapphire, Sapphire)
6. `redpower:wool_card`
7. `redpower:ruby`
8. `redpower:sapphire`
9. `redpower:green_sapphire`
10. `redpower:nikolite`
11. `redpower:red_alloy_ingot`
12. `redpower:blue_alloy_ingot`
13. `redpower:brass_ingot`
14. `redpower:copper_ingot`
15. `redpower:tin_ingot`
16. `redpower:silver_ingot`
17. `redpower:tungsten_ingot`
18. `redpower:silicon_wafer`
19. `redpower:red_doped_wafer`
20. `redpower:blue_doped_wafer`
21. `redpower:floppy_disk`
22. `redpower:canvas`
23. `redpower:battery` (BT rechargeable)
24. `redpower:logic_tile` (Subtypes: Inverter, AND, OR, NAND, NOR, XOR, XNOR, Buffer, Mux, Repeater, Timer, Counter, Sequencer, Pulse Former, Toggle Latch, RS Latch, Transparent Latch, State Cell, Synchronizer, Light Sensor)
25. `redpower:wire_red_alloy`
26. `redpower:wire_insulated` (16 colors)
27. `redpower:wire_bundled`
28. `redpower:wire_jacketed`
29. `redpower:ribbon_cable`

---

## 6. Key Crafting & Smelting Recipes

### 6.1 Alloy Furnace (Brick-like 2x2 internal recipe grid)
* **Red Alloy Ingot:** 4x Redstone Dust + 1x Copper Ingot (or Iron Ingot).
* **Blue Alloy Ingot:** 4x Nikolite Dust + 1x Silver Ingot.
* **Brass Ingot:** 3x Copper Ingot + 1x Tin Ingot $\rightarrow$ 4x Brass Ingot.
* **Silicon Boule:** 8x Sand + 8x Coal $\rightarrow$ Silicon Boule.
* **Silicon Wafer:** Sliced from Silicon Boule using Handsaw.
* **Red-Doped Wafer:** 4x Redstone + 1x Silicon Wafer in Alloy Furnace.
* **Blue-Doped Wafer:** 4x Nikolite + 1x Silicon Wafer in Alloy Furnace.

### 6.2 Logic & Wiring
* **Red Alloy Wire (x12):** 1x Red Alloy Ingot + 1x Paper / Stone strip.
* **Insulated Wire:** 1x Red Alloy Wire + 1x Wool (matching color).
* **Bundled Cable:** 1x Insulated Wire of multiple colors + String.
* **Handsaw:** 2x Sticks + 1x Ingot/Gem + 2x Iron Ingots.
* **Screwdriver:** 1x Iron Ingot + 1x Stick.
* **Logic Gates:** Red-doped wafers, stone base plates, red alloy wires, and redstone torches laid out in geometric patterns matching their logic function.

---

## 7. Recreation Implementation Notes for Rekkit (Rust)
1. **Multi-Part / Voxel Model:** Use a sub-voxel octree or bitmask (8 octants, 12 edges, 6 faces) inside block entities to support multi-part microblocks, wires, and tiles in a single coordinate.
2. **Tube Pathfinding:** Implement an asynchronous breadth-first search (BFS) on tube connection graphs with caching. Tube queries must simulate destination slot reservation so two items don't route to the same single open slot simultaneously.
3. **Frame Movement Physics:** When a Frame Motor triggers, collect all rigidly connected blocks via DFS. Validate destination bounding volume: if any target block is blocked, cancel movement with zero loss. If clear, atomically update chunk block entities and broadcast translation packets.
4. **6502 Emulator:** Integrate an embedded 6502 CPU core crate (e.g. `oxur` or a lightweight instruction interpreter) with memory-mapped MMIO bounded execution (e.g. max 50,000 cycles per tick) to prevent server hangs.
