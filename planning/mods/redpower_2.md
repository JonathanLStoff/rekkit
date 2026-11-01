# RedPower 2 - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** RedPower 2 Pre-Release 6 (v2.0pr6) for Minecraft 1.4.6/1.4.7 (shipped in Direwolf20 1.4.7: `assets/direwolf20/mods/redpowercore.zip`, `redpowerdigital.zip`, `redpowermechanical.zip`, `redpowercompat.zip`).
* **Cross-Reference Version:** RedPower 2 Pre-Release 5b2 (v2.0pr5b2) for Minecraft 1.2.5 (shipped in Tekkit Classic: `assets/tekkit/client/mods/RedPowerCore-2.0pr5b2.zip`, `RedPowerWiring-2.0pr5b2.zip`, `RedPowerLogic-2.0pr5b2.zip`, `RedPowerLighting-2.0pr5b2.zip`, `RedPowerWorld-2.0pr5b2.zip`, `RedPowerMachine-2.0pr5b2.zip`, `RedPowerControl-2.0pr5b2.zip`).
* **Author:** Eloraam (`mcmod.info`, `com.eloraam.redpower.RedPowerCore`).
* **Licence:** Proprietary / All Rights Reserved (Eloraam custom freeware; decompiled client archive used solely for clean-room reverse-engineering of behavioral specifications).
* **Decompiled Source Paths:**
  * `assets/decompiled/redpowercore/` (`RedPowerCore`, `RedPowerBase`)
  * `assets/decompiled/redpowerdigital/` (`RedPowerWiring`, `RedPowerLogic`, `RedPowerLighting`)
  * `assets/decompiled/redpowermechanical/` (`RedPowerWorld`, `RedPowerMachine`, `RedPowerControl`)
  * `assets/decompiled/redpowercompat/` (`RedPowerCompat`)
* **Core Philosophy:** Modular, deterministic automation without magical single-block teleporters or destructive overflow drops. Precise digital logic, multi-colored insulated and bundled wiring, pneumatic item transport with strict destination pathfinding, sub-voxel microblocks, structural translation via frames, 100V DC Blutricity simulation, and a full 65C816/6502 microprocessor computing environment.

---

## 2. Core Systems & Mechanics

### 2.1 Microblocks & Sub-Voxel System
* **Source:** `com.eloraam.redpower.core.CoverLib`, `com.eloraam.redpower.core.CoverRecipe`, `com.eloraam.redpower.core.TileCovered`, `com.eloraam.redpower.core.BlockMicro`.
* **Sub-Voxel Coordinate Grid:** A standard 1×1×1 voxel space is subdivided into 8 octants, 12 edges, and 6 faces. Slices are calibrated in units of 1/8th of a block ($0.125\text{ m}$).
* **Thickness Hierarchy:**
  * Cover: 1/8th block ($0.125\text{ m}$)
  * Panel: 2/8th block ($0.250\text{ m}$)
  * Triple Cover: 3/8th block ($0.375\text{ m}$)
  * Slab / Half-Block: 4/8th block ($0.500\text{ m}$)
  * Cover Slab: 5/8th block ($0.625\text{ m}$)
  * Triple Panel: 6/8th block ($0.750\text{ m}$)
  * Anticover: 7/8th block ($0.875\text{ m}$)
* **Geometrical Forms per Thickness:**
  * **Face Plate:** Bound to one of 6 faces ($Y^-, Y^+, Z^-, Z^+, X^-, X^+$).
  * **Hollow Face Plate:** Face plate with a centered 1/2-width opening allowing wires, cables, and pipes to pass through the face.
  * **Edge Strip:** Rectangular prismatic strip running along 1 of 12 block edges ($t \times t \times 1$).
  * **Corner:** Cubic vertex piece placed in 1 of 8 block corners ($t \times t \times t$).
* **Structural Columns:**
  * Post (`rppole1`): 1/8th centered column along an axis.
  * Pillar (`rppole2`): 1/4th centered column along an axis.
  * Column (`rppole3`): 1/2th centered column along an axis.
* **Placement & Occlusion Masking (`CoverLib.addMaterial`, `TileCovered.getTubeConnectableSides`):**
  * Up to 6 face plates, 12 strips, and 8 corners can occupy a single block coordinate together with an internal wire, cable, or tube (`TileCovered`).
  * Placing a solid cover on face $F$ seals that face: external neighbor connections through face $F$ are blocked. Hollow covers permit wire/tube connections through the face (`Covers[i] >> 8 >= 3`).
* **Cutting Rules (`CoverRecipe.getSawRecipe`):**
  * Saws have sharpness ratings: Iron Handsaw = 0, Ruby/Sapphire/Green Sapphire Handsaw = 1, Diamond Handsaw = 2 (`ItemHandsaw.getSharpness`). Materials specify minimum required hardness (`CoverLib.getHardness`).
  * **Vertical Cut (saw above or below material):** Halves thickness ($1 \rightarrow 2\times\text{Slab} \rightarrow 4\times\text{Panel} \rightarrow 8\times\text{Cover}$).
  * **Horizontal Cut (saw beside material):** Slices slice into strips, and strips into corners ($1\times\text{Cover} \rightarrow 2\times\text{Cover Strip} \rightarrow 4\times\text{Cover Corner}$).
  * 8 covers arranged around an empty 3×3 grid center craft 8 hollow covers.
  * Slices can be recombined: 2 covers craft 1 panel; 2 panels craft 1 slab; 2 slabs craft 1 full block.

### 2.2 Wiring System
* **Source:** `com.eloraam.redpower.wiring.TileRedwire`, `TileInsulatedWire`, `TileCable`, `TileBluewire`, `com.eloraam.redpower.core.RedPowerLib`.
* **Red Alloy Wire (`TileRedwire`):**
  * Transmits analog signal strength ($0..=255$ internal steps, mapped to $0..=15$ vanilla redstone levels via `strength / 17`).
  * Signal starts at 255 and decreases by 1 per block hop (`cs = Math.max(0, mx - 1)` in `RedPowerLib.java:897`), propagating up to 256 blocks from the power source.
  * Surface-mounted on floors, walls, and ceilings. Can traverse internal and external 90-degree corners over block edges (`RedPowerLib.getConnections`).
* **Insulated Wire (`TileInsulatedWire`):**
  * 16 distinct dye color variants (metadata 0..15: White, Orange, Magenta, Light Blue, Yellow, Lime, Pink, Gray, Light Gray, Cyan, Purple, Blue, Brown, Green, Red, Black).
  * Connects only to wires of the identical dye color or bare Red Alloy wire. Adjacent parallel insulated wires of different colors do not bridge or crosstalk.
* **Bundled Cable (`TileCable`):**
  * Encapsulates all 16 insulated signals into a single 16-channel multiplexed bus (`BundledSignal` bitmask `0..=0xFFFF`).
  * Available in Neutral form (damage 768) and 16 dyed color variants (damage 769..784).
  * Connects directly to Insulated Wires (splitting out the matching color bit), Logic Gates, and Computer I/O Expanders.
* **Bluewire (`TileBluewire`):**
  * Transmits Blutricity (100V DC). Uninsulated (damage 1280) and Insulated (damage 1281).
  * Internal resistance: $R = 0.01\,\Omega/\text{block}$ (`BluePowerConductor.getResistance`).
* **Jacketed Wires / Cables (`MicroPlacementJacket`):**
  * Freestanding, rigid 1-block columnar conduits that do not require attachment to a solid supporting block face.

### 2.3 Integrated Logic Tiles
* **Source:** `com.eloraam.redpower.logic.BlockLogic`, `TileLogicPointer`, `TileLogicSimple`, `TileLogicStorage`, `TileLogicAdv`, `TileLogicArray`.
* Flat 1/8-block tiles placeable on any solid face, rotatable in 4 directions. Configured and rotated with a Screwdriver (`ItemScrewdriver`).
* **Gate Catalog & State Machines:**
  1. **Inverter / NOT Gate (`TileLogicSimple` sub 9):** 1 input (back), 3 outputs (left, front, right). Outputs high when input is low.
  2. **AND Gate (`TileLogicSimple` sub 4):** 3 configurable inputs. Outputs high when all active inputs are high.
  3. **OR Gate (`TileLogicSimple` sub 2):** 3 configurable inputs. Outputs high when any active input is high.
  4. **NAND Gate (`TileLogicSimple` sub 3):** Inverted AND.
  5. **NOR Gate (`TileLogicSimple` sub 1):** Inverted OR.
  6. **XOR Gate (`TileLogicSimple` sub 6):** Outputs high when an odd number of active inputs are high.
  7. **XNOR Gate (`TileLogicSimple` sub 5):** Outputs high when active inputs match / even parity.
  8. **Buffer Gate (`TileLogicSimple` sub 10):** Delays signal by 1 tick; prevents backflow.
  9. **Multiplexer (`TileLogicSimple` sub 11):** Inputs A (left) and B (right), control S (back). Selects input A when S is low, input B when S is high.
  10. **Repeater (`TileLogicSimple` sub 12):** Configurable delay using schedule table: 2, 4, 6, 8, 16, 32, 64, 128, 256 redstone ticks (`tickSchedule`).
  11. **Timer (`TileLogicPointer` sub 0):** Rotating pointer interval timer. Default interval 40 ticks ($2.0\text{ s}$); adjustable from 4 ticks ($0.2\text{ s}$) to continuous via GUI slider buttons. Emits a 2-tick output pulse on interval completion.
  12. **Sequencer (`TileLogicPointer` sub 1):** 4 outputs arranged around the perimeter. Rotates its pointer continuously according to its configured time interval (`TileLogicPointer.sequencerUpdate`), pulsing each of the 4 outputs in sequence (cycles clockwise or counter-clockwise, playing `random.click` on step change; does not cycle per input pulse).
  13. **State Cell (`TileLogicPointer` sub 2):** Monostable pulse timer with configurable active duration.
  14. **RS Latch (`TileLogicSimple` sub 0):** Bistable Set/Reset latch.
  15. **Toggle Latch (`TileLogicSimple` sub 8):** T flip-flop; toggles output state on each rising edge input pulse.
  16. **Transparent Latch / D-Latch (`TileLogicSimple` sub 15):** Data passes through while clock input is high; latches and holds state when clock transitions low.
  17. **Pulse Former (`TileLogicSimple` sub 7):** Emits an exact 1-tick pulse on rising edge of continuous input signal.
  18. **Synchronizer (`TileLogicSimple` sub 13):** Latches two independent input pulses (left and right) and emits a joint output pulse only when both have arrived.
  19. **Randomizer (`TileLogicSimple` sub 14):** Clock pulse randomly activates 1 of 3 outputs (33.3% equal probability).
  20. **Light Sensor (`TileLogicSimple` sub 16):** Emits signal based on ambient skylight and blocklight levels.
  21. **Counter (`TileLogicStorage` sub 0):** Counts pulses. Configurable `CountMax` (default 10), `Inc` (default 1), `Dec` (default 1). Left input increments, right input decrements. Outputs signal at minimum count (0) or maximum count (`CountMax`). Plays `random.click` ($0.3\text{ volume}$, $0.5\text{ pitch}$).
  22. **Null Cell (`TileLogicArray` sub 0):** Non-functional passive pass-through cell for array logic.
  23. **Invert Cell (`TileLogicArray` sub 1):** Inverting pass-through cell in array logic grids.
  24. **Non-Invert Cell (`TileLogicArray` sub 2):** Non-inverting buffer cell in array logic grids.
  25. **Bus Transceiver (`TileLogicAdv` sub 0):** 16-channel bidirectional Bundled Cable transceiver with direction control and latching.

### 2.4 Pneumatic Tubes & Item Logistics
* **Source:** `com.eloraam.redpower.core.TubeLib`, `com.eloraam.redpower.core.TubeFlow`, `com.eloraam.redpower.machine.TileTube`, `TileRestrictTube`, `TileMagTube`.
* **Zero Drop Guarantee:** Items inside tubes never drop into the world as loose entities during normal operation. If an item cannot path to a valid recipient, extraction halts or the item remains inside internal machine buffers.
* **Shortest-Path Dijkstra Routing (`TubeLib.findRoute`, `TubeLib.RouteFinder`):**
  * When an item enters the network, a Dijkstra search traverses valid tube paths to find the closest recipient inventory with available capacity (`MachineLib.canAddToInventory`).
  * **Hop Weight:** Standard pneumatic tube traversal adds $+2$ weight per block hop (`wr.weight + 2`); turns deviating from the entry direction add $+1$ weight.
  * **Restrictor Tube Weight:** Adds $+1,000,000$ weight (`TileRestrictTube.tubeWeight() == 1000000`). Used as virtual high-cost routes to designate overflow chests.
  * **Color Filtering:** Tubes dyed with a Paintbrush (16 colors) enforce strict matching: an item tagged with dye color $C$ can only enter unpainted tubes or tubes painted matching color $C$ (`TileTube.tubeItemCanEnter`).
* **Magtubes & Accelerator (`TileMagTube`, `TileAccel`):**
  * Standard pneumatic speed: $3.0\text{ blocks/sec}$.
  * Accelerator accelerates payloads into magnetic capsules traveling at $16.0\text{ blocks/tick}$ ($320\text{ blocks/sec}$) through Magtubes.
  * Acceleration power consumption: $100\text{ W-t} \times \text{stack count}$ drawn from Blutricity at $\ge 60\text{V}$ (`TileAccel.cond.drawPower`).

### 2.5 Blutricity Power Grid
* **Source:** `com.eloraam.redpower.core.BluePowerConductor`, `com.eloraam.redpower.core.BluePowerEndpoint`, `com.eloraam.redpower.machine.TileBatteryBox`, `TileSolarPanel`, `TileThermopile`, `TileWindTurbine`, `TileTransformer`.
* **Physical Model:** Voltage ($V$ in Volts), Current ($I$ in Amps), Charge ($Q$ in Coulombs).
  * Nominal operating voltage: $100.0\text{ V}$.
  * Minimum machine operating threshold: $60.0\text{ V}$ (`cond.getVoltage() >= 60.0`).
  * Peak safe operating limit: $120.0\text{ V}$ (wind turbines allow up to $130.0\text{ V}$).
  * Conductor parameters (`BluePowerConductor`):
    * Conductor internal resistance: $R = 0.01\,\Omega$ (`getResistance`).
    * Inductive scale factor: $0.07$ (`getIndScale`).
    * Parallel conductance factor: $0.5$ (`getCondParallel`).
    * Endpoint capacitance: $C = 4.0\text{ F}$ (`getInvCap() == 0.25`).
    * Panel capacitance: $C = 0.25\text{ F}$ (`getInvCap() == 4.0`).
  * Dynamic transfer step (`BluePowerConductor.iterate()`):
    $$R = R_1 + R_2, \qquad V = V_1 - V_2$$
    $$I_{\text{next}} = I + (V - I \cdot R) \times 0.07$$
    $$I_{\text{flow}} = I_{\text{next}} + V \times 0.5$$
    Apply $-I_{\text{flow}}$ to local node, $+I_{\text{flow}}$ to neighbor node.
* **Generation Devices:**
  * **Solar Panel (`TileSolarPanel`):** Generates $2.0\text{ A}$ current when $V \le 100\text{V}$, block can see sky, is daytime, and world has no storm (`applyDirect(2.0)`).
  * **Thermopile (`TileThermopile`):** Scans 6 faces every 20 ticks. Generates $0.005 \times \min(T_{\text{hot}}, T_{\text{cold}})\text{ A}$ when $V \le 100\text{V}$. Temperature ratings: Air (Nether +1, Overworld -1), Torch (+5), Fire (+3), Lava (+100), Burning Furnace (+25), Water (-25), Snow layer (-50), Ice/Snow block (-100).
  * **Wind Turbine (`TileWindTurbine`):** Raytraces unobstructed air in 4 cardinal directions (224 rays for vertical turbine, 340 rays for horizontal windmill). Applies $P / 5\text{ W}$ per tick when $V \le 130\text{V}$ and brakes released. Durability wear: 1 damage to sails every 6,600 ticks (~5.5 minutes). Redstone signal engages mechanical brakes, halting generation.
* **Storage & Transmission:**
  * **Battery Box (`TileBatteryBox`):** Stores 6,000 storage units (~1,200,000 W-t). Charges when $V > 90\text{V}$ ($n = \min((V\times 10 - 900)/10, 10)$, draws $n \times 1000\text{ W}$); discharges when $V < 80\text{V}$ and not redstone-powered ($n = \min((800 - V\times 10)/10, 10)$, applies $n \times 1000\text{ W}$). Charges/discharges BT Battery items in slots 0 and 1 at up to 25 units/tick.
  * **Transformer (`TileTransformer`):** Steps voltage between 100V low-voltage network and 10,000V high-voltage long-distance transmission line ($100\times$ step-up/down, `Vcap2 = Vcap1 * 100.0`, `Iin2 = Iin1 * 100.0`).

### 2.6 Frame Automation & Structural Movement
* **Source:** `com.eloraam.redpower.machine.TileMotor`, `com.eloraam.redpower.machine.TileFrame`, `TileFrameMoving`, `com.eloraam.redpower.core.FrameLib`.
* **Contiguous Body Assembly:** Support Frames (`rpframe`, `rptframe`, `rprtframe`) adhere to adjacent solid blocks and neighboring frames on unmasked faces (configured via Screwdriver or microblock covers).
* **Solver Algorithm (`FrameLib.FrameSolver`):**
  * Breadth-first search traverses all connected frame links up to `FrameLinkSize` (default: 1,000 blocks; config `settings.machine.frame.linksize`).
  * **Immovable Block Rule:** Fails if any block has negative hardness: `block.getBlockHardness(world, x, y, z) < 0.0F` (`FrameLib.java:46`). Bedrock and End Portal frames cannot be moved. Obsidian has hardness 50.0F and CAN be moved by default.
* **Movement Execution (`TileMotor.pickFrame`, `TileMotor.dropFrame`):**
  * Movement takes exactly 16 world ticks (`MovePos` progresses 0 to 16).
  * Power cost: Draws $100 + 10 \times \text{LinkSize}\text{ W-t}$ per tick for 16 ticks (`TileMotor.java:107`). Requires $V \ge 60\text{V}$.
  * During the 16-tick translation, blocks are represented by moving frame placeholders (`TileFrameMoving`), preserving block IDs, metadata, and full TileEntity NBT payloads.
  * At tick 16, blocks are atomically re-instantiated at their new world coordinates without dropping inventory items or resetting states.

### 2.7 Control 6502 Computing Subsystem
* **Source:** `com.eloraam.redpower.control.TileCPU`, `TileDisplay`, `TileDiskDrive`, `TileIOExpander`, `TileBackplane`, `TileRAM`, `GuiDisplay`.
* **Microprocessor Architecture:**
  * Emulated WDC 65C816 / 6502 8/16-bit microprocessor with 8 KB onboard RAM (addresses `0x0000..=0x1FFF`).
  * Backplane expansion: Up to 7 RAM modules (`TileRAM`) mounted on backplanes (`TileBackplane`) expand address space in 8 KB increments up to 64 KB total RAM (`TileCPU.readOnlyMem`).
  * Execution rate: $+1,000$ instruction cycles per world tick; burst accumulator capped at 100,000 cycles (`TileCPU.sliceCycles`).
  * Bootloader: Automatically loads 256 bytes from `/eloraam/control/rpcboot.bin` into address 1024 (`0x0400`) on cold boot.
* **Display / Terminal (`TileDisplay`, `GuiDisplay`):**
  * Character matrix: $80\text{ columns} \times 50\text{ rows}$ (4,000-byte ASCII screen buffer, `screen = new byte[4000]`).
  * Cursor modes: 0 = off, 1 = solid block, 2 = blink (`cursMode`).
  * Hardware blitter: Rectangular screen block memory copy (`blitXS`, `blitYS`, `blitXD`, `blitYD`, `blitW`, `blitH`).
  * Keyboard input: Built directly into terminal GUI (`GuiDisplay.sendKey`). Keystrokes sent over packet 212 into 16-byte server ring buffer (`kbbuf`).
* **Floppy Disk Drive (`TileDiskDrive`):**
  * 256 KB virtual disk storage: 2,048 sectors of 128 bytes each (`TileDiskDrive.databuf = new byte[128]`).
  * Redbus commands: 1 = Read Label, 2 = Write Label, 3 = Read Serial Number, 4 = Read Sector, 5 = Write Sector.
  * System disks: RedPower Forth Boot Disk (`redforth.img`), RedPower Forth Extra Disk (`redforthxp.img`).
* **I/O Expander (`TileIOExpander`):**
  * Memory-mapped Redbus device (default address 3).
  * 4 registers mapping low/high bytes of 16-channel Bundled Cable inputs (`RBuf`) and outputs (`WBuf`).

---

## 3. Machines & Devices

### 3.1 Appliance Machines
1. **Alloy Furnace (`redpower_2:alloy_furnace`):**
   * Purpose: Smelts metallurgical alloys using solid fuel.
   * Inventory: 11 slots (`ContainerAlloyFurnace`).
     * Slots 0..8: 3×3 alloy input matrix.
     * Slot 9: Solid fuel slot (standard burn times).
     * Slot 10: Smelted output slot.
   * Cook time: 200 ticks ($10.0\text{ s}$).
   * Sided access: Top = slots 0..8; Front = slot 9 (fuel); Back = slot 10 (output).
2. **Project Table (`redpower_2:project_table`):**
   * Purpose: Advanced workbench with persistent 3×3 crafting grid, blueprint plan slot, and storage.
   * Inventory: 28 slots (`ContainerAdvBench`).
     * Slots 0..8: 3×3 persistent crafting grid.
     * Slot 9: Plan slot (`blank_plan` or `plan`).
     * Slots 10..27: 18 internal inventory storage slots (2 rows of 9).
   * Sided access: Sided inventories can insert/extract to/from storage slots 10..27.
3. **Blulectric Furnace (`redpower_2:blulectric_furnace`):**
   * Purpose: Smelts standard vanilla recipes at 2× speed using Blutricity.
   * Inventory: 2 slots (`ContainerBlueFurnace`): Slot 0 = input, Slot 1 = output.
   * Power: Requires $\ge 60\text{V}$, draws $1,000\text{ W-t}$ per tick during operation.
   * Cook time: 100 ticks ($5.0\text{ s}$).
4. **Buffer Chest (`redpower_2:buffer_chest`):**
   * Purpose: High-capacity multi-faced buffer to decouple fast extraction from slow processing.
   * Inventory: 20 slots (`ContainerBufferChest`).
   * Sided access: Rear face connects to all 20 slots; each side face connects to a dedicated 4-slot partition.
5. **Blulectric Alloy Furnace (`redpower_2:blulectric_alloy_furnace`):**
   * Purpose: Smelts alloy recipes without solid fuel using Blutricity.
   * Inventory: 10 slots (`ContainerBlueAlloyFurnace`): Slots 0..8 = 3×3 inputs, Slot 9 = output.
   * Power: Requires $\ge 60\text{V}$, draws $1,000\text{ W-t}$ per tick. Cook time: 100 ticks ($5.0\text{ s}$).
6. **Charging Bench (`redpower_2:charging_bench`):**
   * Purpose: Recharges portable Blutricity items (BT Battery, Sonic Screwdriver).
   * Inventory: 16 slots (4×4 grid, `ContainerChargingBench`).
   * Storage: Internal buffer of 3,000 units (~600,000 W-t).

### 3.2 Tube & Logistics Machines
1. **Deployer (`redpower_2:deployer`):**
   * Purpose: Simulates right-click of held item onto adjacent block on redstone pulse.
   * Inventory: 9 slots (`ContainerDeploy`).
   * Behavior: Places blocks, uses buckets, plants seeds, shears sheep, or applies bonemeal using a fake player entity.
2. **Block Breaker (`redpower_2:block_breaker`):**
   * Purpose: Breaks adjacent block on redstone pulse.
   * Inventory: Internal buffer (`TubeBuffer`).
   * Behavior: Harvests block in front (unless hardness < 0 like Bedrock), collects drops, and ejects into rear-facing tube or chest.
3. **Transposer (`redpower_2:transposer`):**
   * Purpose: Vacuums ground entities or extracts 1 item stack from adjacent inventory on redstone pulse.
   * Behavior: Injects extracted items into connected rear tube.
4. **Filter (`redpower_2:filter`):**
   * Purpose: Extracts 1 item stack matching a 9-slot template on redstone pulse.
   * Inventory: 9 ghost filter slots (`ContainerFilter`).
   * Dye Tagging: Assigns configured color tag ($0..=16$) to all passing items.
5. **Item Detector (`redpower_2:item_detector`):**
   * Purpose: In-line tube detector.
   * Inventory: 9 filter slots (`ContainerItemDetect`).
   * Behavior: Emits redstone pulse when an item passing through matches the filter.
6. **Sorting Machine (`redpower_2:sorting_machine`):**
   * Purpose: Advanced 8-channel routing and distribution.
   * Inventory: 40 slots (8 columns of 5 rows, `ContainerSorter`).
   * Color buttons: 8 independent column color selectors.
   * Modes: Any item matching column, sequential column cycling, full stack matching, all items matching, even distribution, random column. Blutricity auto-pulling mode.
7. **Retriever (`redpower_2:retriever`):**
   * Purpose: Pulls matching items across the entire tube network to itself on pulse.
   * Inventory: 9 filter slots (`ContainerRetriever`). Requires Blutricity.
8. **Regulator (`redpower_2:regulator`):**
   * Purpose: Maintains an exact stocking count in an adjacent inventory.
   * Inventory: 27 slots (`ContainerRegulator`): Slots 0..8 = target stock template, Slots 9..17 = buffer, Slots 18..26 = excess export filter.
9. **Assembler (`redpower_2:assembler`):**
   * Purpose: Advanced programmable deployer connected to Bundled Cables.
   * Inventory: 34 slots (`ContainerAssemble`): Slots 0..15 = 16 item template selectors, Slots 16..33 = 18 internal inventory slots. Each of the 16 bundled cable channels deploys the corresponding item.
10. **Ejector (`redpower_2:ejector`):**
    * Purpose: Ejects 1 item from its 9-slot inventory into a tube on each redstone pulse.
11. **Relay (`redpower_2:relay`):**
    * Purpose: Automatically extracts and ejects 1 item stack every 10 ticks from its 9-slot inventory into an attached tube.
12. **Sortron (`redpower_2:sortron`):**
    * Purpose: Memory-mapped Redbus sorting interface for 6502 Forth computer.
    * Registers: Command, Item Quantity, Slot, Item ID (32-bit), Damage, Color.
13. **Manager (`redpower_2:manager`):**
    * Purpose: Automatically balances and synchronizes an adjacent inventory across the tube network to match a 24-slot template (`ContainerManager`).

### 3.3 Power & Fluid Machines
1. **Solar Panel (`redpower_2:solar_panel`):** 1/4-block flat panel. Generates 2.0A daytime current.
2. **Thermopile (`redpower_2:thermopile`):** Solid block. Generates current from thermal gradients across 6 faces.
3. **Wind Turbine (`redpower_2:wind_turbine`):** Multiblock mast turbine. Converts wind speed into current.
4. **Pump (`redpower_2:pump`):** Pumping station with +1000 outlet pressure and -1000 inlet pressure. Moves fluids through pipes.
5. **Grate (`redpower_2:grate`):** Sucks liquid blocks from the world under suction, or pours liquids into the world under pressure.
6. **Transformer (`redpower_2:transformer`):** Steps voltage between 100V and 10,000V.
7. **Battery Box (`redpower_2:battery_box`):** 6,000 storage units capacity. Discharges when unpowered.
8. **Blulectric Engine (`redpower_2:blulectric_engine`):** Converts Blutricity into BuildCraft MJ (512 MJ flywheel).

---

## 4. Tools, Armor & Items

### 4.1 Metallurgy & Crafting Tools
* **Handsaws (`ItemHandsaw`):**
  * Iron Handsaw (`redpower_2:iron_handsaw`): Durability 320, Sharpness 0.
  * Ruby Handsaw (`redpower_2:ruby_handsaw`): Durability 640, Sharpness 1.
  * Green Sapphire Handsaw (`redpower_2:green_sapphire_handsaw`): Durability 640, Sharpness 1.
  * Sapphire Handsaw (`redpower_2:sapphire_handsaw`): Durability 640, Sharpness 1.
  * Diamond Handsaw (`redpower_2:diamond_handsaw`): Durability 1280, Sharpness 2.
  * Retained on craft (`crafting_remainder`), taking 1 durability damage per microblock cut.
* **Screwdriver (`redpower_2:screwdriver`):**
  * Lossless crafting remainder (`crafting_remainder = "redpower_2:screwdriver"`).
  * Right-click rotates microblock covers, logic gates, frame motors, and machines. Shift+right-click cycles operational modes.
* **Diamond Drawplate (`redpower_2:diamond_drawplate`):**
  * Durability 255 (`crafting_remainder = "redpower_2:diamond_drawplate"`).
  * Draws copper ingots into Fine Copper Wire and iron ingots into Fine Iron Wire.
* **Wool Card (`redpower_2:wool_card`):**
  * Durability 63. Crafts 1 wool block into 4 string.
* **Voltmeter (`redpower_2:voltmeter`):**
  * Right-click on Blutricity nodes displays precise voltage ($V$) and charge ($Q$).
* **Sonic Screwdriver (`redpower_2:sonic_screwdriver`):**
  * Rechargeable Blutricity screwdriver with internal battery. Configures machines remotely.
* **Plans (`redpower_2:blank_plan`, `redpower_2:plan`):**
  * Used in Project Table to encode and save 3×3 crafting recipes.

### 4.2 Mining & Combat Equipment
* **Gem Tools (Ruby, Green Sapphire, Sapphire):**
  * Pickaxes, Shovels, Axes, Swords, Hoes: Durability 500, Harvest level 2, Mining speed 8.0F, Damage 3, Enchantability 12 (`EnumHelper.addToolMaterial`).
* **Sickles (`ItemSickle`):**
  * Variants: Wood (durability 59), Stone (131), Iron (250), Diamond (1561), Gold (32), Ruby (500), Green Sapphire (500), Sapphire (500).
  * Radius: Harvests leaves in a $3\times 3\times 3$ volume (`leafRadius = 1`); harvests crops and tall grass in a $5\times 5$ flat area (`cropRadius = 2`).
* **Athame (`redpower_2:athame`):**
  * Silver ritual dagger, durability 100.
  * Base attack damage: 1 against normal mobs; **25 damage** against Endermen and the Ender Dragon (`ItemAthame.java:18`).
  * Accepts Disjunction enchantment.
* **Enchantments:**
  * **Disjunction (`EnchantmentDisjunction`, max level 5):** Exclusive to Athame. Deals $+ (6 \times \text{level})$ additional damage to Endermen and Ender Dragons.
  * **Vorpal (`EnchantmentVorpal`, max level 4):** Weapon enchantment. Chance of $2 \times \text{level}^2\,\%$ (2%, 8%, 18%, 32%) to deal 100 extra critical damage. Overkill damage $\le -20\text{ HP}$ decapitates victim heads (Skeleton, Zombie, Player, Creeper skulls, `WorldEvents.java:41`).

### 4.3 Utility & Storage Items
* **Canvas Bag (`redpower_2:canvas_bag`):** 27-slot portable inventory, 16 dye colors.
* **Seed Bag (`redpower_2:seed_bag`):** Holds up to 576 seeds (9 slots). Plants crops in a $5\times 5$ spiral on farmland. Automatically vacuums matching seeds picked up by the player.
* **Paint Cans & Brushes (`redpower_2:paint_can`, `redpower_2:paint_brush`):** 16 colors. Colors pneumatic tubes, cables, and canvas bags.

---

## 5. World Generation & Resources
* **Source:** `com.eloraam.redpower.world.WorldGenHandler`, `BlockCustomOre`, `BlockCustomStone`.
* **Ores (`WorldGenCustomOre`):**
  * **Ruby Ore:** 2 clusters/chunk, vein size 7, Y 0..47. Drops 1 Ruby (fortune-affected, XP 3..7).
  * **Green Sapphire Ore:** 2 clusters/chunk, vein size 7, Y 0..47. Drops 1 Green Sapphire (fortune-affected, XP 3..7).
  * **Sapphire Ore:** 2 clusters/chunk, vein size 7, Y 0..47. Drops 1 Sapphire (fortune-affected, XP 3..7).
  * **Silver Ore:** 4 clusters/chunk, vein size 8, Y 0..31. Drops ore block.
  * **Tin Ore:** 10 clusters/chunk, vein size 8, Y 0..47. Drops ore block.
  * **Copper Ore:** 20 clusters/chunk, vein size 8, Y 0..63. Drops ore block.
  * **Tungsten Ore:** 1 cluster/chunk, vein size 4, Y 0..15. Drops ore block.
  * **Nikolite Ore:** 4 clusters/chunk, vein size 10, Y 0..15. Drops 4..5 Nikolite dust ($+0..\text{fortune}$, XP 1..5).
* **Stone Formations:**
  * **Marble Strata (`WorldGenMarble`):** 4 pockets/chunk, Y 32..63. Smooth white decorative stone (hardness 1.0F).
  * **Basalt Volcanoes (`WorldGenVolcano`):** Active volcanoes with basalt cones and flowing lava chambers (hardness 2.5F, blast resistance 12.0F). Mining basalt drops basalt cobblestone.
* **Plants & Trees:**
  * **Wild Indigo Flowers:** Generates in Plains, Forest, ForestHills, and Swamp biomes. Harvested for Indigo Dye.
  * **Rubber Trees (`WorldGenRubberTree`):** Generates in Jungle and JungleHills biomes (6 per chunk). Rubber wood smelts into charcoal; leaves drop rubber saplings.
  * **Flax (`BlockCustomCrops`):** Wild seeds found in tall grass. Farmed on farmland for String and Flax Seeds.

---

## 6. Complete Item & Block Registry (`redpower_2`)

### 6.1 Blocks (113 entries)

| Rekkit ID | Name | Original Java Mapping |
| :--- | :--- | :--- |
| `redpower_2:alloy_furnace` | Alloy Furnace | `BlockAppliance` meta 0 |
| `redpower_2:project_table` | Project Table | `BlockAppliance` meta 3 |
| `redpower_2:blulectric_furnace` | Blulectric Furnace | `BlockAppliance` meta 1 |
| `redpower_2:buffer_chest` | Buffer Chest | `BlockAppliance` meta 2 |
| `redpower_2:blulectric_alloy_furnace` | Blulectric Alloy Furnace | `BlockAppliance` meta 4 |
| `redpower_2:charging_bench` | Charging Bench | `BlockAppliance` meta 5 |
| `redpower_2:microblock` | Microblock Plate | `BlockMicro` (compound multi-part) |
| `redpower_2:wire_red_alloy` | Red Alloy Wire | `BlockMicro` damage 256 |
| `redpower_2:wire_insulated` | Insulated Wire (16 colors) | `BlockMicro` damage 512..527 |
| `redpower_2:wire_bundled` | Bundled Cable (Neutral + 16 colors) | `BlockMicro` damage 768..784 |
| `redpower_2:wire_blutricity` | Bluewire | `BlockMicro` damage 1280 |
| `redpower_2:wire_blutricity_insulated` | Insulated Bluewire | `BlockMicro` damage 1281 |
| `redpower_2:wire_jacketed` | Jacketed Wire | `BlockMicro` damage 16384+ |
| `redpower_2:cable_jacketed` | Jacketed Cable | `BlockMicro` damage 16640+ |
| `redpower_2:wire_jacketed_blutricity` | Jacketed Bluewire | `BlockMicro` damage 16896+ |
| `redpower_2:pipe` | Fluid Pipe | `BlockMicro` damage 1792 |
| `redpower_2:pneumatic_tube` | Pneumatic Tube | `BlockMicro` damage 2048 |
| `redpower_2:redstone_tube` | Redstone Tube | `BlockMicro` damage 2304 |
| `redpower_2:restrictor_tube` | Restrictor Tube | `BlockMicro` damage 2560 |
| `redpower_2:magtube` | Magtube | `BlockMicro` damage 2816 |
| `redpower_2:ribbon_cable` | Ribbon Cable | `BlockMicro` damage 3072 |
| `redpower_2:timer` | Timer | `BlockLogic` meta 0 |
| `redpower_2:sequencer` | Sequencer | `BlockLogic` meta 1 |
| `redpower_2:state_cell` | State Cell | `BlockLogic` meta 2 |
| `redpower_2:rs_latch` | RS Latch | `BlockLogic` meta 256 |
| `redpower_2:nor_gate` | NOR Gate | `BlockLogic` meta 257 |
| `redpower_2:or_gate` | OR Gate | `BlockLogic` meta 258 |
| `redpower_2:nand_gate` | NAND Gate | `BlockLogic` meta 259 |
| `redpower_2:and_gate` | AND Gate | `BlockLogic` meta 260 |
| `redpower_2:xnor_gate` | XNOR Gate | `BlockLogic` meta 261 |
| `redpower_2:xor_gate` | XOR Gate | `BlockLogic` meta 262 |
| `redpower_2:pulse_former` | Pulse Former | `BlockLogic` meta 263 |
| `redpower_2:toggle_latch` | Toggle Latch | `BlockLogic` meta 264 |
| `redpower_2:not_gate` | NOT Gate / Inverter | `BlockLogic` meta 265 |
| `redpower_2:buffer_gate` | Buffer Gate | `BlockLogic` meta 266 |
| `redpower_2:multiplexer` | Multiplexer | `BlockLogic` meta 267 |
| `redpower_2:repeater` | Repeater | `BlockLogic` meta 268 |
| `redpower_2:synchronizer` | Synchronizer | `BlockLogic` meta 269 |
| `redpower_2:randomizer` | Randomizer | `BlockLogic` meta 270 |
| `redpower_2:transparent_latch` | Transparent Latch | `BlockLogic` meta 271 |
| `redpower_2:light_sensor` | Light Sensor | `BlockLogic` meta 272 |
| `redpower_2:null_cell` | Null Cell | `BlockLogic` meta 512 |
| `redpower_2:invert_cell` | Invert Cell | `BlockLogic` meta 513 |
| `redpower_2:non_invert_cell` | Non-Invert Cell | `BlockLogic` meta 514 |
| `redpower_2:counter` | Counter | `BlockLogic` meta 768 |
| `redpower_2:bus_transceiver` | Bus Transceiver | `BlockLogic` meta 1024 |
| `redpower_2:lamp` | Lamp (16 colors) | `BlockLamp` off |
| `redpower_2:lamp_lit` | Lamp Lit (16 colors) | `BlockLamp` on |
| `redpower_2:lamp_inverted` | Inverted Lamp (16 colors) | `BlockLamp` inv off |
| `redpower_2:lamp_inverted_lit` | Inverted Lamp Lit (16 colors) | `BlockLamp` inv on |
| `redpower_2:lumar_fixture` | Lumar Fixture (16 colors) | `BlockShapedLamp` meta 0..15 |
| `redpower_2:lumar_fixture_inverted` | Inverted Fixture (16 colors) | `BlockShapedLamp` meta 16..31 |
| `redpower_2:cage_lamp` | Cage Lamp (16 colors) | `BlockShapedLamp` meta 32..47 |
| `redpower_2:cage_lamp_inverted` | Inverted Cage Lamp (16 colors) | `BlockShapedLamp` meta 48..63 |
| `redpower_2:ore_ruby` | Ruby Ore | `BlockCustomOre` meta 0 |
| `redpower_2:ore_green_sapphire` | Green Sapphire Ore | `BlockCustomOre` meta 1 |
| `redpower_2:ore_sapphire` | Sapphire Ore | `BlockCustomOre` meta 2 |
| `redpower_2:ore_silver` | Silver Ore | `BlockCustomOre` meta 3 |
| `redpower_2:ore_tin` | Tin Ore | `BlockCustomOre` meta 4 |
| `redpower_2:ore_copper` | Copper Ore | `BlockCustomOre` meta 5 |
| `redpower_2:ore_tungsten` | Tungsten Ore | `BlockCustomOre` meta 6 |
| `redpower_2:ore_nikolite` | Nikolite Ore | `BlockCustomOre` meta 7 |
| `redpower_2:block_ruby` | Ruby Block | `BlockStorage` meta 0 |
| `redpower_2:block_green_sapphire` | Green Sapphire Block | `BlockStorage` meta 1 |
| `redpower_2:block_sapphire` | Sapphire Block | `BlockStorage` meta 2 |
| `redpower_2:block_silver` | Silver Block | `BlockStorage` meta 3 |
| `redpower_2:block_tin` | Tin Block | `BlockStorage` meta 4 |
| `redpower_2:block_copper` | Copper Block | `BlockStorage` meta 5 |
| `redpower_2:marble` | Marble | `BlockCustomStone` meta 0 |
| `redpower_2:basalt` | Basalt | `BlockCustomStone` meta 1 |
| `redpower_2:marble_brick` | Marble Brick | `BlockCustomStone` meta 2 |
| `redpower_2:basalt_cobblestone` | Basalt Cobblestone | `BlockCustomStone` meta 3 |
| `redpower_2:basalt_brick` | Basalt Brick | `BlockCustomStone` meta 4 |
| `redpower_2:basalt_chiseled` | Chiseled Basalt Brick | `BlockCustomStone` meta 5 |
| `redpower_2:basalt_paver` | Basalt Paver | `BlockCustomStone` meta 6 |
| `redpower_2:flax_crop` | Flax Crop | `BlockCustomCrops` meta 0..5 |
| `redpower_2:indigo_flower` | Indigo Flower | `BlockCustomFlower` meta 0 |
| `redpower_2:rubber_sapling` | Rubber Sapling | `BlockCustomFlower` meta 1 |
| `redpower_2:rubber_leaves` | Rubber Leaves | `BlockCustomLeaves` |
| `redpower_2:rubber_log` | Rubber Wood | `BlockCustomLog` |
| `redpower_2:deployer` | Deployer | `BlockMachine` meta 0 |
| `redpower_2:block_breaker` | Block Breaker | `BlockMachine` meta 1 |
| `redpower_2:transposer` | Transposer | `BlockMachine` meta 2 |
| `redpower_2:filter` | Filter | `BlockMachine` meta 3 |
| `redpower_2:item_detector` | Item Detector | `BlockMachine` meta 4 |
| `redpower_2:sorting_machine` | Sorting Machine | `BlockMachine` meta 5 |
| `redpower_2:battery_box` | Battery Box | `BlockMachine` meta 6 |
| `redpower_2:frame_motor` | Frame Motor | `BlockMachine` meta 7 |
| `redpower_2:retriever` | Retriever | `BlockMachine` meta 8 |
| `redpower_2:wind_turbine` | Wind Turbine | `BlockMachine` meta 9 |
| `redpower_2:regulator` | Regulator | `BlockMachine` meta 10 |
| `redpower_2:thermopile` | Thermopile | `BlockMachine` meta 11 |
| `redpower_2:igniter` | Igniter | `BlockMachine` meta 12 |
| `redpower_2:assembler` | Assembler | `BlockMachine` meta 13 |
| `redpower_2:ejector` | Ejector | `BlockMachine` meta 14 |
| `redpower_2:relay` | Relay | `BlockMachine` meta 15 |
| `redpower_2:sortron` | Sortron | `BlockMachine2` meta 0 |
| `redpower_2:manager` | Manager | `BlockMachine2` meta 1 |
| `redpower_2:solar_panel` | Solar Panel | `BlockMachinePanel` meta 0 |
| `redpower_2:pump` | Pump | `BlockMachinePanel` meta 1 |
| `redpower_2:accelerator` | Accelerator | `BlockMachinePanel` meta 2 |
| `redpower_2:grate` | Grate | `BlockMachinePanel` meta 3 |
| `redpower_2:transformer` | Transformer | `BlockMachinePanel` meta 4 |
| `redpower_2:frame` | Support Frame | `BlockFrame` meta 0 |
| `redpower_2:frame_tube` | Tube Frame | `BlockFrame` meta 2 |
| `redpower_2:frame_redstone_tube` | Redstone Tube Frame | `BlockFrame` meta 3 |
| `redpower_2:backplane` | Backplane | `BlockMultipart` meta 0 |
| `redpower_2:ram_module` | 8K RAM Module | `BlockMultipart` meta 1 |
| `redpower_2:display` | Monitor / Display | `BlockPeripheral` meta 0 |
| `redpower_2:cpu` | CPU | `BlockPeripheral` meta 1 |
| `redpower_2:disk_drive` | Disk Drive | `BlockPeripheral` meta 2 |
| `redpower_2:io_expander` | IO Expander | `BlockMultipart` meta 0 |
| `redpower_2:blulectric_engine` | Blulectric Engine | `BlockMachineCompat` meta 0 |

### 6.2 Items (88 entries)

| Rekkit ID | Name | Original Java Mapping |
| :--- | :--- | :--- |
| `redpower_2:lumar` | Lumar (16 colors) | `ItemParts` itemLumar meta 0..15 |
| `redpower_2:ruby` | Ruby | `ItemParts` itemResource meta 0 |
| `redpower_2:green_sapphire` | Green Sapphire | `ItemParts` itemResource meta 1 |
| `redpower_2:sapphire` | Sapphire | `ItemParts` itemResource meta 2 |
| `redpower_2:silver_ingot` | Silver Ingot | `ItemParts` itemResource meta 3 |
| `redpower_2:tin_ingot` | Tin Ingot | `ItemParts` itemResource meta 4 |
| `redpower_2:copper_ingot` | Copper Ingot | `ItemParts` itemResource meta 5 |
| `redpower_2:nikolite` | Nikolite | `ItemParts` itemResource meta 6 |
| `redpower_2:red_alloy_ingot` | Red Alloy Ingot | `ItemParts` itemAlloy meta 0 |
| `redpower_2:blue_alloy_ingot` | Blue Alloy Ingot | `ItemParts` itemAlloy meta 1 |
| `redpower_2:brass_ingot` | Brass Ingot | `ItemParts` itemAlloy meta 2 |
| `redpower_2:silicon_boule` | Silicon Boule | `ItemParts` itemAlloy meta 3 |
| `redpower_2:silicon_wafer` | Silicon Wafer | `ItemParts` itemAlloy meta 4 |
| `redpower_2:blue_doped_wafer` | Blue-Doped Wafer | `ItemParts` itemAlloy meta 5 |
| `redpower_2:red_doped_wafer` | Red-Doped Wafer | `ItemParts` itemAlloy meta 6 |
| `redpower_2:tinplate` | Tinplate | `ItemParts` itemAlloy meta 7 |
| `redpower_2:fine_copper_wire` | Fine Copper Wire | `ItemParts` itemAlloy meta 8 |
| `redpower_2:fine_iron_wire` | Fine Iron Wire | `ItemParts` itemAlloy meta 9 |
| `redpower_2:copper_coil` | Copper Coil | `ItemParts` itemAlloy meta 10 |
| `redpower_2:bl_motor` | BL Motor | `ItemParts` itemAlloy meta 11 |
| `redpower_2:canvas` | Canvas | `ItemParts` itemAlloy meta 12 |
| `redpower_2:iron_nugget` | Iron Nugget | `ItemParts` itemNugget meta 0 |
| `redpower_2:silver_nugget` | Silver Nugget | `ItemParts` itemNugget meta 1 |
| `redpower_2:tin_nugget` | Tin Nugget | `ItemParts` itemNugget meta 2 |
| `redpower_2:copper_nugget` | Copper Nugget | `ItemParts` itemNugget meta 3 |
| `redpower_2:dye_indigo` | Indigo Dye | `ItemDyeIndigo` |
| `redpower_2:iron_handsaw` | Iron Handsaw | `ItemHandsaw` iron |
| `redpower_2:diamond_handsaw` | Diamond Handsaw | `ItemHandsaw` diamond |
| `redpower_2:screwdriver` | Screwdriver | `ItemScrewdriver` |
| `redpower_2:diamond_drawplate` | Diamond Drawplate | `ItemDrawplate` |
| `redpower_2:blank_plan` | Blank Plan | `ItemTextured` planBlank |
| `redpower_2:plan` | Plan | `ItemPlan` |
| `redpower_2:canvas_bag` | Canvas Bag (16 colors) | `ItemBag` meta 0..15 |
| `redpower_2:stone_wafer` | Stone Wafer | `ItemParts` itemParts meta 0 |
| `redpower_2:wire_part` | Wire Component | `ItemParts` itemParts meta 1 |
| `redpower_2:anode` | Anode | `ItemParts` itemParts meta 2 |
| `redpower_2:cathode` | Cathode | `ItemParts` itemParts meta 3 |
| `redpower_2:pointer` | Pointer | `ItemParts` itemParts meta 4 |
| `redpower_2:redwire_part` | Redwire Part | `ItemParts` itemParts meta 5 |
| `redpower_2:plate` | Plate Part | `ItemParts` itemParts meta 6 |
| `redpower_2:silicon_chip` | Silicon Chip | `ItemParts` itemParts meta 7 |
| `redpower_2:tainted_chip` | Tainted Chip | `ItemParts` itemParts meta 8 |
| `redpower_2:bundled_wafer` | Bundled Wafer | `ItemParts` itemParts meta 9 |
| `redpower_2:ruby_pickaxe` | Ruby Pickaxe | `ItemCustomPickaxe` |
| `redpower_2:green_sapphire_pickaxe` | Green Sapphire Pickaxe | `ItemCustomPickaxe` |
| `redpower_2:sapphire_pickaxe` | Sapphire Pickaxe | `ItemCustomPickaxe` |
| `redpower_2:ruby_shovel` | Ruby Shovel | `ItemCustomShovel` |
| `redpower_2:green_sapphire_shovel` | Green Sapphire Shovel | `ItemCustomShovel` |
| `redpower_2:sapphire_shovel` | Sapphire Shovel | `ItemCustomShovel` |
| `redpower_2:ruby_axe` | Ruby Axe | `ItemCustomAxe` |
| `redpower_2:green_sapphire_axe` | Green Sapphire Axe | `ItemCustomAxe` |
| `redpower_2:sapphire_axe` | Sapphire Axe | `ItemCustomAxe` |
| `redpower_2:ruby_sword` | Ruby Sword | `ItemCustomSword` |
| `redpower_2:green_sapphire_sword` | Green Sapphire Sword | `ItemCustomSword` |
| `redpower_2:sapphire_sword` | Sapphire Sword | `ItemCustomSword` |
| `redpower_2:ruby_hoe` | Ruby Hoe | `ItemCustomHoe` |
| `redpower_2:green_sapphire_hoe` | Green Sapphire Hoe | `ItemCustomHoe` |
| `redpower_2:sapphire_hoe` | Sapphire Hoe | `ItemCustomHoe` |
| `redpower_2:ruby_handsaw` | Ruby Handsaw | `ItemHandsaw` ruby |
| `redpower_2:green_sapphire_handsaw` | Green Sapphire Handsaw | `ItemHandsaw` green sapphire |
| `redpower_2:sapphire_handsaw` | Sapphire Handsaw | `ItemHandsaw` sapphire |
| `redpower_2:wood_sickle` | Wood Sickle | `ItemSickle` wood |
| `redpower_2:stone_sickle` | Stone Sickle | `ItemSickle` stone |
| `redpower_2:iron_sickle` | Iron Sickle | `ItemSickle` iron |
| `redpower_2:diamond_sickle` | Diamond Sickle | `ItemSickle` diamond |
| `redpower_2:gold_sickle` | Gold Sickle | `ItemSickle` gold |
| `redpower_2:ruby_sickle` | Ruby Sickle | `ItemSickle` ruby |
| `redpower_2:green_sapphire_sickle` | Green Sapphire Sickle | `ItemSickle` green sapphire |
| `redpower_2:sapphire_sickle` | Sapphire Sickle | `ItemSickle` sapphire |
| `redpower_2:athame` | Athame | `ItemAthame` |
| `redpower_2:flax_seeds` | Flax Seeds | `ItemCustomSeeds` |
| `redpower_2:wool_card` | Wool Card | `ItemWoolCard` |
| `redpower_2:dry_paint_brush` | Dry Paint Brush | `ItemTextured` paintbrush.dry |
| `redpower_2:paint_brush` | Paint Brush (16 colors) | `ItemPaintBrush` 16 colors |
| `redpower_2:empty_paint_can` | Empty Paint Can | `ItemTextured` paintcan.empty |
| `redpower_2:paint_can` | Paint Can (16 colors) | `ItemPaintCan` 16 colors |
| `redpower_2:seed_bag` | Seed Bag | `ItemSeedBag` |
| `redpower_2:voltmeter` | Voltmeter | `ItemVoltmeter` |
| `redpower_2:sonic_screwdriver` | Sonic Screwdriver | `ItemSonicDriver` |
| `redpower_2:battery_empty` | BT Battery (Empty) | `ItemTextured` btbattery |
| `redpower_2:battery_powered` | BT Battery (Powered) | `ItemBattery` |
| `redpower_2:wood_sail` | Wooden Windmill Sail | `ItemParts` itemMachineParts meta 0 |
| `redpower_2:wood_turbine` | Wooden Wind Turbine | `ItemWindmill` type 1 |
| `redpower_2:wood_windmill` | Wooden Windmill | `ItemWindmill` type 2 |
| `redpower_2:blank_disk` | Blank Floppy Disk | `ItemDisk` meta 0 |
| `redpower_2:forth_boot_disk` | RedPower Forth Boot Disk | `ItemDisk` meta 1 |
| `redpower_2:forth_extra_disk` | RedPower Forth Extra Disk | `ItemDisk` meta 2 |
| `redpower_2:brass_gear` | Brass Gear | `ItemParts` itemCompatParts meta 0 |

---

## 7. Recipes

### 7.1 Custom Machine Recipes (`"type": "redpower_2:alloy_furnace"`)
* **Red Alloy Ingot (`redpower_2:alloy_red_alloy`):** 4x `minecraft:redstone` + 1x `redpower_2:copper_ingot` (or 1x `minecraft:iron_ingot`) $\rightarrow$ 1x `redpower_2:red_alloy_ingot`.
* **Blue Alloy Ingot (`redpower_2:alloy_blue_alloy`):** 4x `redpower_2:nikolite` + 1x `redpower_2:silver_ingot` $\rightarrow$ 1x `redpower_2:blue_alloy_ingot`.
* **Brass Ingot (`redpower_2:alloy_brass`, x4):** 3x `redpower_2:copper_ingot` + 1x `redpower_2:tin_ingot` $\rightarrow$ 4x `redpower_2:brass_ingot`.
* **Tinplate (`redpower_2:alloy_tinplate`, x4):** 2x `minecraft:iron_ingot` + 1x `redpower_2:tin_ingot` $\rightarrow$ 4x `redpower_2:tinplate`.
* **Silicon Boule (`redpower_2:alloy_silicon_boule`):** 8x `minecraft:coal` (or charcoal) + 8x `minecraft:sand` $\rightarrow$ 1x `redpower_2:silicon_boule`.
* **Blue-Doped Wafer (`redpower_2:alloy_blue_doped_wafer`):** 1x `redpower_2:silicon_wafer` + 4x `redpower_2:nikolite` $\rightarrow$ 1x `redpower_2:blue_doped_wafer`.
* **Red-Doped Wafer (`redpower_2:alloy_red_doped_wafer`):** 1x `redpower_2:silicon_wafer` + 4x `minecraft:redstone` $\rightarrow$ 1x `redpower_2:red_doped_wafer`.
* **Iron Recycling (`CraftLib.java:231..258`):**
  * 8x `minecraft:iron_bars` $\rightarrow$ 3x `minecraft:iron_ingot`
  * 1x `minecraft:minecart` $\rightarrow$ 5x `minecraft:iron_ingot`
  * 1x `minecraft:iron_door` $\rightarrow$ 6x `minecraft:iron_ingot`
  * 1x `minecraft:cauldron` $\rightarrow$ 7x `minecraft:iron_ingot`
  * 1x `minecraft:bucket` $\rightarrow$ 3x `minecraft:iron_ingot`
  * 1x `minecraft:shears` $\rightarrow$ 2x `minecraft:iron_ingot`
  * Iron Tools: Sword $\rightarrow$ 2 ingots; Pickaxe $\rightarrow$ 3 ingots; Axe $\rightarrow$ 3 ingots; Shovel $\rightarrow$ 1 ingot; Hoe $\rightarrow$ 2 ingots.
  * Iron Armor: Helmet $\rightarrow$ 5 ingots; Chestplate $\rightarrow$ 8 ingots; Leggings $\rightarrow$ 7 ingots; Boots $\rightarrow$ 4 ingots.
  * 9x `redpower_2:iron_nugget` $\rightarrow$ 1x `minecraft:iron_ingot`.
* **Gold Recycling (`CraftLib.java:244..256`):**
  * Gold Tools: Sword $\rightarrow$ 2 ingots; Pickaxe $\rightarrow$ 3 ingots; Axe $\rightarrow$ 3 ingots; Shovel $\rightarrow$ 1 ingot; Hoe $\rightarrow$ 2 ingots.
  * Gold Armor: Helmet $\rightarrow$ 5 ingots; Chestplate $\rightarrow$ 8 ingots; Leggings $\rightarrow$ 7 ingots; Boots $\rightarrow$ 4 ingots.
* **Fine Wire Recycling:**
  * 1x `redpower_2:fine_copper_wire` $\rightarrow$ 1x `redpower_2:copper_ingot`
  * 1x `redpower_2:fine_iron_wire` $\rightarrow$ 1x `minecraft:iron_ingot`

### 7.2 Smelting Recipes (`"type": "minecraft:smelting"`)
* `redpower_2:ore_silver` $\rightarrow$ `redpower_2:silver_ingot` (XP 1.0)
* `redpower_2:ore_tin` $\rightarrow$ `redpower_2:tin_ingot` (XP 0.7)
* `redpower_2:ore_copper` $\rightarrow$ `redpower_2:copper_ingot` (XP 0.7)
* `redpower_2:basalt_cobblestone` $\rightarrow$ `redpower_2:basalt` (XP 0.2)
* `redpower_2:rubber_log` $\rightarrow$ `minecraft:charcoal` (XP 0.15)
* `minecraft:stone` $\rightarrow$ 2x `redpower_2:stone_wafer` (XP 0.1)

### 7.3 Key Crafting Recipes (`"type": "minecraft:crafting_shaped"` / `_shapeless`)
* **Red Alloy Wire (x12):** Row of 3x `redpower_2:red_alloy_ingot`.
* **Insulated Wire (x12):** 3x `redpower_2:red_alloy_ingot` flanked by 6x matching colored `minecraft:wool`.
* **Bundled Cable (x2):** 5x `redpower_2:wire_insulated` + 4x `minecraft:string`.
* **Iron Handsaw:** 3x `minecraft:stick` + 4x `minecraft:iron_ingot`.
* **Diamond Handsaw:** 3x `minecraft:stick` + 2x `minecraft:iron_ingot` + 2x `minecraft:diamond`.
* **Gem Handsaws (Ruby, Sapphire, Green Sapphire):** 3x `minecraft:stick` + 2x `minecraft:iron_ingot` + 2x Gem.
* **Screwdriver:** 1x `minecraft:iron_ingot` + 1x `minecraft:stick`.
* **Diamond Drawplate:** 4x Iron Cover microblocks + 1x Diamond Panel microblock.
* **Fine Copper Wire:** `redpower_2:copper_ingot` + `redpower_2:diamond_drawplate` (shapeless).
* **Fine Iron Wire:** `minecraft:iron_ingot` + `redpower_2:diamond_drawplate` (shapeless).
* **Silicon Wafer (x16):** `redpower_2:silicon_boule` + `redpower_2:diamond_handsaw` (shapeless).
* **Alloy Furnace:** 8x `minecraft:brick` around empty center.
* **Project Table:** 3x `minecraft:stone` + 4x `minecraft:planks` + 1x `minecraft:crafting_table` + 1x `minecraft:chest`.
* **Pneumatic Tube (x8):** 2x `redpower_2:brass_ingot` + 1x `minecraft:glass`.
* **Restrictor Tube:** 1x `redpower_2:pneumatic_tube` + 1x `minecraft:iron_ingot` (shapeless).
* **Redstone Tube:** 1x `redpower_2:pneumatic_tube` + 1x `minecraft:redstone` (shapeless).
* **Magtube (x8):** 6x `redpower_2:fine_copper_wire` + 2x `minecraft:obsidian` + 1x `minecraft:glass`.
* **Deployer:** 6x `minecraft:cobblestone` + 1x `minecraft:chest` + 1x `minecraft:piston` + 1x `minecraft:redstone`.
* **Block Breaker:** 6x `minecraft:cobblestone` + 1x `minecraft:iron_pickaxe` + 1x `minecraft:piston` + 1x `minecraft:redstone`.
* **Transposer:** 6x `minecraft:cobblestone` + 2x `minecraft:planks` + 1x `minecraft:piston` + 1x `minecraft:redstone`.
* **Filter:** 6x `minecraft:cobblestone` + 2x `minecraft:gold_ingot` + 1x `minecraft:piston` + 1x `redpower_2:red_doped_wafer`.
* **Item Detector:** 2x `redpower_2:brass_ingot` + 1x `redpower_2:pneumatic_tube` + 2x `redpower_2:red_doped_wafer` + 2x `minecraft:planks` + 1x `minecraft:stone_pressure_plate`.
* **Sorting Machine:** 4x `minecraft:iron_ingot` + 2x `redpower_2:red_doped_wafer` + 1x `redpower_2:filter` + 1x `redpower_2:blue_alloy_ingot`.
* **Battery Box:** 4x `redpower_2:battery_empty` + 1x `minecraft:planks` + 2x `minecraft:iron_ingot` + 1x `redpower_2:blue_alloy_ingot`.
* **Frame Motor:** 4x `minecraft:iron_ingot` + 2x `redpower_2:brass_ingot` + 1x `redpower_2:bl_motor` + 1x `redpower_2:blue_alloy_ingot`.
* **Support Frame:** 8x `minecraft:stick` + 1x `redpower_2:brass_ingot`.
* **Solar Panel:** 8x `redpower_2:blue_doped_wafer` + 1x `redpower_2:blue_alloy_ingot`.
* **Thermopile:** 4x `redpower_2:copper_ingot` + 2x `minecraft:iron_ingot` + 2x `redpower_2:blue_doped_wafer` + 1x `redpower_2:blue_alloy_ingot`.
* **BL Motor:** 4x `minecraft:iron_ingot` + 2x `redpower_2:copper_coil` + 1x `redpower_2:blue_alloy_ingot`.
* **Copper Coil:** 4x `redpower_2:fine_copper_wire` + 4x `minecraft:iron_bars` + 1x `minecraft:iron_ingot`.
* **Athame:** 1x `redpower_2:silver_ingot` + 1x `minecraft:stick`.
* **Canvas Bag:** 8x `redpower_2:canvas` around empty center.
* **Canvas:** 8x `minecraft:string` around 1x `minecraft:stick`.
* **Seed Bag:** 5x `redpower_2:canvas` + 1x `minecraft:string`.
* **Wool Card:** 1x `redpower_2:fine_iron_wire` + 1x `minecraft:planks` + 1x `minecraft:stick`.
* **Empty Paint Can (x3):** 7x `redpower_2:tinplate`.
* **Paint Can (colored):** 1x `redpower_2:empty_paint_can` + 1x vanilla dye + 2x `redpower_2:flax_seeds` (shapeless).
* **Paint Brush:** 1x `redpower_2:paint_can` + 1x `redpower_2:dry_paint_brush` (shapeless).
* **CPU:** 4x `minecraft:planks` + 1x `minecraft:diamond_block` + 2x `redpower_2:red_doped_wafer` + 1x `redpower_2:ribbon_cable`.
* **Monitor / Display:** 4x `minecraft:planks` + 3x `minecraft:glass` + 1x `redpower_2:lumar` + 1x `redpower_2:red_doped_wafer` + 1x `redpower_2:ribbon_cable`.
* **Disk Drive:** 5x `minecraft:planks` + 1x `redpower_2:bl_motor` + 1x `redpower_2:red_doped_wafer` + 1x `redpower_2:ribbon_cable`.
* **IO Expander:** 4x `minecraft:planks` + 1x `redpower_2:wire_bundled` + 1x `redpower_2:red_doped_wafer` + 1x `redpower_2:ribbon_cable`.
* **Floppy Disk:** 7x `minecraft:planks` + 1x `minecraft:iron_ingot`.
* **Blulectric Engine:** 4x `redpower_2:brass_ingot` + 1x `minecraft:glass` + 2x `redpower_2:brass_gear` + 1x `redpower_2:bl_motor`.

---

## 8. Hazards & Safety Limits
* **6502 CPU Runaway Cycles:** Unbounded Forth/6502 loops could freeze server ticks. Bounded to a time-slice of $+1,000$ instruction cycles per world tick, with a maximum burst buffer of $100,000$ cycles (`TileCPU.java:198..205`).
* **Frame Motor Collision & Bedrock:** Frame Motors refuse to move assemblies exceeding $1,000$ blocks (`FrameLinkSize`) or assemblies containing unbreakable blocks (`hardness < 0.0F` like `minecraft:bedrock`). Obsidian has hardness 50.0F and CAN be moved.
* **Thermopile Thermal Meltdown / Shock:** Operating with hot temperatures $\ge 100$ and cold temperatures $\ge 200$ creates a 1/300 chance per tick for adjacent lava to solidify into obsidian or basalt, and snow/ice to melt into water/air (`TileThermopile.java:84..113`).
* **Blutricity Over-Voltage:** Wind turbines can reach $130\text{V}$; peak safe grid voltage is $120\text{V}$; Transformers step voltage to $10,000\text{V}$.
* **Vorpal Overkill Decapitation:** Overkill damage exceeding $\le -20\text{ HP}$ drops victim heads (`WorldEvents.java:41`).

---

## 9. Recreation Implementation Notes for Rekkit (Rust)
1. **Sub-Voxel Multi-Part Architecture:** Use a bitmask (6 faces, 12 edges, 8 vertices, 1 center) inside `ModBlockEntity.nbt` to represent compound microblocks, surface wiring, and flat logic tiles co-existing in a single voxel space.
2. **Tube Network Pathfinding:** Integrate Dijkstra route finding with Restrictor Tube ($+1,000,000$ weight), $+2$ step distance, and color compatibility matching into `rekkit_core::grid::GridGraph`.
3. **Frame Movement Physics:** Collect contiguous blocks via BFS (`FrameSolver`), validate hardness $\ge 0.0\text{F}$, convert blocks to moving placeholders in 1 tick, animate over 16 ticks, and re-place atomically via `Game::native_set_block` (`API 24`).
4. **6502/65C816 Microprocessor State Machine:** Implement a pure Rust emulator (`Cpu6502`) executing instructions with bounded 1,000 cycles/tick slices, 64 KB memory banking, and Redbus peripheral dispatch.
5. **Blutricity Dynamic Solver:** Implement RLC iterative network simulation (`BluePowerConductor`) maintaining $V$, $I$, and $Q$ across $0.01\,\Omega$ conductors with $0.07$ inductive scaling.

---

## 10. Hooks Needed (`NATIVE_HOOKS_API = 40`)

| Horizon Hook API | Entrypoint in `native_hooks.rs` | RedPower 2 Feature |
| :--- | :--- | :--- |
| **API 12** | `Game::native_tooltip` | Voltmeter voltage/charge tooltips, canvas bag item counts, sickle area of effect. |
| **API 13** | `Game::native_use_item_on`, `Game::native_use_item` | Screwdriver gate rotation/mode cycling, Paintbrush coloring, Handsaw slicing, Seed Bag planting. |
| **API 15** | `Game::native_block_entity`, `Game::native_block_entity_ticks` | Attach and tick block entities for all 24 machines, logic tiles, tubes, frames, and CPU. |
| **API 16–17** | `Game::native_block_entity_container`, `Game::native_block_entity_tick` | 24 machine inventory screens, Alloy Furnace smelting, Project Table crafting, Buffer routing. |
| **API 18** | `Game::native_with_mod_block_entity`, `Game::native_set_block_property` | Blutricity voltage equalization, fluid pipe pressure propagation, pneumatic tube item transfer, lamp lit states. |
| **API 19–20** | `Game::native_screen`, `Game::native_screen_click`, `Game::native_screen_tooltip`, `Game::native_menu_button`, `Game::native_insert_item` | Alloy Furnace, Project Table, Sorting Machine, Timer, Counter, Seed Bag, and Terminal screens. |
| **API 24** | `Game::native_set_block` | Block Breaker harvesting, Deployer placement, Frame Motor assembly translation. |
| **API 32** | `Game::native_update_shape`, `Game::native_schedule_block_tick` | Wire/cable/tube 6-face neighbor connectivity, logic gate tick scheduling. |
| **API 33–34** | `Game::native_random_tick`, `Game::native_tick_block` | Rubber tree sapling and Flax crop growth; logic gate scheduled pulse evaluation. |
| **API 36** | `Game::native_recipes_of_type`, `Game::native_recipe_ingredient`, `Game::native_recipe_result` | Custom `"type": "redpower_2:alloy_furnace"` metallurgical recipes. |
| **API 38** | `Game::native_block_signal`, `Game::native_is_signal_source`, `Game::native_update_signal_neighbors` | Red Alloy wire signal transmission (0..255 mapped to 0..15), logic gate outputs, Item Detector pulses. |
| **API 39** | `Game::native_hurt_enemy` | Athame 25 damage vs Endermen; Vorpal critical strike (100 damage) and head decapitation. |
| **API 40** | `Game::native_net_from_client`, `Game::native_net_send_to_player` | 6502 terminal keystrokes and 80×50 screen buffer synchronization on `horizon:mod_net`. |

---

## 11. Uses from `rekkit_core`
* `rekkit_core::energy::BlutricityNode`: 100V nominal, 60V threshold, 120V peak safe, Ohm's law charge flow ($I = (V_a - V_b)/R$).
* `rekkit_core::frequency::{DyeColor, BundledSignal}`: 16-color insulated wires, 16-channel bundled cables, and 6502 I/O Expanders.
* `rekkit_core::grid::{BlockPos, GridGraph, GridNode, UnionFind}`: Pneumatic tube network topology, Dijkstra shortest-path pathfinding, and wire cluster union-find.
* `rekkit_core::side::{Direction, ConnectionMask}`: 6-direction connection bitmasks and machine rotation orientations.
* `rekkit_core::inventory::{Inventory, ItemStack}`: Fixed-size inventories for Alloy Furnace (11 slots), Project Table (28 slots), Buffer Chest (20 slots), Sorter (40 slots), Canvas Bag (27 slots).
* `rekkit_core::hooks::{HorizonNativeMod, ModBlockEntity, ModContainer, NativeModInfo, NativeScreen}`: Native mod trait implementations.
* `rekkit_core::util::{TickTimer, XorShift64}`: Logic gate timers and Vorpal roll evaluation.
* **Gaps / What `rekkit_core` lacks ("new: ..."):**
  * `new: rekkit_core::microblock::MicroblockVoxel` (sub-voxel 6-face, 12-edge, 8-vertex compound bounding box and occlusion bitmask shared with other multi-part mods).
  * `new: rekkit_core::energy::BlutricityConductor` (full dynamic inductive/RLC circuit simulation matching `BluePowerConductor` with resistance, capacitance, and 0.07 inductive scaling).
  * `new: rekkit_core::grid::GridNode::restrictor_weight` update to 1,000,000 (currently 500 in rekkit_core).

---

## 12. Open Questions & Verifications
* **Asset Mapping:** Sprite coordinates from legacy sheets (`eloraam/base/items1.png`, `eloraam/lighting/lighting1.png`, `eloraam/machine/machine1.png`, `eloraam/world/world1.png`, `eloraam/world/worlditems1.png`, `eloraam/compat/compat1.png`) are mapped in `mods/redpower_2/asset-map.json` and verified against the decompiled 2.0pr6 jar.
* **6502 ROM Storage:** Disk image files (`redforth.img`, `redforthxp.img`, `rpcboot.bin`) are verified in `assets/decompiled/redpowermechanical/eloraam/control/` and read into virtual disk memory.
