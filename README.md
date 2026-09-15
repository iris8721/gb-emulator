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

With GCC 15 the bundled SDL2 also fails to compile under the C23 default, so add `CFLAGS=-std=gnu17` as well.

## file by file breakdown

- `build.rs` — links advapi32 on Windows
- `src/main.rs` — SDL2 window, event loop, key mapping, 60fps rendering
- `src/emulator.rs` — main update loop (CPU/timers/GPU/interrupts sync), tile & sprite rendering
- `src/cpu.rs` — all 256 base opcodes + 256 CB-prefixed opcodes, register system, ALU operations
- `src/memory.rs` — memory map, read/write control, MBC1/MBC2 banking, DMA transfer
- `src/gpu.rs` — colour palette lookup, screen data buffer
- `src/timer.rs` — TIMA/TMA/TMC timer + divider register
- `src/interrupts.rs` — interrupt IDs and service routine addresses
- `src/joypad.rs` — 8-button state, interrupt requests on press
- `src/cartridge.rs` — ROM loading, MBC type detection

## notable details

- all 256 + CB-prefixed opcodes implemented
- timer, divider and PPU scanline counters driven by CPU cycle counts, including the 20-cycle interrupt dispatch
- MBC1 and MBC2 memory bank controllers
- HALT bug (halt with IME=0 and an interrupt pending repeats the next byte)

## known limitations

- no audio (APU not implemented)
- no MBC3/MBC5, so many later cartridges won't run
- cartridge RAM is not persisted to disk — no save files
- no serial link; FF01/FF02 are plain bytes and the serial interrupt never fires
- PPU mode timing is a fixed 80/172/204 split per scanline, and the window uses LY-WY rather than an internal line counter
- TIMA overflow reloads TMA and fires the interrupt at once instead of 4 cycles later
