/// Interrupt bit constants
pub const VBLANK: u8 = 0;
pub const LCD: u8 = 1;
pub const TIMER: u8 = 2;
pub const JOYPAD: u8 = 4;

/// Interrupt service routine addresses
pub fn interrupt_address(id: u8) -> u16 {
    match id {
        0 => 0x40,
        1 => 0x48,
        2 => 0x50,
        3 => 0x58,
        4 => 0x60,
        _ => 0x00,
    }
}
