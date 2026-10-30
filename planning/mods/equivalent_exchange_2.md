# Equivalent Exchange 2 (EE2) - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** Equivalent Exchange 2 (EE2) v1.4.6.7 for Minecraft 1.2.5 (the peak original canonical version created by Pahimar prior to community remakes like ProjectE or the divergent EE3).
* **Author:** Pahimar.
* **Core Philosophy:** Energy-Mass Equivalence ($E = mc^2$). No matter can be created or destroyed from nothing; all matter is evaluated on a universal scalar: **Energy-Matter Currency (EMC)**. Materials can be freely converted into any other material possessing equal or lower EMC.

---

## 2. Universal Economy: EMC (Energy-Matter Currency)

### 2.1 The Principle of Equivalence
* Every basic element and crafted item has an immutable EMC value derived from the sum of its recipe ingredients.
* Items can be converted to raw EMC via "learning" in the Transmutation Tablet, dissolution in Energy Condensers, or digestion in Antimatter Relays.
* Transmutation is 100% efficient: 8,192 Cobblestone (1 EMC each) converts exactly into 1 Diamond (8,192 EMC).

### 2.2 Canonical Baseline EMC Values
| Item / Block | EMC Value | Notes |
|---|---:|---|
| Cobblestone, Dirt, Sand, Netherrack | 1 | Base unit of mass |
| Gravel, Stick | 4 | |
| Wood Log / Planks | 32 / 8 | |
| Redstone Dust | 64 | Base alchemical spark |
| Obsidian | 64 | |
| Iron Ingot | 256 | $1 \text{ Iron} = 4 \text{ Redstone} = 256 \text{ Cobble}$ |
| Glowstone Dust | 384 | |
| Gold Ingot | 2,048 | $1 \text{ Gold} = 8 \text{ Iron}$ |
| Diamond | 8,192 | $1 \text{ Diamond} = 4 \text{ Gold} = 32 \text{ Iron}$ |
| Lapis Lazuli | 864 | |
| Clay | 64 | |
| Gunpowder | 192 | |
| Ender Pearl | 1,024 | |
| Nether Star | 139,264 | |
| Alchemical Coal | 512 | Infused coal fuel |
| Mobius Fuel | 2,048 | Compressed alchemical fuel |
| Aeternalis Fuel | 8,192 | Peak fuel tier ($1 \text{ Aeternalis} = 1 \text{ Diamond}$) |
| Dark Matter | 139,264 | Crafted from 8 Aeternalis + 1 Diamond Block |
| Red Matter | 466,944 | Crafted from 3 Dark Matter + 6 Aeternalis |

---

## 3. Storage, Transmutation & Generation Blocks

### 3.1 Transmutation Tablet
* **Function:** The master alchemical interface. Available as a placed Block and as a handheld Portable Item.
* **Mechanics:**
  * **Knowledge Base:** Tracks "learned" items per player UUID. An item is permanently unlocked when placed into the central flame/burn slot.
  * **Input/Burn Slot:** Consumes items and deposits their full EMC value into the player's personal alchemical pool.
  * **Output Matrix:** Displays grid of all learned items whose cost $\le$ available EMC pool. Clicking an item extracts it and deducts its EMC.
  * **Search & Filter:** Allows instant text filtering and sorting by EMC cost.
  * **Klein Star Slot:** Allows charging or discharging handheld Klein Star batteries.

### 3.2 Energy Condenser
* **Internal Inventory:** 72 item slots + 1 Target Lock slot.
* **Operation:**
  * When an item is placed in the top-left Target slot, the condenser locks to that item.
  * Any items placed or piped into the 72 inventory slots are instantly destroyed and their EMC is added to an internal accumulator.
  * Once accumulated EMC $\ge \text{Target EMC}$, a new copy of the target item is generated in the inventory.
  * Extremely popular for mass duplication loops (e.g. condensing Cobblestone into Diamonds).

### 3.3 Energy Collectors (MK1, MK2, MK3)
* **Mechanics:** Collects ambient photons (sunlight or artificial light $\ge 8$) and synthesizes raw EMC without material input.
  * **Collector MK1:** Generates 4 EMC/sec. Light buffer: 10,000 EMC.
  * **Collector MK2:** Generates 12 EMC/sec. Light buffer: 30,000 EMC.
  * **Collector MK3:** Generates 40 EMC/sec. Light buffer: 60,000 EMC.
* **Transmutation Chamber:** When fuel items (Coal, Alchemical Coal, Mobius Fuel) or matter items (Cobble, Iron, Gold) are placed inside, the generated light EMC upgrades them to the next tier once sufficient EMC is reached.
* **Adjacency Transfer:** Automatically pushes generated EMC into touching Antimatter Relays or Energy Condensers.

### 3.4 Antimatter Relays (MK1, MK2, MK3)
* **Mechanics:** Acts as an alchemical power substation and charge multiplier.
  * Accepts items to dissolve into EMC.
  * Pulls EMC from adjacent Energy Collectors and routes it into touching Condensers or charging items placed in its charge slots.
  * Buffer sizes: MK1 (100k EMC), MK2 (1M EMC), MK3 (10M EMC).

### 3.5 Alchemical Chest & Interdiction Torch
* **Alchemical Chest:** A 104-slot high-capacity chest that accepts any item and allows rings/amulets to operate actively inside it.
* **Interdiction Torch:** Emits light level 15. Creates a 9x9 repulsive forcefield that actively repels all hostile mobs and slows projectiles away from the torch.

### 3.6 Dark Matter & Red Matter Furnaces
* **Dark Matter Furnace:** Smelts items at 8x speed. Has a chance to double smelting yields.
* **Red Matter Furnace:** Smelts items at 20x speed. Has two smelting slots and a higher chance to double output drops.

---

## 4. Alchemical Tools, Weapons & Armor

### 4.1 Covalence Dust & Philosopher's Stone
* **Low Covalence Dust:** 8 Cobblestone + 1 Charcoal $\rightarrow$ 40 Dust.
* **Medium Covalence Dust:** 1 Iron Ingot + 1 Redstone $\rightarrow$ 40 Dust.
* **High Covalence Dust:** 1 Diamond + 1 Coal $\rightarrow$ 40 Dust.
* **Philosopher's Stone:**
  * The fundamental alchemical tool.
  * **In-World Transmutation (Right-Click):** Transmutes blocks in world: Stone $\leftrightarrow$ Cobblestone, Sand $\leftrightarrow$ Dirt, Grass $\leftrightarrow$ Sand, etc.
  * **Charge Levels ('V' key):** 0 to 4. Increases transmutation area from 1 block up to 9x9 or deep columns.
  * **Mode Switch ('G' key):** Cube, Panel, Line.
  * **Portable Crafting Grid ('C' key):** Opens a 3x3 crafting grid anywhere.
  * **Grid Transmutation:** In crafting grid: 4 Iron $\rightarrow$ 1 Gold; 4 Gold $\rightarrow$ 1 Diamond; and reversible with dusts.

### 4.2 Dark Matter & Red Matter Tools
* **Durability:** Completely unbreakable (durability is infinite).
* **Charge System:** Charged with the `V` key (Shift+V discharges). DM has 2 charge tiers; RM has 3 charge tiers.
* **Tools:**
  * **Pickaxes:** Charges dig massive 3x3x3 volumes. Right-click toggles Ore-Smelting/Mining filter mode (mines only ores within a radius).
  * **Shovels:** Digs massive volumes of gravel/sand/dirt instantly. Right-click creates flat leveling planes.
  * **Axes:** Fells entire trees (all connected wood/leaves) in a single blow.
  * **Swords:** Sweeping area-of-effect strikes damaging all hostile mobs in range.
  * **Shears:** Shears all sheep and shears leaves in a large sphere.
* **Hybrid Endgame Multi-Tools:**
  * **Red Matter Morning Star:** Combines RM Pickaxe, Shovel, and Hammer. Right-click pulverizes any stone/dirt block instantly. Max charge clears an entire 16x16x16 subterranean cavern in one click.
  * **Red Matter Katar:** Combines RM Axe, Sword, Hoe, and Shears. Right-click strips all foliage and wood in a 20-block sphere. `C` key triggers a radial death shockwave on all hostile mobs.

### 4.3 Dark Matter & Red Matter Armor
* **Dark Matter Armor:** Grants 90% total damage absorption, immune to all fire and lava damage, nullifies fall damage.
* **Red Matter Armor:** Grants **100% total invulnerability** to almost all forms of damage in Minecraft (explosions, void, suffocation, drowning, mob attacks). Pressing `C` unleashes an explosive radial knockback shockwave that blasts away all surrounding entities.

---

## 5. Rings, Amulets, Talismans & Artifacts

### 5.1 Flight & Motion
* **Swiftwolf's Rending Gale (SWRG):**
  * Grants true creative flight.
  * Pushes away projectiles (arrows, ghast fireballs).
  * In stormy weather, right-click strikes targeted locations with lethal lightning bolts.
  * Consumes fuel (Redstone, Glowstone, or Klein Star EMC) from inventory.

### 5.2 Elemental Control
* **Volcanite Amulet:** Places infinite lava source blocks (consumes EMC); absorbs/evaporates water; protects holder from lava.
* **Evertide Amulet:** Places infinite water source blocks without consuming fuel; creates water shields.
* **Ring of Ignition:** Fire immunity, right-click ignites tracks of fire, shoots fireballs.
* **Zero Ring:** Freezes water into ice, lava into obsidian, generates snow covers.
* **Harvest Goddess Band:** Dramatically accelerates crop growth in a 9x9 area; automatically harvests and replants mature crops.
* **Archangel's Smite:** Fires a rapid-fire salvo of homing magic arrows that track down hostile mobs without using arrows.
* **Black Hole Band:** Magnetically vacuums all dropped item entities within a 15-block radius directly into the player's inventory.
* **Void Ring:** Combines Black Hole Band and alchemical compression; teleports the player on right-click.
* **Watch of Flowing Time:**
  * Accelerates tile entity tick rates in a 9x9x9 area.
  * Placed on a Dark Matter Pedestal, speeds up neighboring machines/collectors/furnaces up to **256x speed**.
* **Talisman of Repair:** Passively repairs the durability of all damaged tools and armor in the player's inventory over time without consuming resources.
* **Destruction Catalyst & Hyperkinetic Lens:** Fires explosive alchemical projectiles into stone strata; vaporizes stone and condenses all dropped ore items into a single concentrated loot ball.

### 5.3 Klein Stars (Portable EMC Batteries)
* Used to power rings, amulets, and alchemical armor.
* **Klein Star Ein:** 50,000 EMC.
* **Klein Star Zwei:** 200,000 EMC.
* **Klein Star Drei:** 800,000 EMC.
* **Klein Star Vier:** 3,200,000 EMC.
* **Klein Star Sphere:** 12,800,000 EMC.
* **Klein Star Omega:** 51,200,000 EMC.

---

## 6. Complete Registry

### Blocks
1. `ee2:transmutation_tablet`
2. `ee2:energy_condenser`
3. `ee2:energy_collector_mk1`, `ee2:energy_collector_mk2`, `ee2:energy_collector_mk3`
4. `ee2:antimatter_relay_mk1`, `ee2:antimatter_relay_mk2`, `ee2:antimatter_relay_mk3`
5. `ee2:alchemical_chest`
6. `ee2:interdiction_torch`
7. `ee2:dark_matter_furnace`
8. `ee2:red_matter_furnace`
9. `ee2:dark_matter_block`
10. `ee2:red_matter_block`
11. `ee2:dm_pedestal`

### Items
1. `ee2:philosophers_stone`
2. `ee2:portable_transmutation_tablet`
3. `ee2:alchemical_bag` (16 dyed colors)
4. `ee2:covalence_dust_low`, `ee2:covalence_dust_medium`, `ee2:covalence_dust_high`
5. `ee2:alchemical_coal`
6. `ee2:mobius_fuel`
7. `ee2:aeternalis_fuel`
8. `ee2:dark_matter`
9. `ee2:red_matter`
10. `ee2:klein_star_ein` through `ee2:klein_star_omega`
11. `ee2:dm_pickaxe`, `ee2:dm_axe`, `ee2:dm_shovel`, `ee2:dm_sword`, `ee2:dm_hoe`, `ee2:dm_shears`, `ee2:dm_hammer`
12. `ee2:rm_pickaxe`, `ee2:rm_axe`, `ee2:rm_shovel`, `ee2:rm_sword`, `ee2:rm_hoe`, `ee2:rm_shears`, `ee2:rm_hammer`
13. `ee2:rm_katar`
14. `ee2:rm_morning_star`
15. `ee2:dm_helmet`, `ee2:dm_chestplate`, `ee2:dm_leggings`, `ee2:dm_boots`
16. `ee2:rm_helmet`, `ee2:rm_chestplate`, `ee2:rm_leggings`, `ee2:rm_boots`
17. `ee2:swiftwolfs_rending_gale`
18. `ee2:ring_of_ignition`
19. `ee2:zero_ring`
20. `ee2:harvest_goddess_band`
21. `ee2:archangels_smite`
22. `ee2:black_hole_band`
23. `ee2:void_ring`
24. `ee2:watch_of_flowing_time`
25. `ee2:volcanite_amulet`
26. `ee2:evertide_amulet`
27. `ee2:mercurial_eye`
28. `ee2:talisman_of_repair`
29. `ee2:destruction_catalyst`
30. `ee2:hyperkinetic_lens`
31. `ee2:body_stone`, `ee2:soul_stone`, `ee2:life_stone`, `ee2:mind_stone`

---

## 7. Recreation Implementation Notes for Rekkit (Rust)
1. **Dynamic EMC Graph Solver:**
   * Compute EMC values at server startup by traversing all crafting, smelting, and machine recipes via an iterative topological sort or relaxation algorithm.
   * Detect and resolve infinite duplication exploits (where recipe output EMC > sum of input ingredient EMC).
2. **Player Alchemical State:**
   * Persist a `HashSet<ItemKey>` of learned knowledge and a `u64` EMC accumulator per player.
3. **Chunk Transmutation & Performance:**
   * Large AOE tools (Morning Star, Katar, Destruction Catalyst) must perform volume raycasting and batch chunk block-state updates in a single tick to avoid network desync and packet spam.
4. **Tile Entity Multiplier (Watch of Flowing Time):**
   * Limit extra tick dispatches per server tick with a configurable cap (e.g. max 16x extra ticks in Rust) to prevent denial-of-service server thread lockups.
