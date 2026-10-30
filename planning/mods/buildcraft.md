# BuildCraft - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** BuildCraft 3.x (v3.1.5 - v3.4.3 for Minecraft 1.2.5 / 1.4.7).
* **Authors:** SpaceToad, CovertJaguar, SirSengir, Krapht.
* **Core Philosophy:** Industrial logistics through visible mechanical action, moving parts, piping networks, autonomous construction, and physical resource extraction powered by kinetic Minecraft Joules (MJ).

---

## 2. Core Systems & Mechanics

### 2.1 Energy System: Minecraft Joules (MJ)
* **Energy Unit:** 1 MJ (Minecraft Joule). Standard conversion: $1 \text{ MJ} \approx 2.5 \text{ EU}$ (IndustrialCraft) or $10 \text{ RF}$ (Thermal Expansion).
* **Engines:**
  1. **Redstone Engine (Wood):**
     * Power: Outputs 0.05 to 1 MJ/s.
     * Fuel: Pure redstone pulse or continuous redstone signal.
     * Explosion: **Cannot explode.**
     * Purpose: Powers Wooden Transport Pipes to extract single items; powers Pumps and Flood Gates slowly.
  2. **Stirling Engine (Stone):**
     * Power: Generates 1 MJ/t (20 MJ/s).
     * Fuel: Solid combustibles (Coal, Charcoal, Wood).
     * Temperature States: Blue (warming up) $\rightarrow$ Green (optimal) $\rightarrow$ Yellow (hot) $\rightarrow$ Red (critical).
     * Hazard: **Can explode** if allowed to run continuously at Red heat without an energy sink draining its power.
  3. **Combustion Engine (Iron):**
     * Power: Generates up to 5–6 MJ/t.
     * Fuel: Oil (3 MJ/t) or refined Fuel (6 MJ/t).
     * Coolant: **Requires continuous Water supply.**
     * Hazard: Consumes water to cool cylinders. If water runs dry while fueled, temperature spikes from Green $\rightarrow$ Yellow $\rightarrow$ Red, culminating in a catastrophic explosive blast (equivalent to 4-8 TNT, destroys surrounding machines and terrain).

### 2.2 Pipe Network Mechanics
Pipes operate on a 1-block grid, routing Items, Liquids, or Energy.
* **Item Transport Pipes:**
  * **Wooden Pipe:** Extracts items from adjacent inventories. Must be powered by an Engine or Autarchic Gate. Does not connect to other wooden pipes.
  * **Cobblestone Pipe:** Basic item conveyor. **Does not connect to Stone Pipes**, allowing two parallel adjacent pipe lines without crossover.
  * **Stone Pipe:** Standard item conveyor with lower drag than cobblestone. **Does not connect to Cobblestone.**
  * **Iron Pipe:** Directional one-way flow. Has 5 inputs and 1 output face. Output face is toggled using a Wrench or Redstone signal.
  * **Golden Pipe:** Accelerates items traveling through it when powered by Redstone.
  * **Diamond Pipe:** Advanced 6-way sorting. The GUI shows 6 colored rows corresponding to the 6 pipe faces. Items matching the row's filter slots are routed exclusively through that face. Unmatched items route through unfiltered directions.
  * **Obsidian Pipe:** Sucks up dropped item entities from the world within a 1.5-block radius when placed in-world; can be powered by an engine to increase suction range.
  * **Void Pipe:** Deletes any item entering it.
  * **Sandstone Pipe:** Connects only to other pipes; will not connect to inventories or machines.
  * **Emerald Pipe:** Filtered extraction pipe that pulls items round-robin from an inventory.
* **Overflow / Dropping Behavior:**
  * If an item reaches an end point with no valid path, or arrives at an inventory that has become full, **the item pops out of the pipe onto the ground as a floating entity**.
* **Waterproof (Fluid) Pipes:**
  * Crafted by coating standard pipes with Pipe Sealant (from Cactus Green).
  * Moves fluids (Water, Oil, Fuel, Lava) measured in Millibuckets (mB).
  * Types: Wooden Waterproof (extracts liquids), Cobble/Stone Waterproof, Iron Waterproof, Golden Waterproof, Void Waterproof.
* **Conductive (Power) Pipes:**
  * Crafted by infusing pipes with Redstone Dust.
  * Carries MJ.
  * Types: Wooden Conductive (extracts MJ from engines), Cobblestone Conductive (high energy loss/meter), Stone Conductive (medium loss), Golden Conductive (low energy loss).
  * Capacity: Can overheat and explode if capacity is exceeded without drain.

### 2.3 Pipe Gates & Logic (Assembly System)
* **Gates:** Mounted directly onto pipes to add logic and telemetry.
  * **Basic Gate:** 1 conditional slot (If [Condition] Then [Action]).
  * **Iron Gate:** 2 conditional slots.
  * **Gold Gate:** 4 conditional slots.
  * **Diamond Gate:** 8 conditional slots.
  * **Autarchic Gate Variants:** Adds "Pulse" action, enabling wooden pipes to extract items/fluids without an external redstone engine.
* **Pipe Wires:** 4 colored wires (Red, Blue, Green, Yellow) run along pipe surfaces to transmit gate signals over pipe networks.
* **Conditions:** Inventory Empty, Space in Inventory, Inventory Full, Pipe Empty, Items in Pipe, Energy Stored, Engine Safe, Engine Overheating.
* **Actions:** Redstone High, Redstone Low, Emit Wire Signal, Energy Pulsing.

---

## 3. Machines & Heavy Industry

### 3.1 Quarry
* **Frame Construction:**
  * If 3 or 4 Landmarks are placed in a rectangle and activated, the Quarry uses their perimeter (up to 64x64 blocks).
  * If no landmarks are used, defaults to an 11x11 or 9x9 area.
  * Constructs an orange-and-black structural frame around the area using automated laser heads.
* **Excavation Mechanism:**
  * A gantry arm moves along X and Z axes across the frame ceiling, lowering a drill head down the Y axis.
  * Mined blocks are converted to item entities and ejected immediately into adjacent chests or transport pipes attached to the Quarry block.
  * Replaces mined liquid blocks with Cobblestone or leaves empty shafts.
  * Drills continuously until Bedrock is encountered across the entire footprint.
* **Energy Draw:** Scales from 2 MJ/t to 10+ MJ/t. Higher power drastically increases arm speed.

### 3.2 Mining Well
* **Operation:** 1-block footprint vertical borehole.
* When powered by MJ, extends Mining Pipe blocks straight down until it hits Bedrock or lava. Mined blocks are pumped out the top.

### 3.3 Pump
* **Operation:** Deploys a flexible suction pipe straight down into liquid reservoirs (Lava or Water lakes, Oil deposits).
* Pumps out contiguous fluid source blocks, replacing them with Stone or removing them, and fills connected Waterproof Pipes.

### 3.4 Refinery
* **Multiblock Construction:** 2 animated glass distillation cylinders.
* **Refining Process:**
  * Consumes raw Oil and MJ.
  * Distills 2 buckets of Oil into 1 bucket of refined yellow Fuel.
  * Fuel produces more than 2x the energy output per bucket in Combustion Engines compared to raw Oil.

### 3.5 Auto Workbench
* **Function:** 3x3 recipe template inventory + 9-slot material buffer.
* When items are piped or placed in, it automatically crafts items according to the template whenever matching ingredients are present in its buffer.

### 3.6 Assembly Table & Lasers
* **Laser (Block):** Consumes MJ (4 MJ/t per laser) and fires visible red laser beams at nearby Assembly Tables or Advanced Workbenches.
* **Assembly Table:**
  * Hosts high-tech recipes (Redstone Chips, Iron/Gold/Diamond Chips, Logic Gates, Pipe Wires, Pipe Facades).
  * Requires cumulative MJ input provided by one or more aiming Lasers.

### 3.7 Architect Table & Builder / Filler
* **Landmarks:** Blue laser beacons defining a 3D box.
* **Architect Table:** Scans the 3D volume enclosed by landmarks and writes the block blueprints to a Blueprint or Template item.
* **Builder:** Consumes MJ and reads a Blueprint or Template; places blocks into the designated volume using materials supplied from an adjacent chest.
* **Filler:** Pre-programmed building patterns: Clear, Fill, Pyramid, Stairs, Box, Wall, Cylinder. Clears or places blocks using items in its internal inventory.

---

## 4. World Generation & Fluids
* **Crude Oil:**
  * Generates on the surface as black oil springs/spouts (geysers shooting oil blocks high into the air) or large subterranean oil reservoirs beneath desert and ocean biomes.
  * Fluid properties: High viscosity, slows entities, flammable.

---

## 5. Complete Registry

### Blocks
1. `buildcraft:engine_wood` (Redstone Engine)
2. `buildcraft:engine_stone` (Stirling Engine)
3. `buildcraft:engine_iron` (Combustion Engine)
4. `buildcraft:pipe_items_wood`
5. `buildcraft:pipe_items_cobblestone`
6. `buildcraft:pipe_items_stone`
7. `buildcraft:pipe_items_iron`
8. `buildcraft:pipe_items_golden`
9. `buildcraft:pipe_items_diamond`
10. `buildcraft:pipe_items_obsidian`
11. `buildcraft:pipe_items_void`
12. `buildcraft:pipe_items_sandstone`
13. `buildcraft:pipe_items_emerald`
14. `buildcraft:pipe_fluids_*` (Waterproof variants of above)
15. `buildcraft:pipe_power_*` (Conductive variants: Wood, Stone, Cobble, Gold)
16. `buildcraft:quarry`
17. `buildcraft:mining_well`
18. `buildcraft:pump`
19. `buildcraft:refinery`
20. `buildcraft:auto_workbench`
21. `buildcraft:assembly_table`
22. `buildcraft:laser`
23. `buildcraft:filler`
24. `buildcraft:builder`
25. `buildcraft:architect_table`
26. `buildcraft:flood_gate`
27. `buildcraft:frame_block` (Yellow/Black striped scaffold)
28. `buildcraft:mining_pipe`
29. `buildcraft:oil_fluid_block`

### Items
1. `buildcraft:wrench`
2. `buildcraft:landmark`
3. `buildcraft:pipe_sealant`
4. `buildcraft:redstone_chipset`
5. `buildcraft:iron_chipset`
6. `buildcraft:golden_chipset`
7. `buildcraft:diamond_chipset`
8. `buildcraft:pulsating_chipset`
9. `buildcraft:gate_basic`
10. `buildcraft:gate_iron`
11. `buildcraft:gate_gold`
12. `buildcraft:gate_diamond`
13. `buildcraft:pipe_wire` (Red, Blue, Green, Yellow)
14. `buildcraft:blueprint`
15. `buildcraft:template`
16. `buildcraft:pipe_facade` (Hides pipes inside decorative block textures)
17. `buildcraft:bucket_oil`
18. `buildcraft:bucket_fuel`

---

## 6. Key Recipes & Math

### 6.1 Basic Components
* **Wrench:** 3x Iron Ingot in diagonal shape.
* **Pipe Sealant:** Smelt 1x Cactus Green.
* **Landmark:** 1x Redstone Torch + 1x Lapis Lazuli.
* **Item Pipes (x8):** 2x Glass + 1x Material (Wood, Cobble, Stone, Iron, Gold, Diamond, Obsidian, Void).
* **Waterproof Pipe:** 1x Item Pipe + 1x Pipe Sealant.
* **Conductive Pipe:** 1x Item Pipe + 1x Redstone Dust.

### 6.2 Machines & Engines
* **Redstone Engine:** 3x Wood Planks + 2x Sticks + 1x Piston + 1x Wooden Gear.
* **Stirling Engine:** 3x Cobblestone + 2x Glass + 1x Piston + 1x Stone Gear.
* **Combustion Engine:** 3x Iron Ingot + 2x Glass + 1x Piston + 1x Iron Gear.
* **Quarry:** 3x Iron Gear + 2x Gold Gear + 2x Diamond Gear + 1x Redstone Dust + 1x Diamond Pickaxe.
* **Assembly Table:** 4x Obsidian + 2x Diamond + 2x Redstone + 1x Diamond Workbench.
* **Laser:** 4x Obsidian + 2x Redstone + 2x Diamond + 1x Golden Gear.

---

## 7. Recreation Implementation Notes for Rekkit (Rust)
1. **Kinetic Pipe Flow:** Pipe items have exact float coordinates `[0.0, 1.0]` along a pipe segment. Update item trajectories each tick; handle intersection junction decisions deterministically (check output filters, choose random valid non-reverse direction if unguided).
2. **Quarry State Machine:** Build as an asynchronous actor:
   * State `ConstructingFrame`: Spawn frame blocks along perimeter using traveling builder beam.
   * State `Digging`: Move gantry coordinates $(X, Z)$, extend drill tip down $Y$, call world block break events, buffer drops into Quarry tile entity.
3. **Engine Heat Curves:** Model each engine with thermal energy $T$ ($0 \le T \le 1000$ mB heat):
   $$T_{t+1} = T_t + \Delta_{\text{combustion}} - \Delta_{\text{cooling}} - \Delta_{\text{power\_out}}$$
   When $T > 800$, state becomes Red. If $T \ge 1000$, trigger block explosion with radius 3.5.
