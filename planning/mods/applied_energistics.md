# Applied Energistics (AE1) - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** Applied Energistics 1 (AE1), specifically rv14.e (final canonical AE1 release by AlgorithmX2 for Minecraft 1.5.2 / 1.6.4).
* **Author:** AlgorithmX2.
* **Core Philosophy:** Digital item dematerialization into Matter Energy (ME). Unlike AE2, **AE1 does NOT have channel restrictions** (infinite devices per cable network). Features multiblock Molecular Assembler Chambers (MAC), centralized ME Controllers, byte-based ME Storage Cells, and seamless external inventory integration.

---

## 2. Core Network Architecture (The ME Network)

### 2.1 The Channel-Free Bus Topology
* In AE1, all ME devices connect along a unified cable bus without channel limitations.
* A single ME Controller coordinates the entire network, providing power distribution and item index caching.
* Multiblock Controller: ME Controllers can be built as multiblock rectangular cubes/clusters to increase internal energy storage and processing capacity.

### 2.2 Energy Units (AE) & Power Integration
* **Internal Energy:** Applied Energistics Units (AE).
* **Cross-Power Conversion:**
  * $1 \text{ EU} = 2 \text{ AE}$ (IndustrialCraft 2).
  * $1 \text{ MJ} = 5 \text{ AE}$ (BuildCraft).
* **Energy Consumption:**
  * Every device attached to the network has an idle power drain (e.g. Cables: 0.1 AE/t; Terminals: 1.0 AE/t; Storage Bus: 1.0 AE/t; Drives: 2.0 AE/t).
  * Active operations (inserting, extracting, autocrafting) consume additional energy bursts.
  * If network power reaches zero, the entire network shuts down into an unpowered state until recharged.
* **Vibration Chamber:** 1-block generator that consumes solid fuels (Coal, Wood, Blaze Rods) to produce raw AE power.

---

## 3. Storage System: The Byte & Type Matrix

### 3.1 Storage Cell Mechanics
AE1 stores items digitized into digital Storage Cells placed inside an **ME Drive** (holds 10 cells) or **ME Chest** (holds 1 cell).
* **The 63-Type Constraint:**
  * Every storage cell, regardless of byte tier, can store a maximum of **63 distinct item types**.
* **Byte Consumption Formula:**
  $$\text{Bytes Used} = (\text{Types Used} \times \text{BytesPerType}) + \left\lfloor \frac{\text{Total Items}}{8} \right\rfloor$$
  * Storing a new distinct item type reserves an initial chunk of bytes.
  * Storing 8 items of an existing type consumes 1 byte.
* **Cell Capacities:**
  1. **1k Storage Cell:** 1,024 bytes. Stores up to 63 types and ~8,192 single-type items.
  2. **4k Storage Cell:** 4,096 bytes. Stores up to 63 types and ~32,768 single-type items.
  3. **16k Storage Cell:** 16,384 bytes. Stores up to 63 types and ~131,072 single-type items.
  4. **64k Storage Cell:** 65,536 bytes. Stores up to 63 types and ~524,288 single-type items.
* **Cell Partitioning & Formatting:**
  * Cells can be formatted in the **ME Cell Workbench** with ghost filters to only accept pre-designated item types, preventing random junk from filling high-value cells.

---

## 4. Network Devices & Terminals

### 4.1 Terminals
* **ME Access Terminal:**
  * Displays the combined contents of all storage cells and attached external inventories.
  * Real-time search bar (by name, mod name with `@`, tooltip).
  * Sorting modes: Item count, Item name, Inventory ID.
  * Instant insertion and extraction.
* **ME Crafting Terminal:**
  * Integrates a standard 3x3 crafting grid directly above the network inventory view.
  * Pulls required crafting ingredients directly from ME storage.
  * Retains leftover items; shifts output directly back into the network with Shift+Click.
* **ME Pattern Terminal:**
  * Encodes recipe patterns onto **Blank Patterns** for use in automated crafting. Supports standard 3x3 crafting recipes and machine processing patterns.

### 4.2 Logistics & Automation Buses
* **ME Storage Bus:**
  * Attached to the face of any external inventory (Chests, Barrels, IC2/BC machines).
  * Treats external inventories as native ME storage.
  * Configurable priority (items are pushed into high-priority storage buses first).
* **ME Import Bus:**
  * Actively pulls items out of adjacent inventories and injects them into the ME network.
  * Tiers: Basic Import Bus, Precision Import Bus (exact damage/stack filtering), Fuzzy Import Bus (ore-dictionary and durability percentage matching).
* **ME Export Bus:**
  * Actively exports configured items from the ME network into adjacent inventories or machines.
  * Tiers: Basic, Precision, Fuzzy.
* **ME Level Emitter:**
  * Emits a redstone signal when a specific item's quantity in the network is greater than, equal to, or less than a specified threshold.
* **ME Interface:**
  * Dual-purpose block: buffers items for external automation pipes/tubes to extract, and instantly ingests any items pushed into it.

### 4.3 Wireless Connectivity
* **Wireless Access Point:** Broadcasts ME network signal wirelessly.
* **Wireless Booster:** Increases the wireless transmission radius (base 16 blocks; up to 128+ blocks with boosters).
* **Wireless Terminal:** Portable handheld Access Terminal allowing full network interaction across the wireless radius.

---

## 5. Automated Crafting: Molecular Assembler Chamber (MAC)

### 5.1 Multiblock Architecture
Unlike AE2's standalone single-block assemblers, AE1 features a grand multiblock structure:
* **Frame & Walls:** Built using **ME Assembler Containment Walls** and **ME Heat Vents**.
* **Dimensions:** Any rectangular cuboid from 3x3x3 up to 16x16x16.
* **Interior Blocks:** The hollow internal space is filled with:
  * **ME Pattern Providers:** Holds encoded crafting recipe patterns.
  * **Crafting CPUs:** Determines how many concurrent crafting jobs and operations the multiblock can execute simultaneously.

### 5.2 Crafting Order Mechanics
* Players order an uncrafted or low-stock item via the ME Crafting Terminal.
* The network calculates the entire dependency tree of missing sub-components.
* Displays a confirmation GUI detailing: Missing Items, Items to be Used, and Estimated Operations.
* When confirmed, the MAC dispatches crafting operations instantaneously, using internal virtual crafting cycles and outputting directly into the network.

---

## 6. Early-Game Processing & World Resources

### 6.1 Quartz Grindstone & Wooden Crank
* **Function:** Electricity-free early-game ore doubling and dust pulverization.
* **Operation:** Player places an ore or crystal in the Grindstone and right-clicks the Wooden Crank to rotate it 8 times.
* **Yield:** 1 Ore $\rightarrow$ 2 Dusts; Certus Quartz $\rightarrow$ Certus Quartz Dust.

### 6.2 Minerals & Materials
* **Certus Quartz Ore:** Spawns naturally underground (Y=16 to Y=64). Yields Certus Quartz Crystals.
* **Charged Certus Quartz:** Spawns rarely; emits faint particles.
* **Nether Quartz:** Vanilla integration.
* **Silicon:** Obtained by smelting Quartz Dust in a furnace. Essential for ME Storage Processors and Logic circuits.
* **Quartz Glass:** Smelted from 4x Quartz Dust + 5x Glass. Highly blast-resistant transparent glass used in containment walls and drives.

---

## 7. Complete Registry

### Blocks
1. `ae1:controller` (ME Controller)
2. `ae1:drive` (ME Drive)
3. `ae1:chest` (ME Chest)
4. `ae1:access_terminal`
5. `ae1:crafting_terminal`
6. `ae1:pattern_terminal`
7. `ae1:interface`
8. `ae1:cell_workbench`
9. `ae1:mac_containment_wall`
10. `ae1:mac_heat_vent`
11. `ae1:mac_pattern_provider`
12. `ae1:mac_crafting_cpu`
13. `ae1:storage_bus`
14. `ae1:import_bus_basic`, `ae1:import_bus_precision`, `ae1:import_bus_fuzzy`
15. `ae1:export_bus_basic`, `ae1:export_bus_precision`, `ae1:export_bus_fuzzy`
16. `ae1:level_emitter`
17. `ae1:wireless_access_point`
18. `ae1:vibration_chamber`
19. `ae1:grindstone`
20. `ae1:crank`
21. `ae1:quartz_glass`
22. `ae1:dark_glass`
23. `ae1:ore_certus_quartz`
24. `ae1:cable` (Standard, Covered, 16 colored variants)

### Items
1. `ae1:storage_cell_1k`, `ae1:storage_cell_4k`, `ae1:storage_cell_16k`, `ae1:storage_cell_64k`
2. `ae1:storage_segment_1k`, `ae1:storage_segment_4k`, `ae1:storage_segment_16k`, `ae1:storage_segment_64k`
3. `ae1:wireless_terminal`
4. `ae1:wireless_booster`
5. `ae1:blank_pattern`
6. `ae1:encoded_pattern`
7. `ae1:certus_quartz_crystal`
8. `ae1:charged_certus_quartz_crystal`
9. `ae1:quartz_dust`
10. `ae1:silicon`
11. `ae1:basic_processor`, `ae1:advanced_processor`
12. `ae1:quartz_wrench`
13. `ae1:quartz_cutting_knife`

---

## 8. Recreation Implementation Notes for Rekkit (Rust)
1. **Unified Item Hash Index:**
   * Maintain a single `HashMap<ItemKey, u64>` at the ME Controller actor level.
   * On insert/extract requests, route directly through cell byte allocators, deduct/add from `HashMap`, and publish delta broadcast packets to all currently open terminal subscribers.
2. **Recursive Recipe Dependency Solver:**
   * Model recipes as directed acyclic graphs (DAG).
   * Implement topological evaluation to calculate missing materials, reject cycles, and queue intermediate craft steps sequentially into the MAC Crafting CPU queue.
3. **No-Channel Scalability:**
   * Since AE1 omits channel routing, cable networks can simply be represented as disjoint connected graph components (Union-Find / Disjoint-Set), allowing $O(1)$ device membership checks.
