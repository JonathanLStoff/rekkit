# EnderStorage - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** EnderStorage v1.1.2 - v1.4.x for Minecraft 1.2.5 / 1.4.7 / 1.5.2.
* **Author:** ChickenBones.
* **Core Philosophy:** Lightweight, infinite-distance, cross-dimensional item and fluid storage networks. Color-coded channel frequency system (4,096 channels), cryptographic private player channel locking via diamond latches, and synchronized handheld Ender Pouches.

---

## 2. Core Frequency & Network System

### 2.1 The 3-Color Frequency Triad
* Every Ender Chest, Ender Tank, and Ender Pouch features **3 independent color pads / bands** on its lid or surface.
* Each pad can be colored using any of the 16 Minecraft dyes:
  $$\text{Total Channels} = 16 \times 16 \times 16 = 4,096 \text{ Unique Public Frequencies}$$
* **In-World Recolor Interaction:**
  * Holding a dye item and right-clicking the left, middle, or right pad on an Ender Chest or Tank instantly updates that pad's color and rebinds the block to the new frequency channel.

### 2.2 Private Player Locking (The Diamond Latch)
* **Public Mode (Default):**
  * Latch appears as a metallic iron clasp.
  * Shared across all players on the server who tune into the same color frequency.
* **Private Mode (Diamond Clasp):**
  * Right-clicking the latch with a **Diamond** locks the frequency channel exclusively to the interacting player's UUID.
  * The latch texture transforms into a glowing cyan diamond clasp.
  * Each player receives their own private 4,096-channel namespace:
    $$\text{Player A (White/White/White)} \ne \text{Player B (White/White/White)} \ne \text{Public (White/White/White)}$$
* **Unlocking:** Sneak-right-clicking the diamond latch with a bare hand or wrench pops the diamond off into the player's inventory and returns the channel to the public pool.

### 2.3 Global Persistent Storage Model
* **Decoupled State:** Inventories and fluid tanks are **not stored inside chunk tile entities**.
* All frequency contents reside in a global world data file (`EnderStorage.dat`).
* Breaking all Ender Chests of a specific color code does **not** erase or drop the items stored on that frequency. Placing a chest with the same color combination anywhere else immediately restores access to the items.

---

## 3. Hardware & Devices

### 3.1 Ender Chest
* **Capacity:** 27 slots (or 54 slots in double-chest mode / configurable).
* **Dimensional Reach:** True zero-latency cross-dimensional access (Overworld, Nether, The End, Mystcraft Ages).
* **Automation Ready:**
  * Connects natively to BuildCraft Wooden Transport Pipes, RedPower Pneumatic Tubes, IndustrialCraft Hoppers, and Applied Energistics Storage/Import/Export Buses.
* **Lid Animation:**
  * The lid smoothly swings open whenever any player on the server opens a chest or pouch on that frequency channel.
  * Synchronized over multiplayer network packets.

### 3.2 Ender Tank
* **Function:** The liquid equivalent of the Ender Chest.
* **Capacity:** Stores **16,000 mB (16 buckets)** of any fluid (Water, Lava, Oil, Fuel, Biomass, Molten Redstone, etc.).
* **Valve Dial Interaction:**
  * Center face features a rotating valve dial.
  * Right-clicking the dial toggles modes:
    * **Neutral:** Accepts and supplies fluids only when pumped by external pipes/machines.
    * **Push Mode (Auto-Eject):** Automatically pumps fluid into touching fluid receptacles or waterproof/liquid conduits.
    * **Pull Mode (Auto-Suction):** Automatically sucks fluid from adjacent fluid source blocks or connected fluid machines.

### 3.3 Ender Pouch
* **Form Factor:** Handheld leather bag.
* **Frequency Binding:**
  * Sneak-right-clicking an Ender Pouch directly onto an Ender Chest or Ender Tank synchronizes the pouch to that container's exact 3-color frequency and diamond lock state.
  * The 3 colored stitches on the pouch item texture dynamically update to match the bound channel.
* **Portable Access:**
  * Right-clicking the Ender Pouch in the air opens the synchronized Ender Chest's inventory GUI from anywhere in any dimension.

---

## 4. Complete Registry

### Blocks
1. `enderstorage:ender_chest`
2. `enderstorage:ender_tank`

### Items
1. `enderstorage:ender_pouch`

---

## 5. Crafting Recipes

### 5.1 Ender Chest
* Layout (3x3):
  * Top Row: Blaze Rod, Wool (any color), Blaze Rod
  * Middle Row: Obsidian, Chest, Obsidian
  * Bottom Row: Blaze Rod, Ender Pearl, Blaze Rod

### 5.2 Ender Tank
* Layout (3x3):
  * Top Row: Blaze Rod, Wool (any color), Blaze Rod
  * Middle Row: Obsidian, Cauldron, Obsidian
  * Bottom Row: Blaze Rod, Ender Pearl, Blaze Rod

### 5.3 Ender Pouch
* Layout (3x3):
  * Top Row: Leather, Wool (any color), Leather
  * Middle Row: Blaze Powder, Ender Pearl, Blaze Powder
  * Bottom Row: Leather, Leather, Leather

---

## 6. Recreation Implementation Notes for Rekkit (Rust)
1. **Global Storage Manager:**
   * Create an actor or state container:
     ```rust
     pub struct EnderStorageManager {
         public_chests: HashMap<FrequencyKey, Inventory27>,
         private_chests: HashMap<(Uuid, FrequencyKey), Inventory27>,
         public_tanks: HashMap<FrequencyKey, FluidTank16k>,
         private_tanks: HashMap<(Uuid, FrequencyKey), FluidTank16k>,
     }
     ```
   * Save and load this state separately to disk on server save events.
2. **Dynamic Item Model Tinting:**
   * In client rendering packets, pass the 3 color indices (`[u8; 3]`) in the item stack NBT/component metadata so the Ender Pouch item model renders the 3 colored stitch bands correctly.
3. **Chunk Independence:**
   * When an Ender Chest/Tank block entity ticks, it merely holds a reference `(FrequencyKey, Option<Uuid>)`. It delegates all item transfers and fluid queries to the central `EnderStorageManager`.
