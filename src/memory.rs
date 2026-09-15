use crate::cartridge::{Cartridge, MbcType};
use crate::joypad::Joypad;
use crate::timer::Timer;

pub struct Memory {
    pub rom: [u8; 0x10000],
    pub cartridge: Cartridge,
    pub ram_banks: [u8; 0x8000],
    pub current_rom_bank: u8,
    pub current_ram_bank: u8,
    pub enable_ram: bool,
    pub rom_banking: bool,
    pub joypad: Joypad,
    pub timer: Timer,
    pub interrupt_master: bool,
    pub halted: bool,
    pub pending_enable_interrupts: i8,
    pub pending_disable_interrupts: i8,
}

impl Memory {
    pub fn new(cartridge: Cartridge) -> Self {
        let mut mem = Memory {
            rom: [0; 0x10000],
            cartridge,
            ram_banks: [0; 0x8000],
            current_rom_bank: 1,
            current_ram_bank: 0,
            enable_ram: false,
            rom_banking: true,
            joypad: Joypad::new(),
            timer: Timer::new(),
            interrupt_master: false,
            halted: false,
            pending_enable_interrupts: 0,
            pending_disable_interrupts: 0,
        };

        // Load first 0x8000 bytes of cartridge into ROM
        for i in 0..0x8000 {
            mem.rom[i] = mem.cartridge.data[i];
        }

        // Initialize I/O registers
        mem.rom[0xFF05] = 0x00;
        mem.rom[0xFF06] = 0x00;
        mem.rom[0xFF07] = 0x00;
        mem.rom[0xFF10] = 0x80;
        mem.rom[0xFF11] = 0xBF;
        mem.rom[0xFF12] = 0xF3;
        mem.rom[0xFF14] = 0xBF;
        mem.rom[0xFF16] = 0x3F;
        mem.rom[0xFF17] = 0x00;
        mem.rom[0xFF19] = 0xBF;
        mem.rom[0xFF1A] = 0x7F;
        mem.rom[0xFF1B] = 0xFF;
        mem.rom[0xFF1C] = 0x9F;
        mem.rom[0xFF1E] = 0xBF;
        mem.rom[0xFF20] = 0xFF;
        mem.rom[0xFF21] = 0x00;
        mem.rom[0xFF22] = 0x00;
        mem.rom[0xFF23] = 0xBF;
        mem.rom[0xFF24] = 0x77;
        mem.rom[0xFF25] = 0xF3;
        mem.rom[0xFF26] = 0xF1;
        mem.rom[0xFF40] = 0x91;
        mem.rom[0xFF42] = 0x00;
        mem.rom[0xFF43] = 0x00;
        mem.rom[0xFF45] = 0x00;
        mem.rom[0xFF47] = 0xFC;
        mem.rom[0xFF48] = 0xFF;
        mem.rom[0xFF49] = 0xFF;
        mem.rom[0xFF4A] = 0x00;
        mem.rom[0xFF4B] = 0x00;
        mem.rom[0xFFFF] = 0x00;

        mem
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let addr = address as usize;

        // ROM bank area
        if (0x4000..=0x7FFF).contains(&addr) {
            let new_addr = addr - 0x4000;
            return self.cartridge.data[new_addr + (self.current_rom_bank as usize) * 0x4000];
        }

        // RAM bank area
        if (0xA000..=0xBFFF).contains(&addr) {
            let new_addr = addr - 0xA000;
            return self.ram_banks[new_addr + (self.current_ram_bank as usize) * 0x2000];
        }

        // Joypad register
        if addr == 0xFF00 {
            return self.joypad.get_joypad_state(self.rom[0xFF00]);
        }

        self.rom[addr]
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        let addr = address as usize;

        // ROM area - handle banking
        if addr < 0x8000 {
            self.handle_banking(address, data);
            return;
        }

        // RAM bank area
        if (0xA000..0xC000).contains(&addr) {
            if self.enable_ram {
                let new_addr = addr - 0xA000;
                self.ram_banks[new_addr + (self.current_ram_bank as usize) * 0x2000] = data;
            }
            return;
        }

        // Echo RAM
        if (0xE000..0xFE00).contains(&addr) {
            self.rom[addr] = data;
            self.rom[addr - 0x2000] = data;
            return;
        }

        // Restricted area
        if (0xFEA0..0xFEFF).contains(&addr) {
            return;
        }

        // Divider register - always resets to 0
        if addr == 0xFF04 {
            self.rom[0xFF04] = 0;
            return;
        }

        // Timer controller - detect frequency change
        if addr == 0xFF07 {
            let current_freq = self.rom[0xFF07] & 0x3;
            self.rom[0xFF07] = data;
            let new_freq = data & 0x3;
            if current_freq != new_freq {
                self.timer.set_clock_freq(new_freq);
            }
            return;
        }

        // Current scanline - reset to 0
        if addr == 0xFF44 {
            self.rom[0xFF44] = 0;
            return;
        }

        // DMA transfer
        if addr == 0xFF46 {
            self.do_dma_transfer(data);
            return;
        }

        // Normal write
        self.rom[addr] = data;
    }

    fn handle_banking(&mut self, address: u16, data: u8) {
        let addr = address as usize;
        let mbc = self.cartridge.mbc_type;

        // RAM enable
        if addr < 0x2000 {
            if mbc == MbcType::Mbc1 || mbc == MbcType::Mbc2 {
                self.do_ram_bank_enable(address, data);
            }
        }
        // ROM bank lower bits
        else if (0x2000..0x4000).contains(&addr) {
            if mbc == MbcType::Mbc1 || mbc == MbcType::Mbc2 {
                self.do_change_lo_rom_bank(data);
            }
        }
        // ROM bank upper bits or RAM bank
        else if (0x4000..0x6000).contains(&addr) {
            if mbc == MbcType::Mbc1 {
                if self.rom_banking {
                    self.do_change_hi_rom_bank(data);
                } else {
                    self.do_ram_bank_change(data);
                }
            }
        }
        // ROM/RAM mode select
        else if (0x6000..0x8000).contains(&addr) {
            if mbc == MbcType::Mbc1 {
                self.do_change_rom_ram_mode(data);
            }
        }
    }

    fn do_ram_bank_enable(&mut self, address: u16, data: u8) {
        if self.cartridge.mbc_type == MbcType::Mbc2 {
            if (address >> 4) & 1 == 1 {
                return;
            }
        }
        let test = data & 0xF;
        if test == 0xA {
            self.enable_ram = true;
        } else if test == 0x0 {
            self.enable_ram = false;
        }
    }

    fn do_change_lo_rom_bank(&mut self, data: u8) {
        if self.cartridge.mbc_type == MbcType::Mbc2 {
            self.current_rom_bank = data & 0xF;
            if self.current_rom_bank == 0 {
                self.current_rom_bank = 1;
            }
            return;
        }

        let lower5 = data & 0x1F;
        self.current_rom_bank &= 0xE0; // clear lower 5
        self.current_rom_bank |= lower5;
        if self.current_rom_bank == 0 {
            self.current_rom_bank = 1;
        }
    }

    fn do_change_hi_rom_bank(&mut self, data: u8) {
        self.current_rom_bank &= 0x1F; // clear upper 3
        let upper = data & 0xE0;
        self.current_rom_bank |= upper;
        if self.current_rom_bank == 0 {
            self.current_rom_bank = 1;
        }
    }

    fn do_ram_bank_change(&mut self, data: u8) {
        self.current_ram_bank = data & 0x3;
    }

    fn do_change_rom_ram_mode(&mut self, data: u8) {
        self.rom_banking = (data & 0x1) == 0;
        if self.rom_banking {
            self.current_ram_bank = 0;
        }
    }

    fn do_dma_transfer(&mut self, data: u8) {
        let address = (data as u16) << 8;
        for i in 0..0xA0u16 {
            let val = self.read_byte(address + i);
            self.rom[(0xFE00 + i) as usize] = val;
        }
    }

    pub fn request_interrupt(&mut self, id: u8) {
        let mut req = self.rom[0xFF0F];
        req |= 1 << id;
        self.rom[0xFF0F] = req;
    }
}
