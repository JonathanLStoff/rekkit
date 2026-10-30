# Forestry - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** Forestry v2.2.8.4 for Minecraft 1.5.2 / 1.4.7 (and v1.4.8.7 for Minecraft 1.2.5).
* **Author:** SirSengir.
* **Core Philosophy:** Biological simulation, Mendelian genetics, apiculture (bee breeding), arboriculture (tree cross-pollination), ecological agriculture, biofuel production chains, and specialized automated backpacks.

---

## 2. Apiculture (The Bee Genetics System)

### 2.1 The Caste System & Life Cycle
* **Castes:**
  * **Princess:** Single female bee carrying genetic code.
  * **Drone:** Male fertilizing bee; stacks up to 64.
  * **Queen:** Formed when a Princess mates with a Drone in an Apiary/Bee House.
* **Life Cycle:**
  * A Queen lives for a fixed number of life ticks (determined by its Lifespan chromosome).
  * Periodically produces Honey Combs and specialty drops into the apiary inventory.
  * On death, the Queen disintegrates and produces:
    * **Exactly 1 Princess** (inheriting combined alleles).
    * **1 to 4 Drones** (inheriting combined alleles).
    * Specialty comb drops.

### 2.2 Chromosomes & Mendelian Inheritance
Each bee carries two alleles for every trait: **Active (Phenotype)** and **Inactive (Recessive/Genotype)**.
1. **Species:** (e.g. Forest, Meadows, Common, Cultivated, Noble, Majestic, Imperial, Diligent, Unweary, Industrious, Tropical, Wintry, Marshy, Demonic, Ender).
2. **Lifespan:** Shortest (10 min), Short (20 min), Normal (30 min), Long (40 min), Longest (70 min).
3. **Production Speed:** Slowest, Slower, Normal, Fast, Faster, Fastest.
4. **Pollination (Flowering):** Rate at which neighboring flowers or leaves mutate.
5. **Flower Provider:** Vanilla Flowers, Wheat, Vines, Netherwart, Jungle, Cacti.
6. **Climate Tolerance:**
  * **Temperature:** Neutral, Up 1-3, Down 1-3, Both 1-3.
  * **Humidity:** Neutral, Up 1-2, Down 1-2, Both 1-2.
7. **Diurnal:** Can fly during daylight.
8. **Nocturnal:** Can fly at night.
9. **Cave / Tolerant Flyer:** Can fly underground without direct sky access; can fly in rain.
10. **Special Effects:** Aggressive (damages players), Heroic (damages hostile mobs), Poisonous (poisons players), Radioactive, Creeper (explodes), Glacial (freezes area).

### 2.3 Housing & The Alveary Multiblock
* **Bee House:** Cheap wooden box. Queen produces combs 3x faster, but bees **cannot mutate** and always produce identical offspring.
* **Apiary:** Standard breeding structure. Holds 3 Frame slots:
  * Untreated Frame (increases production, shortens lifespan).
  * Impregnated Frame.
  * Proven Frame.
* **Alveary:**
  * 3x3x3 multiblock composed of 27 **Alveary Blocks**.
  * Extreme breeding speed and high mutation rates.
  * Modular upgrades replace exterior blocks:
    * **Alveary Heater:** Raises temperature to breed tropical bees in cold biomes.
    * **Alveary Fan:** Cools temperature to breed wintry bees.
    * **Alveary Hygroregulator:** Adjusts humidity.
    * **Alveary Swarmer:** Spawns extra princesses.
    * **Alveary Rain Shield:** Shields bees from rain.

---

## 3. Arboriculture & Tree Breeding
* Bees visiting leaf blocks cross-pollinate trees, mutating leaves into hybrid varieties.
* Grafting with a **Grafter** guarantees sapling drops from mutated leaves.
* Produces exotic wood species (Teak, Balsa, Mahogany, Kapok, Chestnut, Cherry, Ebony, Baobab, Sequoia) and harvestable fruits (Plums, Walnuts, Papayas, Dates).

---

## 4. Automated Agriculture & Farming Systems

### 4.1 Automated Farm Multiblocks
* Powered by BuildCraft MJ or Forestry engines.
* Requires continuous supplies of **Water**, **Fertilizer** (Apatite-based), and soil substrates (Humus, Bog Earth, Sand, Dirt).
* Farm Types:
  * **Arboretum / Tree Farm:** Clears land, places Humus, plants saplings, cuts mature trees, harvests wood and apples.
  * **Peat Bog:** Places Bog Earth; harvests Bog Earth once it matures into harvestable **Peat** (high-density solid fuel).
  * **Crop Farm:** Auto-farms Wheat, Potatoes, Carrots, Melons, Pumpkins.
  * **Infernal Farm:** Auto-farms Netherwart and Soul Sand.
  * **Succulent Farm:** Auto-farms Cactus and Sugarcane.

---

## 5. Industrial Machinery & Biofuel Production

### 5.1 Biofuel Pipeline
1. **Squeezer:**
   * Squeezes Seeds $\rightarrow$ **Seed Oil** (used for Impregnated Casings).
   * Squeezes Apples / Fruits $\rightarrow$ **Fruit Juice**.
2. **Fermenter:**
   * Combines organic biomass (Saplings, Wheat, Sugarcane) with Water, Fruit Juice, or Honey.
   * Produces **Biomass** (green liquid).
3. **Still:**
   * Distills Biomass using MJ power into **Biofuel / Ethanol** at a 10:3 ratio.
   * High-energy combustible fuel for Combustion and Biogas engines.

### 5.2 Processing & Energy Machines
* **Centrifuge:**
  * Spins Honey Combs into Honey Drops, Beeswax, Pollen Clusters, Propolis, and specialty drops.
* **Carpenter:**
  * Combines crafting materials with liquids (Water, Seed Oil, Creosote, Honey).
  * Crafts Impregnated Casing, Impregnated Sticks, Woven Backpacks, and Crates.
* **Thermionic Fabricator:**
  * Melts Sand / Glass with MJ into Molten Glass.
  * Crafts **Electron Tubes** (Copper, Tin, Bronze, Iron, Gold, Diamond) used for farm circuit boards.
* **Engines:**
  * **Clockwork Engine:** Hand-wound engine for early-game kinetic power.
  * **Peat-Fired Engine:** Burns Peat to generate 1–2 MJ/t; produces **Ash** byproduct.
  * **Biogas Engine:** Burns Biomass or Liquid Honey with a lava kickstart; outputs up to 5 MJ/t.

---

## 6. Logistics: Specialized Backpacks
Backpacks automatically scoop up matching item drops directly into their dedicated internal inventories:
* **Miner's Backpack:** Ores, ingots, gems, dusts, coal, redstone.
* **Digger's Backpack:** Dirt, cobblestone, stone, sand, gravel, flint.
* **Forester's Backpack:** Logs, planks, saplings, leaves, flowers, seeds.
* **Hunter's Backpack:** Mob drops (bones, arrows, gunpowder, rotten flesh, leather, ender pearls).
* **Builder's Backpack:** Bricks, stairs, slabs, glass, torches, building materials.
* **Adventurer's Backpack:** Tools, weapons, specialty items.
* **Capacity:** Standard Backpack (15 slots) $\rightarrow$ Woven Backpack (45 slots, upgraded via Carpenter).

---

## 7. World Generation & Resources
* **Apatite Ore:** Spawns in giant, expansive seams at high elevations (Y=40 to Y=80). Yields Apatite gems, the core ingredient in Fertilizer.
* **Copper & Tin Ores:** Standard world gen veins.
* **Wild Beehives:**
  * Forest Hive (Forest biomes).
  * Meadows Hive (Plains biomes).
  * Modest Hive (Desert biomes).
  * Tropical Hive (Jungle biomes).
  * Wintry Hive (Taiga/Tundra biomes).
  * Marshy Hive (Swamp biomes).
  * Resonant / Ender Hive (The End).

---

## 8. Complete Registry

### Blocks
1. `forestry:bee_house`
2. `forestry:apiary`
3. `forestry:alveary_plain`
4. `forestry:alveary_heater`
5. `forestry:alveary_fan`
6. `forestry:alveary_hygroregulator`
7. `forestry:alveary_swarmer`
8. `forestry:engine_clockwork`
9. `forestry:engine_peat`
10. `forestry:engine_biogas`
11. `forestry:machine_centrifuge`
12. `forestry:machine_squeezer`
13. `forestry:machine_carpenter`
14. `forestry:machine_fermenter`
15. `forestry:machine_still`
16. `forestry:machine_fabricator`
17. `forestry:farm_plain`, `forestry:farm_gearbox`, `forestry:farm_hatch`, `forestry:farm_valve`
18. `forestry:ore_apatite`
19. `forestry:ore_copper`, `forestry:ore_tin`
20. `forestry:beehive_*` (Forest, Meadows, Modest, Tropical, Wintry, Marshy, Ender)

### Items
1. `forestry:bee_queen`, `forestry:bee_princess`, `forestry:bee_drone`
2. `forestry:scoop` (Tool to safely harvest wild beehives)
3. `forestry:grafter` (Tool to guarantee sapling drops)
4. `forestry:honey_comb_*` (Honey, Cocoa, Simmering, Frozen, Dripping, Silky, etc.)
5. `forestry:honey_drop`
6. `forestry:beeswax`
7. `forestry:royal_jelly`
8. `forestry:pollen_cluster`
9. `forestry:propolis`
10. `forestry:fertilizer`
11. `forestry:humus`
12. `forestry:bog_earth`
13. `forestry:peat`
14. `forestry:ash`
15. `forestry:apatite`
16. `forestry:impregnated_casing`
17. `forestry:impregnated_stick`
18. `forestry:electron_tube_*` (Copper, Tin, Bronze, Iron, Gold, Diamond)
19. `forestry:backpack_miner`, `forestry:backpack_digger`, `forestry:backpack_forester`, `forestry:backpack_hunter`, `forestry:backpack_builder`, `forestry:backpack_adventurer` (and woven T2 upgrades)

---

## 9. Recreation Implementation Notes for Rekkit (Rust)
1. **Genetics Engine:**
   * Represent individual bees as bit-packed or struct records containing pairs of alleles: `struct Chromosome<T> { active: T, inactive: T }`.
   * On Queen death, evaluate a 2x2 Punnett square crossing the Queen's alleles with the Drone's alleles. Apply species mutation probability matrices (e.g. `Forest` + `Meadows` $\rightarrow 15\%$ `Common`).
2. **Backpack Pickup Interceptor:**
   * In the server item pickup event hook, check player inventory for backpacks before placing items in standard inventory slots. If an item matches a backpack's inclusion filter, route the stack directly into the backpack's serialized inventory.
3. **Alveary Multiblock Formation:**
   * Validate 3x3x3 bounds: verify 27 valid alveary blocks, detect special upgrade blocks, and aggregate climate modification offsets into a single tick controller.
