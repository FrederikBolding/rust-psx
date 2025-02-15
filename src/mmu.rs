/**
*   KUSEG     KSEG0     KSEG1
 00000000h 80000000h A0000000h  2048K  Main RAM (first 64K reserved for BIOS)
 1F000000h 9F000000h BF000000h  8192K  Expansion Region 1 (ROM/RAM)
 1F800000h 9F800000h    --      1K     Scratchpad (D-Cache used as Fast RAM)
 1F801000h 9F801000h BF801000h  4K     I/O Ports
 1F802000h 9F802000h BF802000h  8K     Expansion Region 2 (I/O Ports)
 1FA00000h 9FA00000h BFA00000h  2048K  Expansion Region 3 (SRAM BIOS region for DTL cards)
 1FC00000h 9FC00000h BFC00000h  512K   BIOS ROM (Kernel) (4096K max)
       FFFE0000h (in KS-EG2)     0.5K   Internal CPU control registers (Cache Control)
*/

pub const RAM_START: u32 = 0x00000000;
pub const RAM_SIZE: u32 = 2 * 1024 * 1024;
pub const RAM_END: u32 = RAM_START + RAM_SIZE;

pub const EXPANSION_1_START: u32 = 0x1F000000;
pub const EXPANSION_1_SIZE: u32 = 8 * 1024 * 1024;
pub const EXPANSION_1_END: u32 = EXPANSION_1_START + EXPANSION_1_SIZE;

pub const IO_START: u32 = 0x1F801000;
pub const IO_SIZE: u32 = 4 * 1024;
pub const IO_END: u32 = IO_START + IO_SIZE;

pub const EXPANSION_2_START: u32 = 0x1F802000;
pub const EXPANSION_2_SIZE: u32 = 66;
pub const EXPANSION_2_END: u32 = EXPANSION_2_START + EXPANSION_2_SIZE;

pub const BIOS_START: u32 = 0x1FC00000;
pub const BIOS_SIZE: u32 = 512 * 1014;
pub const BIOS_END: u32 = BIOS_START + BIOS_SIZE;

// Since some of the memory regions are mirrors of each other, these masks let us map them to the same memory region where applicable.
const MEMORY_REGION_MASK: [u32; 8] = [
    0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, // KUSEG
    0x7FFFFFFF, // KSEG0
    0x1FFFFFFF, // KSEG1
    0xFFFFFFFF, 0xFFFFFFFF, // KSEG2
];

pub struct MMU {
    bios: Vec<u8>,
    ram: Box<[u8; RAM_SIZE as usize]>,

    // Store the 9 values used for memory control 1
    memory_control: [u32; 9],
    // Store configurable RAM_SIZE (aka memory control 2)
    ram_size: u32,
    // Cache control (memory control 3)
    cache_control: u32,
}

impl MMU {
    pub fn new(bios: Vec<u8>) -> Self {
        Self {
            bios,
            ram: vec![0; RAM_SIZE as usize].try_into().unwrap(),
            memory_control: [0; 9],
            ram_size: 0,
            cache_control: 0,
        }
    }

    pub fn is_instruction_cache_enabled(&self) -> bool {
        self.cache_control & 0x800 != 0
    }

    pub fn read(&self, address: u32, size: u32) -> u32 {
        let mut word = 0;

        let address = address & MEMORY_REGION_MASK[(address >> 29) as usize];

        let offset = match address {
            RAM_START..RAM_END => address,
            BIOS_START..BIOS_END => address - BIOS_START,
            _ => panic!("Cannot read from address 0x{:2x}", address),
        } as usize;

        let source = match address {
            RAM_START..RAM_END => &self.ram[offset..offset + 4],
            BIOS_START..BIOS_END => &self.bios[offset..offset + 4],
            _ => panic!("Cannot read from address"),
        };

        for i in 0..size {
            let value = source[i as usize];
            word |= (value as u32) << (i * 8)
        }

        word
    }

    pub fn write(&mut self, address: u32, size: u32, value: u32) {
        let address = address & MEMORY_REGION_MASK[(address >> 29) as usize];

        match address {
            RAM_START..RAM_END => {
                for i in 0..size {
                    self.ram[(address + i) as usize] = (value >> (i * 8)) as u8;
                }
            }
            EXPANSION_1_START..EXPANSION_1_END => {
                panic!("Cannot write to expansion");
            }
            // IO
            0x1F80100..=0x1F801020 => {
                let index = (address - IO_START) >> 2;
                self.memory_control[index as usize] = value;
            }
            0x1F801060 => {
                self.ram_size = value;
            }
            0x1F801D80..0x1F801DBC => {
                // TODO: Sound Processing Unit registers
            }
            EXPANSION_2_START..EXPANSION_2_END => {
                // TODO: DUART
            }
            0xFFFE0130 => {
                self.cache_control = value;
            }
            _ => panic!("Cannot write to address 0x{:2x}", address),
        }
    }
}
