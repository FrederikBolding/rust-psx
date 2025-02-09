use crate::mmu::{BIOS_START, MMU};

pub struct CPU {
    registers: [u32; 32],
    mmu: MMU,
    pc: u32,
}

impl CPU {
    pub fn new(mmu: MMU) -> Self {
        Self {
            registers: [0; 32],
            mmu,
            pc: BIOS_START,
        }
    }

    fn load_instruction(&self) -> Instruction {
        let word = self.mmu.read(self.pc);

        println!("0x{:2x}", word);

        Instruction(word)
    }

    pub fn step(&mut self) {
        let instruction = self.load_instruction();

        let opcode = instruction.opcode();

        let secondary_opcode = instruction.secondary_opcode();

        println!("Opcode {:b}, Secondary {:b}", opcode, secondary_opcode);

        match opcode {
            0b000000 => match secondary_opcode {
                0b000000 => {
                    panic!("SLL")
                }
                0b000010 => {
                    panic!("SRL")
                }
                0b000011 => {
                    panic!("SRA")
                }
                0b000100 => {
                    panic!("SSLV")
                }
                0b000110 => {
                    panic!("SRLV")
                }
                0b000111 => {
                    panic!("SRAV")
                }
                0b001000 => {
                    panic!("JR")
                }
                0b001001 => {
                    panic!("JALR")
                }
                0b001100 => {
                    panic!("SYSCALL")
                }
                0b001101 => {
                    panic!("BREAK")
                }
                0b010000 => {
                    panic!("MFHI")
                }
                0b010001 => {
                    panic!("MTHI")
                }
                0b010010 => {
                    panic!("MFLO")
                }
                0b010011 => {
                    panic!("MTLO")
                }
                0b011000 => {
                    panic!("MULT")
                }
                0b011001 => {
                    panic!("MULTU")
                }
                0b011010 => {
                    panic!("DIV")
                }
                0b011011 => {
                    panic!("DIVU")
                }
                0b100000 => {
                    panic!("ADD")
                }
                0b100001 => {
                    panic!("ADDU")
                }
                0b100010 => {
                    panic!("SUB")
                }
                0b100011 => {
                    panic!("SUBU")
                }
                0b100100 => {
                    panic!("AND")
                }
                0b100101 => {
                    panic!("OR")
                }
                0b100110 => {
                    panic!("XOR")
                }
                0b100111 => {
                    panic!("NOR")
                }
                0b101010 => {
                    panic!("SLT")
                }
                0b101011 => {
                    panic!("SLTU")
                }
                _ => {
                    panic!("Invalid instruction")
                }
            },
            0b000001 => {
                panic!("BXX")
            }
            0b000010 => {
                panic!("J")
            }
            0b000011 => {
                panic!("JAL")
            }
            0b000100 => {
                panic!("BEQ")
            }
            0b000101 => {
                panic!("BNE")
            }
            0b000110 => {
                panic!("BLEZ")
            }
            0b000111 => {
                panic!("BGTZ")
            }
            0b001000 => {
                panic!("ADDI")
            }
            0b001001 => {
                panic!("ADDIU")
            }
            0b001010 => {
                panic!("SLTI")
            }
            0b001011 => {
                panic!("SLTIU")
            }
            0b001100 => {
                panic!("ANDI")
            }
            0b001101 => {
                let immediate = instruction.immediate();
                let source = instruction.source() as usize;
                let target = instruction.target() as usize;

                let value = self.registers[source] | immediate;

                self.registers[target] = value;
            }
            0b001110 => {
                panic!("XORI")
            }
            0b001111 => {
                let value = instruction.immediate();
                let target = instruction.target() as usize;

                self.registers[target] = value << 16;
            }
            0b010000 => {
                panic!("COP0")
            }
            0b010001 => {
                panic!("COP1")
            }
            0b010010 => {
                panic!("COP2")
            }
            0b010011 => {
                panic!("COP3")
            }
            0b100000 => {
                panic!("LB")
            }
            0b100001 => {
                panic!("LH")
            }
            0b100010 => {
                panic!("LWL")
            }
            0b100011 => {
                panic!("LW")
            }
            0b100100 => {
                panic!("LBU")
            }
            0b100101 => {
                panic!("LHU")
            }
            0b100110 => {
                panic!("LWR")
            }
            0b101000 => {
                panic!("SB")
            }
            0b101001 => {
                panic!("SH")
            }
            0b101010 => {
                panic!("SWL")
            }
            0b101011 => {
                let immediate = instruction.immediate();
                let source = instruction.source();

                let address = immediate + source;
                let target = instruction.target();

                // TODO: Write queue?
                self.mmu.write(address, target);
            }
            0b101110 => {
                panic!("SWR")
            }
            0b110000 => {
                panic!("LWC0")
            }
            0b110001 => {
                panic!("LWC1")
            }
            0b110010 => {
                panic!("LWC2")
            }
            0b110011 => {
                panic!("LWC3")
            }
            0b111000 => {
                panic!("SWC0")
            }
            0b111001 => {
                panic!("SWC1")
            }
            0b111010 => {
                panic!("SWC2")
            }
            0b111011 => {
                panic!("SWC3")
            }
            _ => {
                panic!("Invalid instruction")
            }
        }

        self.pc = self.pc.wrapping_add(4);
    }
}

struct Instruction(u32);

impl Instruction {
    // Opcode is last 6 bits
    pub fn opcode(&self) -> u32 {
        self.0 >> 26
    }

    // Secondary opcode is first 6 bits
    pub fn secondary_opcode(&self) -> u32 {
        self.0 & 0x3F
    }

    // Immediate values are first 16 bits
    pub fn immediate(&self) -> u32 {
        self.0 & 0xFFFF
    }

    // Target is bit 16..20
    pub fn target(&self) -> u32 {
        (self.0 >> 16) & 0x1F
    }

    // Source is bit 21..25
    pub fn source(&self) -> u32 {
        (self.0 >> 21) & 0x1F
    }
}
