/// Joypad button mapping:
/// Bit 0: Right
/// Bit 1: Left
/// Bit 2: Up
/// Bit 3: Down
/// Bit 4: A
/// Bit 5: B
/// Bit 6: Select
/// Bit 7: Start

pub struct Joypad {
    /// Each bit: 0 = pressed, 1 = not pressed
    pub state: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { state: 0xFF }
    }

    /// Called when a key is pressed. Returns true if a joypad interrupt should be requested.
    pub fn key_pressed(&mut self, key: u8, ff00: u8) -> bool {
        let previously_unset = (self.state >> key) & 1 == 1;
        self.state &= !(1 << key);

        let button = key > 3;

        let mut request_interrupt = false;
        if button && (ff00 & 0x20) == 0 {
            request_interrupt = true;
        } else if !button && (ff00 & 0x10) == 0 {
            request_interrupt = true;
        }

        request_interrupt && previously_unset
    }

    pub fn key_released(&mut self, key: u8) {
        self.state |= 1 << key;
    }

    pub fn get_joypad_state(&self, ff00: u8) -> u8 {
        let mut res = ff00 ^ 0xFF;

        if (res & 0x10) == 0 {
            // Standard buttons (A, B, Select, Start) in upper nibble of state
            let top = (self.state >> 4) | 0xF0;
            res &= top;
        } else if (res & 0x20) == 0 {
            // Directional buttons in lower nibble of state
            let bottom = (self.state & 0xF) | 0xF0;
            res &= bottom;
        }
        res
    }
}
