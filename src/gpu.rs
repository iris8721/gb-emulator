#[derive(Clone, Copy, PartialEq)]
pub enum Colour {
    White,
    LightGray,
    DarkGray,
    Black,
}

pub struct Gpu {
    pub scanline_counter: i32,
    pub screen_data: [[[u8; 3]; 144]; 160],
    pub bg_line: [u8; 160],
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            scanline_counter: 456,
            screen_data: [[[255; 3]; 144]; 160],
            bg_line: [0; 160],
        }
    }

    pub fn is_lcd_enabled(memory: &[u8]) -> bool {
        memory[0xFF40] & 0x80 != 0
    }

    pub fn get_colour(colour_num: u8, palette_addr: u16, memory: &[u8]) -> Colour {
        let palette = memory[palette_addr as usize];
        let (hi, lo) = match colour_num {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 4),
            3 => (7, 6),
            _ => (1, 0),
        };

        let colour = ((palette >> hi) & 1) << 1 | ((palette >> lo) & 1);
        match colour {
            0 => Colour::White,
            1 => Colour::LightGray,
            2 => Colour::DarkGray,
            3 => Colour::Black,
            _ => Colour::White,
        }
    }

    pub fn colour_to_rgb(col: Colour) -> (u8, u8, u8) {
        match col {
            Colour::White => (255, 255, 255),
            Colour::LightGray => (0xCC, 0xCC, 0xCC),
            Colour::DarkGray => (0x77, 0x77, 0x77),
            Colour::Black => (0, 0, 0),
        }
    }
}
