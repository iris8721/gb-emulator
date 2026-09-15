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

        let ram_bank_count = match data[0x149] {
            0 => 0,
            1 => 1,
            2 => 1,
            3 => 4,
            4 => 16,
            _ => 0,
        };

        // Pad to max cartridge size
        let mut padded = vec![0u8; 0x200000];
        let copy_len = data.len().min(0x200000);
        padded[..copy_len].copy_from_slice(&data[..copy_len]);

        Ok(Cartridge {
            data: padded,
            mbc_type,
            ram_bank_count,
        })
    }
}
