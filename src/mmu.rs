pub const BIOS_START: u32 = 0xBFC00000;
pub const BIOS_SIZE: u32 = 512 * 1014;
pub const BIOS_END: u32 = BIOS_START + BIOS_SIZE;

pub const RAM_START: u32 = 0x00000000;
pub const RAM_SIZE: u32 = 2 * 1024 * 1024;
pub const RAM_END: u32 = RAM_START + RAM_SIZE;

pub struct MMU {
    bios: Vec<u8>,
    ram: Box<[u8; RAM_SIZE as usize]>,
}

impl MMU {
    pub fn new(bios: Vec<u8>) -> Self {
        Self {
            bios,
            ram: vec![0; RAM_SIZE as usize].try_into().unwrap(),
        }
    }

    pub fn read(&self, address: u32) -> u32 {
        let mut word = 0;

        let offset = match address {
            RAM_START..RAM_END => address,
            BIOS_START..BIOS_END => address - BIOS_START,
            _ => panic!("Cannot read from address"),
        } as usize;

        let source = match address {
            RAM_START..RAM_END => &self.ram[offset..offset + 4],
            BIOS_START..BIOS_END => &self.bios[offset..offset + 4],
            _ => panic!("Cannot read from address"),
        };

        for i in 0..4 {
            let value = source[i];
            word |= (value as u32) << (i * 8)
        }

        word
    }

    pub fn write(&mut self, address: u32, value: u32) {
        for i in 0..4 {
            match address {
                RAM_START..RAM_END => {
                    self.ram[(address + i) as usize] = (value >> (i * 8)) as u8;
                }
                _ => panic!("Cannot write to address 0x{:2x}", address),
            }
        }
    }
}
