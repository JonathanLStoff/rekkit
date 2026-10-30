# IndustrialCraft 2 (IC2) - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** IC2 Classic / IC2 v1.95 - v1.118 for Minecraft 1.2.5 / 1.4.7 (pre-Experimental rework; prior to item durability plates, forge hammers, and cutter tools).
* **Authors:** Alblaka, Player, RichardG, GregoriusT.
* **Core Philosophy:** Industrial energy grids, voltage-tiered distribution, ore doubling via mechanical pulverization, hazard-rich nuclear power, advanced metallurgy, and powered armor sets (NanoSuit and QuantumSuit).

---

## 2. Electrical Grid & Energy System (EU)

### 2.1 Voltage Tiers & Explosion Mechanics
IndustrialCraft 2 uses **Energy Units (EU)**. Electricity is packetized: a cable transmits $N$ packets of size $V$ EU per tick.
* **Voltage Tiers:**
  * **Low Voltage (LV):** Up to 32 EU/packet.
  * **Medium Voltage (MV):** Up to 128 EU/packet.
  * **High Voltage (HV):** Up to 512 EU/packet.
  * **Extreme Voltage (EV):** Up to 2048 EU/packet.
* **Overvoltage Overload:**
  * If a machine receives a packet higher than its rated voltage tier (e.g. 128 EU sent into a 32 EU Macerator), **the machine immediately explodes**, destroying itself and dealing damage to surrounding blocks and players.
  * Transformer upgrades or step-down transformers must be placed before machine clusters.

### 2.2 Conductors & Cable Loss
Energy is lost as packets travel through cable blocks according to cable resistance:
$$\text{Loss per block} = \text{Cable Loss Rate}$$
* **Tin Cable (Ultra-Low):**
  * Max Voltage: 32 EU/t.
  * Loss: 0.025 EU per meter (1 EU lost every 40 blocks). Extremely low loss for micro-power.
* **Copper Cable:**
  * Max Voltage: 32 EU/t.
  * Loss: 0.20 EU per meter (1 EU lost every 5 blocks). Insulated with 1x Rubber.
* **Gold Cable (2x Insulated):**
  * Max Voltage: 128 EU/t.
  * Loss: 0.40 EU per meter (1 EU lost every 2.5 blocks).
* **Iron / HV Cable (3x Insulated):**
  * Max Voltage: 512 EU/t.
  * Loss: 0.80 EU per meter (1 EU lost every 1.25 blocks).
* **Glass Fibre Cable:**
  * Max Voltage: 512 EU/t.
  * Loss: 0.025 EU per meter (1 EU lost every 40 blocks). Premium conductor made with Diamonds and Silver/Electrum/Glass.
* **Electrocution:** Uninsulated energized cables shock any unarmored creature touching them, dealing damage proportional to the voltage packet.

### 2.3 Energy Storage Units & Transformers
* **BatBox:** Stores 40,000 EU. Input/Output: 32 EU/t (LV).
* **MFE (Multi-Functional Energy storage):** Stores 600,000 EU. Input/Output: 128 EU/t (MV).
* **MFSU (Multi-Functional Storage Unit):** Stores 10,000,000 EU. Input/Output: 512 EU/t (HV).
* **Transformers (LV, MV, HV):**
  * Have 5 low-voltage faces (1 dot) and 1 high-voltage face (3 dots / striped).
  * Step-down mode (default): Converts high packets into 4 low packets per tick.
  * Step-up mode (redstone active): Converts 4 low packets into 1 high packet per tick.

---

## 3. Power Generation

### 3.1 Basic Generators
* **Generator:** Burns solid combustibles (Coal = 4,000 EU at 10 EU/t; Wood = 250 EU).
* **Geothermal Generator:** Consumes Lava (1 bucket / capsule = 20,000 EU at 20 EU/t).
* **Solar Panel:** Generates 1 EU/t during clear day with direct sky exposure. 0 EU/t at night or rain.
* **Water Mill:**
  * Passive mode: 0.01 EU/t per adjacent stationary water source block.
  * Active mode: Consumes Water Buckets/Capsules to output 2 EU/t.
* **Wind Mill:** Generates 0 to 4+ EU/t depending on altitude ($Y=64$ to $Y=250$), weather (storms double output), and unobstructed 9x9x9 clearance.
  * Hazard: If wind speed exceeds 12 EU/t in high storms, wind turbine breaks into iron ingots.

### 3.2 Nuclear Reactor
* **Architecture:** Core block expandable with up to 6 Reactor Chambers for a total 6x9 (54 slots) internal component grid.
* **Core Components:**
  * **Uranium Fuel Cells:** Emits energy pulses and heat. Each adjacent uranium cell triggers additional pulses ($P = 1 + \text{adjacent cells}$).
  * **Coolant Cells (10k, 30k, 60k):** Absorbs heat before deteriorating.
  * **Heat Vents:** Dissipates heat from the reactor hull or directly from adjacent components into the air.
  * **Heat Exchangers:** Balances heat between hull and adjacent components.
  * **Reactor Plating:** Increases hull heat capacity and reduces radiation radius.
  * **Condensators:** Consumes Redstone or Lapis to instantly absorb immense heat spikes.
* **Meltdown Mechanics:**
  * Reactor core tracks internal hull heat ($0$ to $10,000$).
  * At 4,000 heat: Surrounding flammable blocks ignite.
  * At 5,000 heat: Surrounding stone converts into Lava.
  * At 8,500 heat: Water around reactor evaporates instantly.
  * At 10,000 heat: **Catastrophic Meltdown.** Enormous explosion (equivalent to 40–80 TNT), destroys bedrock/casing, leaves radioactive fallout, creates lava pools, and applies Radiation debuff to players within 50 blocks.

---

## 4. Machines & Industrial Processing

### 4.1 Ore Doubling & Primary Processing
* **Iron Furnace:** 20% faster than vanilla furnace; burns fuel 25% more efficiently.
* **Electric Furnace:** 100% electric; consumes 3 EU/t for 130 ticks (390 EU per operation).
* **Macerator:**
  * Pulverizes 1 raw ore into **2 Crushed Ores / Dusts**, effectively doubling mining yields.
  * Crushes Cobblestone $\rightarrow$ Sand, Sand $\rightarrow$ Clay Dust, Gravel $\rightarrow$ Flint, Coal $\rightarrow$ Coal Dust, Plants $\rightarrow$ Bio Chaff.
  * Power: 2 EU/t for 400 ticks (800 EU).
* **Extractor:**
  * Extracts 3x Rubber from 1x Sticky Resin (smelting in a furnace only yields 1x).
  * Extracts 1x Rubber from 1x Rubber Wood.
* **Compressor:**
  * Compresses materials into dense industrial plates and gems:
    * Refined Iron $\rightarrow$ Advanced Alloy.
    * Carbon Mesh $\rightarrow$ Carbon Plate.
    * Compressed Coal Ball + Obsidian + Brick $\rightarrow$ Industrial Diamond.
    * 9x Snowballs $\rightarrow$ Ice Block.

### 4.2 Advanced & Synthesis Machinery
* **Induction Furnace:**
  * Dual-slot high-efficiency electric smelter with a Heat Gauge ($0\% - 100\%$).
  * Smelts continuously to retain heat; at 100% heat, smelts 2 items every ~20 ticks at minimal EU cost.
* **Recycler:**
  * Rapidly consumes any junk item (1 EU/t, 45 ticks).
  * Has a 12.5% (1 in 8) chance to output **Scrap**.
* **Mass Fabricator (Matter Generator):**
  * The pinnacle of IC2 endgame.
  * Generates **UU-Matter** (Universal Unique Matter), a pink alchemical liquid/matter that crafts any vanilla resource (diamonds, iridium, ores, grass).
  * Power: Requires 1,000,000 EU per UU-Matter.
  * Scrap Amplifier: Consuming 1 Scrap provides 45,000 amplifier points, reducing the net power cost to ~166,666 EU per UU-Matter.
* **Canning Machine:** Cans food into Tin Cans, packs Uranium fuel into empty fuel cells, and fills Hydration/Coolant Cells.
* **Teleporter:**
  * Consumes EU to teleport players/animals between two linked pads.
  * Formula: $\text{EU Cost} = \text{Weight} \times (\text{Distance} + 10)^2 / 100$.
* **Terraformer:**
  * Modifies the local biome using terraformer blueprints: Cultivation, Irrigation, Chilling, Desertification, Flatificator.

---

## 5. Tools, Armor & Weapons

### 5.1 Power Tools
* **Mining Drill & Diamond Drill:** Powered by internal rechargeable battery (30,000 EU). Digs instantly through stone and dirt; never breaks as long as charged.
* **Chainsaw:** Cuts down trees, shears sheep, slices leaves instantly.
* **Electric Wrench:** Configures and dismantles IC2 machines safely with 100% drop rate (lossless mode).
* **Mining Laser:**
  * Modes toggled via keybind: Mining (straight beam), Low-Focus (short range), Long-Range, Horizontal, Super-Heat (auto-smelts mined blocks), Explosive (causes explosive detonation on block impact).
* **Nano Saber:** Lightsaber-style melee weapon. Consumes EU while turned on; deals 20 points of true damage (10 hearts) per swing.

### 5.2 Armor Sets
* **Rubber Boots:** Fully cushions falling damage, converting it into durability loss.
* **Composite Armor:** Mid-tier armor made of Advanced Alloys.
* **NanoSuit Armor:**
  * High-tech armor storing 1,000,000 EU per piece.
  * Provides 90% total damage absorption as long as EU charge remains.
* **QuantumSuit Armor:**
  * Endgame powered armor storing 10,000,000 EU per piece.
  * **Quantum Helmet:** Underwater rebreather (consumes EU for oxygen), cures status effects, built-in night vision.
  * **Quantum Bodyarmor:** Absorbs 100% of physical damage, fires personal forcefield, built-in Jetpack mode.
  * **Quantum Leggings:** Super-sprint boost (speeds up player movement up to 300%).
  * **Quantum Boots:** High-jump boost (9 blocks vertical leap) with complete fall-damage nullification.

---

## 6. World Generation & Materials
* **Copper Ore:** Generates Y=10 to Y=70, vein size 8-15.
* **Tin Ore:** Generates Y=0 to Y=40, vein size 4-8.
* **Uranium Ore:** Generates Y=0 to Y=64, vein size 1-4. Mined with Iron Pickaxe or better.
* **Rubber Tree:**
  * Features distinctive pointed leaves and orange sap spots on trunk blocks.
  * Harvesting sap spot with a Treetap yields 1-3 Sticky Resin without destroying the trunk.

---

## 7. Complete Registry

### Blocks
1. `ic2:ore_copper`
2. `ic2:ore_tin`
3. `ic2:ore_uranium`
4. `ic2:rubber_wood`
5. `ic2:rubber_leaves`
6. `ic2:rubber_sapling`
7. `ic2:generator`
8. `ic2:geothermal_generator`
9. `ic2:solar_panel`
10. `ic2:water_mill`
11. `ic2:wind_mill`
12. `ic2:nuclear_reactor`
13. `ic2:reactor_chamber`
14. `ic2:batbox`
15. `ic2:mfe`
16. `ic2:mfsu`
17. `ic2:lv_transformer`
18. `ic2:mv_transformer`
19. `ic2:hv_transformer`
20. `ic2:cable_tin`
21. `ic2:cable_copper`
22. `ic2:cable_gold`
23. `ic2:cable_iron`
24. `ic2:cable_glass_fibre`
25. `ic2:iron_furnace`
26. `ic2:electric_furnace`
27. `ic2:macerator`
28. `ic2:extractor`
29. `ic2:compressor`
30. `ic2:recycler`
31. `ic2:induction_furnace`
32. `ic2:canning_machine`
33. `ic2:mass_fabricator`
34. `ic2:teleporter`
35. `ic2:terraformer`
36. `ic2:tesla_coil`
37. `ic2:machine_block`
38. `ic2:advanced_machine_block`

### Items
1. `ic2:treetap`
2. `ic2:electric_treetap`
3. `ic2:wrench`
4. `ic2:electric_wrench`
5. `ic2:mining_drill`
6. `ic2:diamond_drill`
7. `ic2:chainsaw`
8. `ic2:nano_saber`
9. `ic2:mining_laser`
10. `ic2:battery_re` (10k EU)
11. `ic2:energy_crystal` (100k EU)
12. `ic2:lapotron_crystal` (1M EU)
13. `ic2:batpack` (60k EU)
14. `ic2:lappack` (300k EU)
15. `ic2:sticky_resin`
16. `ic2:rubber`
17. `ic2:refined_iron`
18. `ic2:bronze_ingot`
19. `ic2:advanced_alloy`
20. `ic2:carbon_fibre`
21. `ic2:carbon_mesh`
22. `ic2:carbon_plate`
23. `ic2:scrap`
24. `ic2:scrap_box`
25. `ic2:uu_matter`
26. `ic2:uranium_drop`
27. `ic2:uranium_fuel_cell`
28. `ic2:coolant_cell_10k`
29. `ic2:nano_helmet`, `ic2:nano_chestplate`, `ic2:nano_leggings`, `ic2:nano_boots`
30. `ic2:quantum_helmet`, `ic2:quantum_chestplate`, `ic2:quantum_leggings`, `ic2:quantum_boots`

---

## 8. Recreation Implementation Notes for Rekkit (Rust)
1. **Packet-Based Power Graph:** Maintain an energy grid solver. Do not simulate individual electron particles; compute per-tick breadth-first-search spanning trees of connected cables with cumulative voltage drops. Emit exploding event when a node receives packet voltage > max rated tier.
2. **Machine Lossless Dismantle:** When broken with pickaxes, machines drop basic `machine_block` (losing circuits/upgrades). When broken with an Electric Wrench in lossless mode, drop the exact machine block with internal state.
3. **Reactor Grid Simulation:** Reactor logic can run as a separate 1-tick subsystem: update uranium fuel pulses, transfer heat to neighbor cooling elements, evaluate meltdown thresholds, and radiate damage if unshielded.
