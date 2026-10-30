# Railcraft - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** Railcraft v6.x - v8.x for Minecraft 1.4.7 / 1.5.2 / 1.2.5.
* **Author:** CovertJaguar.
* **Core Philosophy:** True railway engineering and heavy thermodynamic logistics. Comprehensive track networks, high-speed rail lines, articulated coupled trains, fluid/item tank carts, signaling systems, blast furnaces, coke ovens, and high-pressure steam boilers.

---

## 2. Thermodynamics & Multiblock Heavy Industry

### 2.1 Coke Oven
* **Construction:** 3x3x3 solid cube of **Coke Oven Bricks** (27 blocks, hollow interior is not required; multiblock forms on placement).
* **Processing:**
  * Bakes Coal or Wood Logs without external energy over 36 seconds.
  * $1 \text{ Coal} \rightarrow 1 \text{ Coal Coke} + 500\text{ mB Creosote Oil}$.
  * $1 \text{ Wood Log} \rightarrow 1 \text{ Charcoal} + 250\text{ mB Creosote Oil}$.
* **Creosote Oil:** Essential liquid used to treat Wooden Planks into Wooden Ties for railway beds.

### 2.2 Blast Furnace
* **Construction:** 3x3x4 hollow structure of **Blast Furnace Bricks** (34 bricks surrounding a 1x1x2 internal cavity).
* **Processing:**
  * Converts Iron Ingots into **Steel Ingots** and Slag.
  * Fuel: Requires high-carbon fuels: **Coal Coke** or Charcoal (standard coal cannot achieve blast furnace temperatures).
  * Cycle time: ~64 seconds per steel ingot.

### 2.3 Steam Boilers (Low Pressure & High Pressure)
* **Construction:**
  * Base Layer: **Boiler Firebox** (Solid-Fueled or Liquid-Fueled). Dimensions: 1x1, 2x2, or 3x3.
  * Upper Layers: **Steam Boiler Tanks** (Low Pressure or High Pressure) stacked 1 to 3 layers high (total multiblock heights: 1x1x2 up to 3x3x4).
* **Thermal Dynamics:**
  * Tracks internal heat ($0^\circ\text{C}$ to $1,000^\circ\text{C}$).
  * Fuel efficiency increases dramatically at higher temperatures.
  * Consumes fuel continuously to build and maintain heat; converts water into Steam ($1\text{ mB Water} \rightarrow 160\text{ mB Steam}$).
* **Boiler Explosion Hazard:**
  * If a boiler runs dry of water while at high temperature ($>100^\circ\text{C}$), it enters an overheated dry state.
  * **If water is pumped into an overheated dry boiler, flash-boiling steam triggers an instantaneous catastrophic explosion**, obliterating the multiblock and shattering adjacent infrastructure.

### 2.4 Steam Engines
Consumes steam produced by Boilers to generate kinetic power (MJ):
* **Hobbyist's Steam Engine:** Consumes 10 mB/t steam $\rightarrow$ Outputs up to 1.6 MJ/t.
* **Commercial Steam Engine:** Consumes 20 mB/t steam $\rightarrow$ Outputs up to 4.0 MJ/t.
* **Industrial Steam Engine:** Consumes 40 mB/t steam $\rightarrow$ Outputs up to 8.0 MJ/t.

### 2.5 Iron & Steel Fluid Tanks
* **Construction:** Hollow cuboid formed from Iron or Steel Tank Wall, Gauge, and Valve blocks (3x3x4 up to 9x9x8).
* **Capacity:** Stores millions of millibuckets of fluids safely with visible gauge windows.

---

## 3. Track Infrastructure & Railway Physics

### 3.1 Track Classes
1. **Wooden Track:** Cheap early-game track; cart top speed capped at 50% vanilla.
2. **Standard Track:** Standard steel/iron rail. Vanilla speed and handling.
3. **Reinforced Track:** Blast-resistant tracks impervious to TNT and creeper detonations.
4. **High Speed (HS) Track:**
  * Accelerates minecarts up to **4x vanilla maximum velocity** (~32 m/s).
  * **HS Transition Tracks:** Must precede HS tracks to safely ramp up velocity.
  * **Hazard:** If an HS train strikes an animal, enters a sharp 90-degree turn without deceleration, or hits a non-HS track at top speed, **it derails and explodes violently**.

### 3.2 Specialty Functional Tracks
* **Boarding Track:** Halts incoming cart until a mob/player boards; then launches cart in configured direction.
* **Holding Track:** Catches and holds carts in place until receiving a redstone signal.
* **Disembarking Track:** Automatically ejects passengers from carts to a designated side.
* **Elevator Track:** Allows minecarts to climb vertical 90-degree walls.
* **Junction Track:** Allows two intersecting tracks to cross orthogonally without collision or path deviation.
* **Switch Track:** Smooth branch turn controlled by a Switch Motor or redstone signal.
* **Control Track:** Enforces unidirectionality or boosts carts without redstone torches.
* **Detector Tracks:** Emits redstone pulses based on criteria: Item, Tank, Mob, Player, Train, Explosive.

---

## 4. Rolling Stock & Articulated Trains

### 4.1 Cart Coupling (Train Physics)
* Using a **Crowbar**, players can right-click two adjacent minecarts to couple them together with link chains.
* Articulated trains move with combined inertia, momentum transfer, and slack dynamics.
* Train length can reach dozens of carts pulled by a single engine.

### 4.2 Locomotives
* **Steam Locomotive:** Consumes Coal Coke/Charcoal and Water. Pulls long trains. Driven manually or controlled via track signaling.
* **Electric Locomotive:** Consumes EU or electric power from overhead or center third-rail tracks.

### 4.3 Specialty Minecarts
* **Tank Cart:** Transports up to 32 buckets (32,000 mB) of any liquid.
* **Energy Cart:** Stores IndustrialCraft EU or BuildCraft MJ to power distant outposts.
* **Anchor Cart:** Acts as a mobile chunk loader, keeping surrounding chunks loaded while the train traverses the world.
* **Tunnel Bore:**
  * Massive multiblock tunneling engine.
  * Requires a Bore Head (Iron, Steel, or Diamond).
  * Consumes fuel, ballast (Gravel/Sand), and Track items.
  * Automatically excavates a 3x3 tunnel, lays gravel ballast beneath, places rails, and tunnels forward through mountains.

---

## 5. Signaling & Interlocking

### 5.1 Block Signals & Signal Blocks
* Dividing tracks with Signal Blocks prevents trains from entering occupied zones.
* **Block Signal:** Displays Green (clear), Yellow (caution/adjacent block occupied), or Red (danger/block occupied).
* **Distant Signal:** Previews status of the next downstream signal to warn high-speed trains to decelerate.
* **Signal Controller & Receiver:** Transmits track occupancy wirelessly across pairs using Signal Tuners.

### 5.2 Routing System
* **Routing Table & Routing Tickets:** Players program destinations into tickets.
* **Routing Tracks:** Reads train ticket destination and automatically throws Switch Tracks to steer trains through track switch yards.

---

## 6. Complete Registry

### Blocks
1. `railcraft:coke_oven`
2. `railcraft:blast_furnace`
3. `railcraft:boiler_firebox_solid`, `railcraft:boiler_firebox_liquid`
4. `railcraft:boiler_tank_lp`, `railcraft:boiler_tank_hp`
5. `railcraft:steam_engine_hobbyist`, `railcraft:steam_engine_commercial`, `railcraft:steam_engine_industrial`
6. `railcraft:tank_iron_wall`, `railcraft:tank_iron_gauge`, `railcraft:tank_iron_valve`
7. `railcraft:tank_steel_wall`, `railcraft:tank_steel_gauge`, `railcraft:tank_steel_valve`
8. `railcraft:rolling_machine`
9. `railcraft:rock_crusher`
10. `railcraft:track_standard`
11. `railcraft:track_reinforced`
12. `railcraft:track_high_speed`
13. `railcraft:track_boarding`
14. `railcraft:track_holding`
15. `railcraft:track_elevator`
16. `railcraft:track_switch`
17. `railcraft:track_junction`
18. `railcraft:signal_block`
19. `railcraft:signal_distant`
20. `railcraft:switch_motor`

### Items
1. `railcraft:crowbar`
2. `railcraft:steel_ingot`
3. `railcraft:coal_coke`
4. `railcraft:creosote_oil_bucket`
5. `railcraft:wooden_tie`
6. `railcraft:railbed_wood`, `railcraft:railbed_stone`
7. `railcraft:rail_standard`, `railcraft:rail_reinforced`, `railcraft:rail_high_speed`
8. `railcraft:cart_tank`
9. `railcraft:cart_energy`
10. `railcraft:cart_anchor`
11. `railcraft:cart_bore`
12. `railcraft:locomotive_steam`, `railcraft:locomotive_electric`
13. `railcraft:signal_tuner`
14. `railcraft:whistle`

---

## 7. Recreation Implementation Notes for Rekkit (Rust)
1. **Articulated Cart Physics:**
   * Model train consists as a linked graph of cart entities connected by spring-damper constraints.
   * Calculate coupled velocity vectors and resolve momentum on slopes and collisions.
2. **High-Speed Derailment Simulation:**
   * Check cart speed entering turns: if velocity exceeds the turn's radius critical speed limit, detach cart from track rails, launch along tangent vector, and trigger an explosion on obstacle impact.
3. **Boiler Thermal Math:**
   * Compute per-tick thermal equilibrium:
     $$\Delta T = \frac{\text{FuelEnergy} - \text{HeatLoss} - \text{BoilingCost}}{\text{ThermalMass}}$$
   * Water check: If $T > 100^\circ\text{C}$, $\text{Water} == 0$, and $\text{IncomingWater} > 0$, call `world.create_explosion(pos, 8.0, true)`.
