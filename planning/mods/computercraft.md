# ComputerCraft - Technical Specification & Recreation Guide

## 1. Overview & Version Target
* **Target Version:** ComputerCraft v1.53 / v1.481 for Minecraft 1.4.7 / 1.2.5 (the original classic Lua 5.1 engine release created by dan200).
* **Author:** dan200.
* **Core Philosophy:** Programmability, automation, and networking via an authentic embedded Lua 5.1 runtime. Introduces in-game desktop computers, mobile programmable robots (Turtles), multiblock monitors, disk drives, printers, and redstone/bundled cable integration.

---

## 2. Hardware Variants & Terminals

### 2.1 Computer (Standard)
* **Chassis:** Stone/Iron casing.
* **Display:** 51 columns $\times$ 19 rows monochrome terminal (green-on-black or white-on-black).
* **Execution:** Runs CraftOS in an isolated Lua coroutine.
* **Peripherals:** Detects peripherals placed adjacent to any of its 6 faces (top, bottom, left, right, front, back).

### 2.2 Advanced Computer
* **Chassis:** Gold casing.
* **Display:** 51 columns $\times$ 19 rows terminal supporting the full 16-color palette (text and background colors).
* **Mouse Input:** Supports interactive GUI clicking; fires `mouse_click`, `mouse_drag`, and `mouse_scroll` events with coordinate offsets $(X, Y)$.

### 2.3 Pocket Computer
* **Form Factor:** Handheld item.
* **Display:** 26 columns $\times$ 20 rows portable terminal screen accessed via right-click.

### 2.4 Turtles (Automated Mobile Robotics)
* **Chassis:** Standard (Stone) or Advanced (Gold).
* **Internal Storage:** 16 inventory slots in a 4x4 grid.
* **Fuel Mechanic:**
  * Turtles consume solid fuels (Coal, Wood, Blaze Rods, Lava Buckets) to maintain motion.
  * Movement costs 1 Fuel Unit per block step.
  * Max Fuel Limit: 20,000 units (Standard) / 100,000 units (Advanced).
* **Peripheral Slots:** 2 hardware upgrade slots (Left and Right sides):
  * **Mining Turtle:** Diamond Pickaxe. Can break any block, including Obsidian.
  * **Melee Turtle:** Diamond Sword. Attacks entities dealing sword weapon damage and collects mob drops.
  * **Digging Turtle:** Diamond Shovel. Rapidly clears dirt, gravel, sand, and clay.
  * **Felling Turtle:** Diamond Axe. Harvests logs and leaves.
  * **Farming Turtle:** Diamond Hoe. Tills soil, harvests and replants crops.
  * **Crafting Turtle:** Crafting Table. Crafts items using its 16-slot grid via `turtle.craft()`.
  * **Wireless Turtle:** Wireless Modem. Enables remote rednet communication.

---

## 3. Peripheral Hardware & Networking

### 3.1 Monitors
* **Multiblock Construction:** Monitors connect seamlessly into flat rectangular display walls up to **8 blocks wide by 6 blocks high**.
* **Resolution:** Auto-scales font size and resolution depending on grid dimensions.
* **Advanced Monitor:** Offers 16 colors and touch-screen mouse click interaction.

### 3.2 Disk Drive & Floppy Disks
* **Disk Drive:** 1-block peripheral. Holds 1 Floppy Disk or Music Disc.
* **Mounting:** Mounted automatically into the computer's virtual file system as `/disk`, `/disk2`, etc.
* **Capacity:** Floppy disks store 128 KB of persistent code or data. Can be dyed in 16 colors.
* **Audio:** Plays vanilla Minecraft music discs via `disk.playAudio()`.

### 3.3 Printer
* **Operation:** Consumes Paper and Ink Sacs (or Redstone/Dye).
* **Output:** Generates printed Pages or stitches multiple pages into bound Books.

### 3.4 Modems & Rednet
* **Wireless Modem:**
  * Range: 64 blocks in clear weather; 384 blocks at high elevation ($Y > 96$) or during thunderstorms.
  * Inter-dimensionally isolated.
* **Wired Modem & Networking Cable:**
  * Cables run along block faces up to 256 blocks.
  * Modems connect computers and peripherals over a single bus, assigning unique network IDs (e.g. `monitor_2`, `drive_0`).

---

## 4. Software Architecture & Operating System (CraftOS)

### 4.1 Sandboxed Lua 5.1 Runtime
* Each computer runs a dedicated Lua state or cooperative coroutine.
* **Tick Limiter / Timeout:** If a program executes for more than 7.0 seconds without yielding via `os.pullEvent()` or `sleep()`, the runtime throws `"Too long without yielding"` and terminates the script to protect server tick rates.

### 4.2 Standard APIs

#### `os`
* `os.pullEvent(filter)`: Blocks execution until an event matching `filter` is popped from the event queue.
* `os.pullEventRaw()`: Like `pullEvent`, but cannot be interrupted by `Ctrl+T` termination.
* `os.startTimer(timeInSeconds)`: Starts an asynchronous countdown timer; fires `timer` event with timer ID.
* `os.setAlarm(timeOfDay)`: Fires `alarm` event at target in-game clock time.
* `os.time()`, `os.day()`, `os.clock()`: Timekeeping utilities.
* `os.reboot()`, `os.shutdown()`: Computer power controls.

#### `fs` (File System)
* Virtual hierarchical file system scoped to the computer's persistent disk directory.
* Methods: `fs.list(path)`, `fs.exists(path)`, `fs.isDir(path)`, `fs.isReadOnly(path)`, `fs.makeDir(path)`, `fs.move(from, to)`, `fs.copy(from, to)`, `fs.delete(path)`, `fs.getSize(path)`, `fs.open(path, mode)`.

#### `term`
* `term.write(text)`, `term.clear()`, `term.clearLine()`, `term.getCursorPos()`, `term.setCursorPos(x, y)`, `term.setCursorBlink(bool)`, `term.getSize()`, `term.scroll(n)`.
* Color methods (Advanced only): `term.setTextColor(color)`, `term.setBackgroundColor(color)`.

#### `turtle`
* Movement: `turtle.forward()`, `turtle.back()`, `turtle.up()`, `turtle.down()`, `turtle.turnLeft()`, `turtle.turnRight()`.
* Interaction: `turtle.dig()`, `turtle.digUp()`, `turtle.digDown()`, `turtle.place()`, `turtle.placeUp()`, `turtle.placeDown()`, `turtle.attack()`, `turtle.attackUp()`, `turtle.attackDown()`.
* Inspection: `turtle.detect()`, `turtle.detectUp()`, `turtle.detectDown()`, `turtle.compare()`, `turtle.compareUp()`, `turtle.compareDown()`.
* Inventory: `turtle.select(slot1_to_16)`, `turtle.getItemCount(slot)`, `turtle.getItemSpace(slot)`, `turtle.drop(count)`, `turtle.suck(count)`, `turtle.refuel(count)`, `turtle.getFuelLevel()`, `turtle.getFuelLimit()`.
* Crafting: `turtle.craft(quantity)`.

#### `redstone` / `rs`
* `redstone.getSides()`: Returns table of 6 faces.
* `redstone.getInput(side)`, `redstone.setOutput(side, bool)`: Binary signals.
* `redstone.getAnalogInput(side)`, `redstone.setAnalogOutput(side, 0_to_15)`: Analog signals.
* `redstone.getBundledInput(side)`, `redstone.setBundledOutput(side, mask)`: 16-bit bundled bitmask compatible with RedPower 2 Bundled Cables.

#### `rednet`
* `rednet.open(modemSide)`: Opens wireless communication.
* `rednet.send(targetID, message)`: Sends unicast message.
* `rednet.broadcast(message)`: Sends broadcast packet to all listening nodes in range.
* `rednet.receive(timeout)`: Listens for incoming rednet packets.

#### `peripheral`
* `peripheral.getNames()`: Returns all connected peripheral names.
* `peripheral.isPresent(sideOrName)`, `peripheral.getType(sideOrName)`: Type discovery.
* `peripheral.getMethods(sideOrName)`: Introspects exposed API methods.
* `peripheral.call(sideOrName, method, ...args)`: Dispatches remote method.
* `peripheral.wrap(sideOrName)`: Returns a high-level proxy table wrapping methods.

#### `http`
* `http.get(url, headers)`: Performs HTTP GET request.
* `http.post(url, postData, headers)`: Performs HTTP POST request.
* Built-in `pastebin` utility for downloading and sharing scripts directly.

---

## 5. Complete Registry

### Blocks
1. `computercraft:computer` (Standard Computer)
2. `computercraft:computer_advanced` (Gold Computer)
3. `computercraft:turtle` (Standard Turtle)
4. `computercraft:turtle_advanced` (Advanced Turtle)
5. `computercraft:monitor` (Standard Monitor)
6. `computercraft:monitor_advanced` (Advanced Color Monitor)
7. `computercraft:disk_drive`
8. `computercraft:printer`
9. `computercraft:wireless_modem`
10. `computercraft:wired_modem`
11. `computercraft:cable` (Networking Cable)

### Items
1. `computercraft:pocket_computer`
2. `computercraft:pocket_computer_advanced`
3. `computercraft:floppy_disk` (16 dyed colors)
4. `computercraft:printout` (Single page)
5. `computercraft:book` (Bound printouts)

---

## 6. Key Recipes
* **Computer:** 7x Stone + 1x Redstone + 1x Glass Pane.
* **Advanced Computer:** 7x Gold Ingot + 1x Redstone + 1x Glass Pane.
* **Turtle:** 7x Iron Ingot + 1x Chest + 1x Computer.
* **Advanced Turtle:** 7x Gold Ingot + 1x Chest + 1x Advanced Computer.
* **Disk Drive:** 7x Stone + 1x Redstone + 1x Redstone Torch.
* **Wireless Modem:** 7x Stone + 1x Ender Pearl.
* **Monitor:** 8x Stone + 1x Glass Pane.
* **Advanced Monitor:** 8x Gold Ingot + 1x Glass Pane.

---

## 7. Recreation Implementation Notes for Rekkit (Rust)
1. **Lua Engine:** Embed `mlua` with Lua 5.1 / Luau backend.
   * Restrict standard library to safe modules (`base`, `table`, `string`, `math`, `bit32`).
   * Hook CPU instruction counting into `mlua::set_interrupt` to abort runaway non-yielding infinite loops after 7 seconds.
2. **Turtle Movement State:** Turtle translation must atomically verify target block collision, perform block destruction/placement, update coordinates in chunk space, and animate the block movement smoothly to client viewers.
3. **Rednet Simulation:** Rednet packets should broadcast to a spatial spatial grid / R-tree: filter receivers by distance $R \le 64$ (or $384$ if $Y > 96$ or storming) and push a `rednet_message` event into recipient event queues.
