use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gpu::Gpu;
use crate::interrupts;
use crate::memory::Memory;
use crate::timer;

const MAX_CYCLES: u32 = 70224;

pub struct Emulator {
    pub cpu: Cpu,
    pub memory: Memory,
    pub gpu: Gpu,
}

impl Emulator {
    pub fn new(cartridge: Cartridge) -> Self {
        Emulator {
            cpu: Cpu::new(),
            memory: Memory::new(cartridge),
            gpu: Gpu::new(),
        }
    }

    pub fn update(&mut self) {
        let mut cycles_this_update: u32 = 0;

        while cycles_this_update < MAX_CYCLES {
            let cycles = if self.memory.halted {
                4 // When halted, still consume cycles until an interrupt wakes us
            } else {
                self.cpu.execute_next_opcode(&mut self.memory)
            };

            cycles_this_update += cycles;
            self.update_timers(cycles);
            self.update_graphics(cycles);

            let cycles = self.do_interrupts();
            if cycles > 0 {
                cycles_this_update += cycles;
                self.update_timers(cycles);
                self.update_graphics(cycles);
            }

            // IME goes high after EI itself, so the next instruction always runs before dispatch
            if self.memory.ei_pending {
                self.memory.ei_pending = false;
                self.memory.interrupt_master = true;
            }
        }
    }

    fn update_timers(&mut self, cycles: u32) {
        // Divider register
        self.memory.timer.divider_counter += cycles as i32;
        while self.memory.timer.divider_counter >= 256 {
            self.memory.timer.divider_counter -= 256;
            self.memory.rom[0xFF04] = self.memory.rom[0xFF04].wrapping_add(1);
        }

        let tmc = self.memory.rom[timer::TMC as usize];
        if timer::Timer::is_clock_enabled(tmc) {
            self.memory.timer.timer_counter -= cycles as i32;

            while self.memory.timer.timer_counter <= 0 {
                let freq = timer::Timer::get_clock_freq_from_byte(tmc);
                self.memory.timer.timer_counter += timer::Timer::freq_for_code(freq);

                let tima = self.memory.rom[timer::TIMA as usize];
                if tima == 255 {
                    let tma = self.memory.rom[timer::TMA as usize];
                    self.memory.rom[timer::TIMA as usize] = tma;
                    self.memory.request_interrupt(interrupts::TIMER);
                } else {
                    self.memory.rom[timer::TIMA as usize] = tima.wrapping_add(1);
                }
            }
        }
    }

    fn update_graphics(&mut self, cycles: u32) {
        self.set_lcd_status();

        if !Gpu::is_lcd_enabled(&self.memory.rom) {
            return;
        }

        self.gpu.scanline_counter -= cycles as i32;

        if self.gpu.scanline_counter <= 0 {
            self.memory.rom[0xFF44] = self.memory.rom[0xFF44].wrapping_add(1);
            let current_line = self.memory.rom[0xFF44];

            self.gpu.scanline_counter = 456;

            if current_line == 144 {
                self.memory.request_interrupt(interrupts::VBLANK);
            } else if current_line > 153 {
                self.memory.rom[0xFF44] = 0;
            } else if current_line < 144 {
                self.draw_scanline();
            }
        }
    }

    fn set_lcd_status(&mut self) {
        let mut status = self.memory.rom[0xFF41];

        if !Gpu::is_lcd_enabled(&self.memory.rom) {
            self.gpu.scanline_counter = 456;
            self.memory.rom[0xFF44] = 0;
            status &= 0xF8; // mode 0, coincidence clear
            self.memory.rom[0xFF41] = status;
            return;
        }

        let current_line = self.memory.rom[0xFF44];
        let current_mode = status & 0x3;
        let mode: u8;
        let mut req_int = false;

        if current_line >= 144 {
            mode = 1;
            status = (status & 0xFC) | 0x01;
            req_int = status & 0x10 != 0;
        } else {
            let mode2_bounds = 456 - 80;
            let mode3_bounds = mode2_bounds - 172;

            if self.gpu.scanline_counter >= mode2_bounds {
                mode = 2;
                status = (status & 0xFC) | 0x02;
                req_int = status & 0x20 != 0;
            } else if self.gpu.scanline_counter >= mode3_bounds {
                mode = 3;
                status = (status & 0xFC) | 0x03;
            } else {
                mode = 0;
                status &= 0xFC;
                req_int = status & 0x08 != 0;
            }
        }

        if req_int && (mode != current_mode) {
            self.memory.request_interrupt(interrupts::LCD);
        }

        // Coincidence flag - interrupt only when it goes 0 -> 1
        let lyc = self.memory.rom[0xFF45];
        let was_coincident = status & 0x04 != 0;
        if current_line == lyc {
            status |= 0x04;
            if !was_coincident && status & 0x40 != 0 {
                self.memory.request_interrupt(interrupts::LCD);
            }
        } else {
            status &= !0x04;
        }

        self.memory.rom[0xFF41] = status;
    }

    fn draw_scanline(&mut self) {
        let control = self.memory.rom[0xFF40];
        if control & 0x01 != 0 {
            self.render_tiles();
        } else {
            let y = self.memory.rom[0xFF44] as usize;
            for px in 0..160 {
                self.gpu.screen_data[px][y] = [255; 3];
            }
            self.gpu.bg_line = [0; 160];
        }
        if control & 0x02 != 0 {
            self.render_sprites();
        }
    }

    fn render_tiles(&mut self) {
        let lcd_control = self.memory.rom[0xFF40];
        let scroll_y = self.memory.rom[0xFF42];
        let scroll_x = self.memory.rom[0xFF43];
        let window_y = self.memory.rom[0xFF4A];
        let window_x = self.memory.rom[0xFF4B].wrapping_sub(7);
        let current_line = self.memory.rom[0xFF44];

        let using_window = (lcd_control & 0x20 != 0) && window_y <= current_line;

        let tile_data: u16;
        let unsigned;
        if lcd_control & 0x10 != 0 {
            tile_data = 0x8000;
            unsigned = true;
        } else {
            tile_data = 0x8800;
            unsigned = false;
        }

        let background_map: u16 = if lcd_control & 0x08 != 0 { 0x9C00 } else { 0x9800 };
        let window_map: u16 = if lcd_control & 0x40 != 0 { 0x9C00 } else { 0x9800 };

        for pixel in 0u8..160 {
            let (x_pos, y_pos, map) = if using_window && pixel >= window_x {
                (pixel - window_x, current_line - window_y, window_map)
            } else {
                (pixel.wrapping_add(scroll_x), scroll_y.wrapping_add(current_line), background_map)
            };

            let tile_row: u16 = (y_pos as u16 / 8) * 32;
            let tile_col: u16 = (x_pos as u16) / 8;
            let tile_addr = map + tile_row + tile_col;
            let tile_num_raw = self.memory.read_byte(tile_addr);

            let tile_location: u16 = if unsigned {
                tile_data + (tile_num_raw as u16) * 16
            } else {
                let signed_num = tile_num_raw as i8;
                (tile_data as i32 + ((signed_num as i32 + 128) * 16)) as u16
            };

            let line = (y_pos % 8) as u16 * 2;
            let data1 = self.memory.read_byte(tile_location + line);
            let data2 = self.memory.read_byte(tile_location + line + 1);

            let colour_bit = 7 - (x_pos % 8) as i32;

            let colour_num = (((data2 >> colour_bit) & 1) << 1) | ((data1 >> colour_bit) & 1);

            let col = Gpu::get_colour(colour_num, 0xFF47, &self.memory.rom);
            let (r, g, b) = Gpu::colour_to_rgb(col);

            let final_y = current_line as usize;
            let px = pixel as usize;
            if final_y < 144 && px < 160 {
                self.gpu.screen_data[px][final_y] = [r, g, b];
                self.gpu.bg_line[px] = colour_num;
            }
        }
    }

    fn render_sprites(&mut self) {
        let lcd_control = self.memory.rom[0xFF40];
        let use_8x16 = lcd_control & 0x04 != 0;
        let y_size: i32 = if use_8x16 { 16 } else { 8 };
        let current_line = self.memory.rom[0xFF44] as i32;

        // First 10 sprites on the line in OAM order get drawn
        let mut visible = [(0i32, 0u16); 10];
        let mut count = 0;
        for sprite in 0..40u16 {
            let index = sprite * 4;
            let y_pos = self.memory.read_byte(0xFE00 + index) as i32 - 16;
            if current_line >= y_pos && current_line < (y_pos + y_size) {
                let x_pos = self.memory.read_byte(0xFE00 + index + 1) as i32 - 8;
                visible[count] = (x_pos, index);
                count += 1;
                if count == 10 {
                    break;
                }
            }
        }

        // Lowest X wins, then lowest OAM index - draw the losers first
        visible[..count].sort_unstable_by(|a, b| b.cmp(a));

        for &(x_pos, index) in &visible[..count] {
            let y_pos = self.memory.read_byte(0xFE00 + index) as i32 - 16;
            let tile_location = self.memory.read_byte(0xFE00 + index + 2);
            let attributes = self.memory.read_byte(0xFE00 + index + 3);

            let y_flip = attributes & 0x40 != 0;
            let x_flip = attributes & 0x20 != 0;
            let priority = attributes & 0x80 != 0;

            let mut line = current_line - y_pos;
            if y_flip {
                line = y_size - 1 - line;
            }

            let tile = if use_8x16 { tile_location & 0xFE } else { tile_location };
            let line = line as u16 * 2;
            let data_addr = 0x8000u16 + (tile as u16) * 16 + line;
            let data1 = self.memory.read_byte(data_addr);
            let data2 = self.memory.read_byte(data_addr + 1);

            for tile_pixel in (0..8i32).rev() {
                let colour_bit = if x_flip {
                    7 - tile_pixel
                } else {
                    tile_pixel
                };

                let colour_num = (((data2 >> colour_bit) & 1) << 1) | ((data1 >> colour_bit) & 1);

                // Colour 0 is transparent
                if colour_num == 0 {
                    continue;
                }

                let pixel_x = x_pos + 7 - tile_pixel;
                if pixel_x < 0 || pixel_x >= 160 {
                    continue;
                }

                // If priority set, only draw over background colour 0
                if priority && self.gpu.bg_line[pixel_x as usize] != 0 {
                    continue;
                }

                let palette_addr: u16 = if attributes & 0x10 != 0 { 0xFF49 } else { 0xFF48 };
                let col = Gpu::get_colour(colour_num, palette_addr, &self.memory.rom);
                let (r, g, b) = Gpu::colour_to_rgb(col);

                self.gpu.screen_data[pixel_x as usize][current_line as usize] = [r, g, b];
            }
        }
    }

    fn do_interrupts(&mut self) -> u32 {
        let req = self.memory.rom[0xFF0F];
        let enabled = self.memory.rom[0xFFFF];
        let pending = req & enabled & 0x1F;

        // Any pending enabled interrupt wakes from halt, even with IME off
        if pending != 0 {
            self.memory.halted = false;
        }

        if !self.memory.interrupt_master || pending == 0 {
            return 0;
        }

        self.service_interrupt(pending.trailing_zeros() as u8);
        20
    }

    fn service_interrupt(&mut self, interrupt: u8) {
        self.memory.interrupt_master = false;
        self.memory.rom[0xFF0F] &= !(1 << interrupt);

        let pc = self.cpu.pc;
        self.cpu.push_word(&mut self.memory, pc);
        self.cpu.pc = interrupts::interrupt_address(interrupt);
    }

    pub fn key_pressed(&mut self, key: u8) {
        let ff00 = self.memory.rom[0xFF00];
        if self.memory.joypad.key_pressed(key, ff00) {
            self.memory.request_interrupt(interrupts::JOYPAD);
        }
    }

    pub fn key_released(&mut self, key: u8) {
        self.memory.joypad.key_released(key);
    }
}
