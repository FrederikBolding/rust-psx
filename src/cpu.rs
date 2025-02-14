use std::ops::Mul;

use crate::mmu::{BIOS_START, MMU};

pub struct CPU {
    registers: [u32; 32],
    pc: u32,
    mmu: MMU,
    cop0: Coprocessor,
}

impl CPU {
    pub fn new(mmu: MMU) -> Self {
        Self {
            registers: [0; 32],
            pc: BIOS_START,
            mmu,
            cop0: Coprocessor::new(),
        }
    }

    fn load_instruction(&self) -> Instruction {
        let word = self.mmu.read(self.pc);

        Instruction(word)
    }

    pub fn step(&mut self) {
        let instruction = self.load_instruction();

        let (next_pc) = self.execute(instruction);

        self.pc = next_pc;
    }

    fn execute(&mut self, instruction: Instruction) -> (u32) {
        let opcode = instruction.opcode();

        let secondary_opcode = instruction.secondary_opcode();

        match opcode {
            0b000000 => match secondary_opcode {
                0b000000 => {
                    // SLL
                    let shift = instruction.immediate_shift();
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[t] << shift;

                    self.registers[d] = value;

                    (self.pc.wrapping_add(4))
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
                    // JR
                    (self.registers[instruction.s() as usize])
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
                    // OR
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[s] | self.registers[t];

                    self.registers[d] = value;

                    (self.pc.wrapping_add(4))
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
                // J
                let jump = instruction.immediate_jump();
                (self.pc.wrapping_add(4) & 0xF0000000) | jump
            }
            0b000011 => {
                panic!("JAL")
            }
            0b000100 => {
                panic!("BEQ")
            }
            0b000101 => {
                // BNE
                let s = instruction.s();
                let t = instruction.t();

                let value = self.registers[s as usize] != self.registers[t as usize];

                if value {
                    let immediate = instruction.immediate_sign_extended();
                    return (self.pc.wrapping_add(4).wrapping_add(immediate << 2));
                }

                (self.pc.wrapping_add(4))
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
                // ADDIU
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = self.registers[s].wrapping_add(immediate);

                self.registers[t] = value;

                (self.pc.wrapping_add(4))
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
                // ORI
                let immediate = instruction.immediate();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = self.registers[s] | immediate;

                self.registers[t] = value;

                (self.pc.wrapping_add(4))
            }
            0b001110 => {
                panic!("XORI")
            }
            0b001111 => {
                // LUI
                let value = instruction.immediate();
                let t = instruction.t() as usize;

                self.registers[t] = value << 16;

                (self.pc.wrapping_add(4))
            }
            0b010000 => {
                // COP0
                let coprocessor_opcode = instruction.coprocessor_opcode();
                match coprocessor_opcode {
                    0b0000 => {
                        panic!("MFC0");
                    }
                    0b0010 => {
                        panic!("CFC0");
                    }
                    0b0100 => {
                        // MTC0
                        let r = instruction.t();
                        let cop0_r = instruction.d();

                        let value = self.registers[r as usize];

                        match cop0_r {
                            12 => {
                                self.cop0.status = value;
                            }
                            _ => panic!("Unsupported COP0 register"),
                        }

                        (self.pc.wrapping_add(4))
                    }
                    0b1000 => {
                        panic!("CTC0");
                    }
                    _ => {
                        panic!("Unhandled coprocessor opcode")
                    }
                }
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
                // SW
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s();

                let address = self.registers[s as usize].wrapping_add(immediate);
                let t = instruction.t();
                let value = self.registers[t as usize];

                if self.cop0.is_cache_isolated() {
                    // TODO: Handle writing to the cache
                    return (self.pc.wrapping_add(4));
                }

                // TODO: Write queue?
                self.mmu.write(address, value);

                (self.pc.wrapping_add(4))
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

    // Coprocessor opcode bit 21..25
    pub fn coprocessor_opcode(&self) -> u32 {
        (self.0 >> 21) & 0x1F
    }

    // Immediate values are first 16 bits
    pub fn immediate(&self) -> u32 {
        self.0 & 0xFFFF
    }

    // Immediate values are first 16 bits
    pub fn immediate_sign_extended(&self) -> u32 {
        ((self.0 & 0xFFFF) as i16) as u32
    }

    // T is bit 16..20
    pub fn t(&self) -> u32 {
        (self.0 >> 16) & 0x1F
    }

    // S is bit 21..25
    pub fn s(&self) -> u32 {
        (self.0 >> 21) & 0x1F
    }

    // D is bit 11..15
    pub fn d(&self) -> u32 {
        (self.0 >> 11) & 0x1F
    }

    // Immediate shift is bit 6..10
    pub fn immediate_shift(&self) -> u32 {
        (self.0 >> 6) & 0x1F
    }

    // Immediate jump values are bit 0..25
    pub fn immediate_jump(&self) -> u32 {
        (self.0 & 0x3FFFFFF) << 2
    }
}

struct Coprocessor {
    status: u32, // System status register
    cause: u32,  // Describes the most recently recognized exception
    epc: u32,    // Retrun address from trap
}

impl Coprocessor {
    pub fn new() -> Self {
        Self {
            status: 0,
            cause: 0,
            epc: 0,
        }
    }

    pub fn is_cache_isolated(&self) -> bool {
        self.status & 0x10000 != 0
    }
}
