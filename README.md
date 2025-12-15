# Rust-Chip8

A fully-featured CHIP-8 emulator written in Rust, achieving accurate emulation with proper quirk handling and consistent 60 FPS performance.

![CHIP-8 Emulator Demo]()

## Overview

This project is a complete CHIP-8 interpreter implementation built from scratch in Rust. It was developed as an exploration of low-level programming concepts and systems emulation while learning Rust's ownership model and safety guarantees.

## Features

- **Complete CHIP-8 instruction set** - All 35 opcodes fully implemented
- **Accurate timing** - 60 Hz display refresh with configurable CPU cycles per frame
- **Comprehensive quirk support** - Handles all major CHIP-8 interpreter quirks:
  - Display wait quirk
  - Clipping quirk
  - Shift quirks
  - Jump quirks
  - Logic operation quirks
  - Memory access quirks
- **Full keyboard input** - 16-key hexadecimal keypad mapping
- **Modern rendering** - Hardware-accelerated pixel rendering via `pixels` crate

## Test Suite Results

This emulator passes the comprehensive [Timendus CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite), demonstrating accuracy and compatibility:

### CHIP-8 Splash Screen
*Tests basic display and sprite rendering*

![CHIP-8 Test]()

### Corax+ Opcode Test
*Validates all opcode implementations*

![Corax Test]()

### Flags Test
*Verifies correct flag register behavior*

![Flags Test]()

### Quirks Test
*Confirms proper handling of various CHIP-8 quirks*

![Quirks Test]()

## Games & Programs

The emulator runs a wide variety of CHIP-8 programs flawlessly:

### IBM Logo
![IBM Logo]()

### Tetris
![Tetris]()

### Space Invaders
![Space Invaders]()

## Technical Implementation

### Architecture

The emulator consists of three main components:

- **CPU Module** (`cpu.rs`) - Handles instruction fetch, decode, and execution cycle
- **Display Module** (`display.rs`) - Manages the 64×32 monochrome display buffer
- **Main Event Loop** (`main.rs`) - Coordinates timing, input handling, and rendering

### Key Technical Details

- **Memory Layout**: 4KB RAM with program space starting at 0x200
- **Registers**: 16 8-bit general-purpose registers (V0-VF) plus 16-bit I register
- **Stack**: 16 levels for subroutine calls
- **Timers**: 60 Hz delay and sound timers
- **Display**: 64×32 pixel monochrome framebuffer with XOR drawing
- **Input**: 16-key hexadecimal keypad with press/release detection

### Dependencies

- **[winit](https://github.com/rust-windowing/winit)** - Cross-platform window creation and event handling
- **[pixels](https://github.com/parasyte/pixels)** - Minimal hardware-accelerated pixel buffer
- **[rand](https://github.com/rust-random/rand)** - Random number generation for RND opcode

These dependencies were chosen for their minimal overhead, cross-platform support, and integration with modern Rust async patterns.

### Performance

The emulator maintains a consistent **60 FPS** with 12 CPU cycles executed per frame, closely matching original CHIP-8 timing characteristics. Frame timing is controlled via sleep-based throttling to prevent excessive CPU usage.

## Controls

CHIP-8 uses a 16-key hexadecimal keypad mapped to modern keyboard:

```
CHIP-8 Keypad:        Keyboard Mapping:
┌─┬─┬─┬─┐            ┌─┬─┬─┬─┐
│1│2│3│C│            │1│2│3│4│
├─┼─┼─┼─┤            ├─┼─┼─┼─┤
│4│5│6│D│            │Q│W│E│R│
├─┼─┼─┼─┤    ──→     ├─┼─┼─┼─┤
│7│8│9│E│            │A│S│D│F│
├─┼─┼─┼─┤            ├─┼─┼─┼─┤
│A│0│B│F│            │Z│X│C│V│
└─┴─┴─┴─┘            └─┴─┴─┴─┘
```

## Building & Running

```bash
# Clone the repository
git clone https://github.com/yourusername/chip8-emulator.git
cd chip8-emulator

# Build and run (release mode recommended for best performance)
cargo run --release

# The ROM path can be changed in main.rs:
# const PATH: &str = "./roms/your-rom.ch8";
```

## Project Structure

```
chip8-emulator/
├── src/
│   ├── main.rs      # Window management, event loop, rendering
│   ├── cpu.rs       # CPU core, instruction execution
│   ├── display.rs   # Display buffer management
│   └── utility.rs   # Helper utilities
├── roms/            # CHIP-8 ROM files
└── Cargo.toml       # Project dependencies
```

## Learning Outcomes

This project provided hands-on experience with:

- **Low-level programming concepts**: Memory management, CPU cycles, instruction decoding, and bitwise operations
- **Rust fundamentals**: Ownership, borrowing, pattern matching, and zero-cost abstractions
- **Systems emulation**: Timing accuracy, hardware quirks, and compatibility testing
- **Graphics programming**: Frame buffers, pixel manipulation, and rendering pipelines

## Resources

- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [CHIP-8 Wikipedia](https://en.wikipedia.org/wiki/CHIP-8)
- [Timendus Test Suite](https://github.com/Timendus/chip8-test-suite)
- [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- 
---

*Built with Rust 🦀*
