pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TMC: u16 = 0xFF07;
const CLOCKSPEED: i32 = 4194304;

pub struct Timer {
    pub timer_counter: i32,
    pub divider_counter: i32,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            timer_counter: 1024,
            divider_counter: 0,
        }
    }

    pub fn get_clock_freq_from_byte(tmc: u8) -> u8 {
        tmc & 0x3
    }

    pub fn is_clock_enabled(tmc: u8) -> bool {
        (tmc >> 2) & 1 == 1
    }

    pub fn set_clock_freq(&mut self, freq: u8) {
        match freq {
            0 => self.timer_counter = 1024,
            1 => self.timer_counter = 16,
            2 => self.timer_counter = 64,
            3 => self.timer_counter = 256,
            _ => {}
        }
    }

    pub fn freq_for_code(code: u8) -> i32 {
        match code {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => 1024,
        }
    }
}
