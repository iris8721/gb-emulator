# gb

Game Boy (DMG — the original 1989 handheld, not the GBA) emulator in Rust + SDL2. ~2.1k LOC.

## run

You must supply your own ROM file (`.gb`); none is included.

```
cargo run -- path/to/rom.gb
```

Note: the bundled SDL2 build uses an older `cmake_minimum_required`, so on the very first build (or after `cargo clean`) you may need:

```
CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo run -- path/to/rom.gb
```

## file by file breakdown

- `build.rs` — links advapi32 on Windows
- `src/main.rs` — SDL2 window, event loop, key mapping, 60fps rendering
- `src/emulator.rs` — main update loop (CPU/timers/GPU/interrupts sync), tile & sprite rendering
- `src/cpu.rs` — all 256 base opcodes + 256 CB-prefixed opcodes, register system, ALU operations
- `src/memory.rs` — memory map, read/write control, MBC1/MBC2 banking, DMA transfer
- `src/gpu.rs` — LCD status, colour palette lookup, screen data buffer
- `src/timer.rs` — TIMA/TMA/TMC timer + divider register
- `src/interrupts.rs` — interrupt IDs and service routine addresses
- `src/joypad.rs` — 8-button state, interrupt requests on press
- `src/cartridge.rs` — ROM loading, MBC type detection

## notable details

- all 256 + CB-prefixed opcodes implemented
- cycle-accurate timer and GPU sync
- MBC1 and MBC2 memory bank controllers

## known limitations

- no audio (APU not implemented)
- no MBC3/MBC5, so many later cartridges won't run
- cartridge RAM is not persisted to disk — no save files
