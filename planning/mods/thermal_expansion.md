# Thermal Expansion - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** Thermal Expansion 2.x (TE2, v2.4.6.0 for Minecraft 1.4.7 / 1.5.2) and TE1 (v1.0 for Minecraft 1.2.5).
* **Authors:** KingLemming, CptSpaceToad, Leming, skyboy (Team CoFH).
* **Core Philosophy:** Clean, explosion-free industrial engineering, intelligent internal automation, modular side input/output configuration, lossless energy conduits, wireless cross-dimensional Tesseracts, and by-product metallurgy.

---

## 2. The CoFH Interaction & Energy System

### 2.1 The Side Configuration Matrix
Thermal Expansion revolutionized tech mods by replacing cumbersome external extraction pipes with internal machine logic:
* **Color-Coded Sides GUI:**
  * Every machine interface displays a 3D isometric or unfolded cube showing all 6 faces (Top, Bottom, Front, Back, Left, Right).
  * Clicking a face cycles its IO role:
    * **Blue:** Primary item/fluid input.
    * **Red:** Primary output.
    * **Green:** Secondary input.
    * **Yellow:** Secondary/by-product output.
    * **Orange:** Omni-directional (both Input and Output).
    * **Grey / Open:** Disabled face (no connection).
* **Auto-Ejection:** Machines automatically push completed items directly into adjacent inventories, pipes, or conduits without requiring external wooden extraction pipes or wooden engines.

### 2.2 Energy & Distribution
* **Energy Standard:** Minecraft Joules (MJ) (native in TE2; later RF in TE3).
* **Redstone Energy Conduit:**
  * Liquid Destabilized Redstone encased in Lead-reinforced glass.
  * Solved BuildCraft pipe loop explosion bugs. Transmits power reliably up to 100 MJ/t per connection with a fixed 5% energy loss penalty.
* **Liquid Conduit:** High-throughput pressurized fluid transfer.
* **Redstone Energy Cell:**
  * Stores up to 600,000 MJ.
  * Fully customizable IO rates: configure slider from 1 to 100 MJ/t input/output per face.
  * Visual charge meter displayed on block texture.

### 2.3 Cross-Dimensional Tesseracts
* **Mechanics:** Enables wireless, zero-latency, cross-dimensional teleportation between paired frequencies.
  * **Energy Tesseract:** Teleports power across dimensions (subject to a 25% energy transfer loss).
  * **Liquid Tesseract:** Teleports fluids with 0% loss.
  * **Item Tesseract:** Teleports item stacks with 0% loss.
* **Frequency Channels:** Supports 1,024 public frequencies, plus private secure frequencies locked to player UUIDs.

---

## 3. Machine Processing Roster

### 3.1 Primary Processing
* **Powered Furnace:**
  * High-efficiency electric furnace.
  * Consumes 20 MJ per smelt at 2 MJ/t (10 ticks per operation).
* **Pulverizer:**
  * Grinds raw ores into **2 Dusts** (ore doubling) + a **10% chance of a secondary byproduct**:
    * Iron Ore $\rightarrow$ 2 Iron Dust + 10% Ferrous (Nickel) Dust.
    * Gold Ore $\rightarrow$ 2 Gold Dust + 10% Rich Slag.
    * Copper Ore $\rightarrow$ 2 Copper Dust + 10% Gold Dust.
    * Tin Ore $\rightarrow$ 2 Tin Dust + 10% Slag.
* **Sawmill:**
  * Converts 1 Wood Log $\rightarrow$ **6 Wood Planks** (vs 4 in vanilla) + 1 Sawdust + occasional Tree Bark.
* **Induction Smelter:**
  * Smelts ores directly into ingots using metallurgical fluxes:
    * 1 Ore + 1 Sand $\rightarrow$ 2 Ingots + 1 Slag.
    * 1 Ore + 1 Rich Slag $\rightarrow$ **3 Ingots** (triple yield!) + 1 Regular Slag.

### 3.2 Fluid & Material Synthesis
* **Magma Crucible:**
  * Melts solid blocks into liquid fluids:
    * Netherrack / Cobblestone $\rightarrow$ Molten Lava (1,000 mB).
    * Redstone Dust $\rightarrow$ Destabilized Redstone (100 mB).
    * Glowstone Dust $\rightarrow$ Energized Glowstone (250 mB).
    * Ender Pearl $\rightarrow$ Resonant Ender (250 mB).
* **Liquid Transposer:**
  * **Emptying Mode:** Extracts fluid from buckets, cans, or capsules into internal tanks.
  * **Filling Mode:** Infuses fluids into empty substrates:
    * Empty Conduit Frame + Destabilized Redstone $\rightarrow$ Redstone Energy Conduit.
    * Empty Tesseract Frame + Resonant Ender $\rightarrow$ Resonant Tesseract Frame.
* **Glacial Precipitator:**
  * Consumes Water to form Snowballs (500 mB), Snow Blocks (1,000 mB), or Ice Blocks (1,000 mB) with minimal MJ.
* **Igneous Extruder:**
  * Passive volcanic material synthesizer requiring **zero energy**:
    * Water + Lava $\rightarrow$ Infinite Cobblestone (consumes neither fluid).
    * Water + Lava $\rightarrow$ Smooth Stone (consumes 1,000 mB Water).
    * Water + Lava $\rightarrow$ Obsidian (consumes 1,000 mB Water + 1,000 mB Lava).

---

## 4. Power Generation (Engines)
* **Steam Engine:**
  * Consumes solid combustibles (Coal, Charcoal) and Water.
  * Generates 2 MJ/t continuously without risk of explosion.
* **Magmatic Engine:**
  * Consumes Lava (18,000 MJ per bucket) or Liquid Fuel.
  * Generates 4 MJ/t continuously. Features internal heat-dissipation shutoff to eliminate explosion hazards entirely.

---

## 5. Metallurgy, Alloys & World Resources
* **Ores:**
  * Copper Ore (Y=40 to Y=75).
  * Tin Ore (Y=20 to Y=55).
  * Silver Ore (Y=5 to Y=30).
  * Lead Ore (Y=10 to Y=35).
  * Ferrous / Nickel Ore (Y=5 to Y=20).
* **Alloys (Induction Smelter or Crafting Dusts):**
  * **Invar:** 2x Iron + 1x Ferrous (Nickel) $\rightarrow$ 3x Invar Ingot. Highly heat-resistant.
  * **Electrum:** 1x Gold + 1x Silver $\rightarrow$ 2x Electrum Ingot. High electrical conductivity.
  * **Bronze:** 3x Copper + 1x Tin $\rightarrow$ 4x Bronze Ingot.
* **Hardened Glass:**
  * Crafted from Lead Dust + Obsidian in the Induction Smelter.
  * Completely blast-resistant transparent glass that deflects TNT and creeper blasts.

---

## 6. Complete Registry

### Blocks
1. `thermalexpansion:machine_furnace` (Powered Furnace)
2. `thermalexpansion:machine_pulverizer`
3. `thermalexpansion:machine_sawmill`
4. `thermalexpansion:machine_smelter` (Induction Smelter)
5. `thermalexpansion:machine_crucible` (Magma Crucible)
6. `thermalexpansion:machine_transposer` (Liquid Transposer)
7. `thermalexpansion:machine_precipitator` (Glacial Precipitator)
8. `thermalexpansion:machine_extruder` (Igneous Extruder)
9. `thermalexpansion:machine_infuser` (Energetic Infuser)
10. `thermalexpansion:engine_steam`
11. `thermalexpansion:engine_magmatic`
12. `thermalexpansion:conduit_energy`
13. `thermalexpansion:conduit_liquid`
14. `thermalexpansion:storage_energy_cell`
15. `thermalexpansion:tesseract_energy`
16. `thermalexpansion:tesseract_liquid`
17. `thermalexpansion:tesseract_item`
18. `thermalexpansion:hardened_glass`
19. `thermalexpansion:ore_copper`, `thermalexpansion:ore_tin`, `thermalexpansion:ore_silver`, `thermalexpansion:ore_lead`, `thermalexpansion:ore_ferrous`

### Items
1. `thermalexpansion:crescent_hammer`
2. `thermalexpansion:multimeter`
3. `thermalexpansion:ingot_copper`, `thermalexpansion:ingot_tin`, `thermalexpansion:ingot_silver`, `thermalexpansion:ingot_lead`, `thermalexpansion:ingot_ferrous`
4. `thermalexpansion:ingot_invar`, `thermalexpansion:ingot_electrum`, `thermalexpansion:ingot_bronze`
5. `thermalexpansion:dust_pulverized_*` (Copper, Tin, Silver, Lead, Ferrous, Iron, Gold)
6. `thermalexpansion:slag`, `thermalexpansion:slag_rich`
7. `thermalexpansion:sawdust`, `thermalexpansion:compressed_sawdust`
8. `thermalexpansion:frame_conduit`
9. `thermalexpansion:frame_tesseract`
10. `thermalexpansion:bucket_redstone`, `thermalexpansion:bucket_glowstone`, `thermalexpansion:bucket_ender`

---

## 7. Key Recipes
* **Crescent Hammer:** 3x Iron Ingot + 1x Stick.
* **Machine Frame:** 4x Iron Ingot + 4x Glass + 1x Gold Gear.
* **Redstone Energy Cell:** 4x Lead Ingot + 4x Electrum Ingot + 1x Diamond + 1x Redstone Energy Core.
* **Energy Tesseract:** 4x Ender Pearl + 4x Silver Ingot + 1x Diamond.

---

## 8. Recreation Implementation Notes for Rekkit (Rust)
1. **Side Routing Automata:**
   * Model each machine face as an IO port with an enum `SideMode { Disabled, PrimaryInput, PrimaryOutput, SecondaryInput, SecondaryOutput, Omni }`.
   * On machine completion tick, query adjacent tile entity capabilities and push items/fluids directly into valid slots.
2. **Global Tesseract Bus:**
   * Maintain a central registry `TesseractNetwork { energy_channels: HashMap<FreqId, EnergyPool>, item_channels: ..., liquid_channels: ... }`.
   * Pushing into one tesseract instantaneously makes items/fluids available to all tesseracts listening on the same frequency ID.
3. **Lossless Energy Conduits:**
   * Energy Conduits form a grid. Calculate total power pushed vs total requested; distribute power evenly across connected sinks in a single graph pass.
