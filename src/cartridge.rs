use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MbcType {
    None,
    Mbc1,
    Mbc2,
}

pub struct Cartridge {
    pub data: Vec<u8>,
    pub mbc_type: MbcType,
    pub rom_bank_count: u16,
    pub ram_bank_count: u8,
}

impl Cartridge {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Cartridge, String> {
        let data = fs::read(path).map_err(|e| format!("Failed to load ROM: {}", e))?;
        if data.len() < 0x150 {
            return Err("ROM too small".to_string());
        }

        let mbc_type = match data[0x147] {
            0 => MbcType::None,
            1 | 2 | 3 => MbcType::Mbc1,
            5 | 6 => MbcType::Mbc2,
            other => {
                eprintln!("Warning: unknown cartridge type 0x{:02X}, defaulting to None", other);
                MbcType::None
            }
        };

        let rom_bank_count = 2u16 << data[0x148].min(8);

        let ram_bank_count = match data[0x149] {
            0 => 0,
            1 => 1,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => 0,
        };

        // Pad to 2MB so every bank an MBC can select is in range
        let mut padded = data;
        if padded.len() < 0x200000 {
            padded.resize(0x200000, 0);
        }

        Ok(Cartridge {
            data: padded,
            mbc_type,
            rom_bank_count,
            ram_bank_count,
        })
    }
}
