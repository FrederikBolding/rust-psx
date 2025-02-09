pub const BIOS_START: u32 = 0xBFC00000;
pub const BIOS_SIZE: u32 = 512 * 1014;
pub const BIOS_END: u32 = BIOS_START + BIOS_SIZE;

pub const RAM_START: u32 = 0x00000000;
pub const RAM_SIZE: u32 = 2 * 1024 * 1024;
pub const RAM_END: u32 = RAM_START + RAM_SIZE;

pub struct MMU {
    bios: Vec<u8>,
    ram: Box<[u32; RAM_SIZE as usize]>,
}

impl MMU {
    pub fn new(bios: Vec<u8>) -> Self {
        Self {
            bios,
            ram: vec![0; RAM_SIZE as usize].try_into().unwrap(),
        }
    }

    fn read_bios_word(&self, offset: u32) -> u32 {
        let mut word = 0;

        for i in 0..4 {
            let value = self.bios[(offset + i) as usize];
            word |= (value as u32) << (i * 8)
        }

        word
    }

    pub fn read(&self, address: u32) -> u32 {
        match address {
            RAM_START..RAM_END => self.ram[address as usize],
            BIOS_START..BIOS_END => self.read_bios_word(address - BIOS_START),
            _ => panic!("Cannot read from address"),
        }
    }

    pub fn write(&mut self, address: u32, value: u32) {
        match address {
            RAM_START..RAM_END => {
                self.ram[address as usize] = value;
            }
            _ => panic!("Cannot write to address 0x{:2x}", address),
        }
    }
}
