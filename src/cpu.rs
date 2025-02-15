use crate::mmu::MMU;

pub struct CPU {
    registers: [u32; 32],
    // Since the CPU is pipelined, we need to keep track of multiple program counters to properly handle branches
    current_pc: u32, // The currently executing instruction
    pc: u32,         // Points to the next instruction, NOT the currently executing instruction
    next_pc: u32,
    mmu: MMU,
    cop0: Coprocessor,
}

const START_PC: u32 = 0xBFC00000;

impl CPU {
    pub fn new(mmu: MMU) -> Self {
        Self {
            registers: [0; 32],
            current_pc: 0,
            pc: START_PC,
            next_pc: START_PC.wrapping_add(4),
            mmu,
            cop0: Coprocessor::new(),
        }
    }

    fn load_instruction(&self) -> Instruction {
        let word = self.mmu.read(self.pc, 4);

        Instruction(word)
    }

    pub fn step(&mut self) {
        let instruction = self.load_instruction();

        self.current_pc = self.pc;
        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);

        self.execute(instruction);
    }

    fn execute(&mut self, instruction: Instruction) {
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
                }
                0b000010 => {
                    panic!("SRL")
                }
                0b000011 => {
                    // SRA
                    let shift = instruction.immediate_shift();
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = (self.registers[t] as i32) >> shift;

                    self.registers[d] = value as u32;
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
                    self.next_pc = self.registers[instruction.s() as usize];
                }
                0b001001 => {
                    // JALR
                    let s = instruction.s() as usize;
                    let d = instruction.d() as usize;

                    self.registers[d] = self.next_pc;

                    self.next_pc = self.registers[s];
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
                    // ADD
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    match (self.registers[s] as i32).checked_add(self.registers[t] as i32) {
                        Some(value) => self.registers[d] = value as u32,
                        None => panic!("Overflow not handled"),
                    }
                }
                0b100001 => {
                    // ADDU
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    self.registers[d] = self.registers[s].wrapping_add(self.registers[t]);
                }
                0b100010 => {
                    panic!("SUB")
                }
                0b100011 => {
                    // SUBU
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    self.registers[d] = self.registers[s].wrapping_sub(self.registers[t]);
                }
                0b100100 => {
                    // AND
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[s] & self.registers[t];

                    self.registers[d] = value;
                }
                0b100101 => {
                    // OR
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[s] | self.registers[t];

                    self.registers[d] = value;
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
                    // SLTU
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = if self.registers[s] < self.registers[t] {
                        1
                    } else {
                        0
                    };

                    self.registers[d] = value;
                }
                _ => {
                    panic!("Invalid instruction")
                }
            },
            0b000001 => {
                match instruction.d() {
                    0b00000 => {
                        // BLTZ
                        let s = instruction.s() as usize;

                        let value = (self.registers[s] as i32) < 0;

                        if value {
                            let immediate = instruction.immediate_sign_extended();
                            self.next_pc = self.pc.wrapping_add(immediate << 2);
                        }
                    }
                    0b00001 => {
                        panic!("BGEZ");
                    }
                    0b10000 => {
                        panic!("BLTZAL");
                    }
                    0b10001 => {
                        panic!("BGEZAL");
                    }
                    _ => panic!("Unsupported branching instruction"),
                }
            }
            0b000010 => {
                // J
                let jump = instruction.immediate_jump();
                self.next_pc = (self.pc & 0xF0000000) | jump
            }
            0b000011 => {
                // JAL
                self.registers[31] = self.next_pc;

                let jump = instruction.immediate_jump();
                self.next_pc = (self.pc & 0xF0000000) | jump
            }
            0b000100 => {
                // BEQ
                let s = instruction.s();
                let t = instruction.t();

                let value = self.registers[s as usize] == self.registers[t as usize];

                if value {
                    let immediate = instruction.immediate_sign_extended();
                    self.next_pc = self.pc.wrapping_add(immediate << 2);
                }
            }
            0b000101 => {
                // BNE
                let s = instruction.s();
                let t = instruction.t();

                let value = self.registers[s as usize] != self.registers[t as usize];

                if value {
                    let immediate = instruction.immediate_sign_extended();
                    self.next_pc = self.pc.wrapping_add(immediate << 2);
                }
            }
            0b000110 => {
                // BLEZ
                let s = instruction.s();

                let value = self.registers[s as usize] <= 0;

                if value {
                    let immediate = instruction.immediate_sign_extended();
                    self.next_pc = self.pc.wrapping_add(immediate << 2);
                }
            }
            0b000111 => {
                // BGTZ
                let s = instruction.s();

                let value = self.registers[s as usize] > 0;

                if value {
                    let immediate = instruction.immediate_sign_extended();
                    self.next_pc = self.pc.wrapping_add(immediate << 2);
                }
            }
            0b001000 => {
                // ADDI
                let immediate = instruction.immediate_sign_extended() as i32;
                let t = instruction.t() as usize;
                let s = instruction.s() as usize;

                match (self.registers[s] as i32).checked_add(immediate) {
                    Some(value) => self.registers[t] = value as u32,
                    None => panic!("Overflow not handled"),
                }
            }
            0b001001 => {
                // ADDIU
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = self.registers[s].wrapping_add(immediate);

                self.registers[t] = value;
            }
            0b001010 => {
                // SLTI
                let immediate = instruction.immediate_sign_extended() as i32;
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = if (self.registers[s] as i32) < immediate {
                    1
                } else {
                    0
                };

                self.registers[t] = value;
            }
            0b001011 => {
                panic!("SLTIU")
            }
            0b001100 => {
                // ANDI
                let immediate = instruction.immediate();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = self.registers[s] & immediate;

                self.registers[t] = value;
            }
            0b001101 => {
                // ORI
                let immediate = instruction.immediate();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = self.registers[s] | immediate;

                self.registers[t] = value;
            }
            0b001110 => {
                panic!("XORI")
            }
            0b001111 => {
                // LUI
                let value = instruction.immediate();
                let t = instruction.t() as usize;

                self.registers[t] = value << 16;
            }
            0b010000 => {
                // COP0
                let coprocessor_opcode = instruction.coprocessor_opcode();
                match coprocessor_opcode {
                    0b0000 => {
                        // MFC0
                        let r = instruction.t() as usize;
                        let cop0_r = instruction.d() as usize;

                        match cop0_r {
                            3 | 5 | 6 | 7 | 9 | 11 => {
                                // No-op, ignoring breakpoints for now
                            }
                            12 => {
                                self.registers[r] = self.cop0.status;
                            }
                            _ => panic!("Unsupported COP0 register {}", cop0_r),
                        }
                    }
                    0b0010 => {
                        panic!("CFC0");
                    }
                    0b0100 => {
                        // MTC0
                        let r = instruction.t() as usize;
                        let cop0_r = instruction.d() as usize;

                        let value = self.registers[r];

                        match cop0_r {
                            3 | 5 | 6 | 7 | 9 | 11 => {
                                // No-op, ignoring breakpoints for now
                            }
                            12 => {
                                self.cop0.status = value;
                            }
                            13 => {
                                self.cop0.cause = value;
                            }
                            _ => panic!("Unsupported COP0 register {}", cop0_r),
                        }
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
                // LB
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let address = self.registers[s].wrapping_add(immediate);

                // TODO: Load delay?
                // Should be sign-extended
                let value = self.mmu.read(address, 1) as i8;
                self.registers[t] = value as u32;
            }
            0b100001 => {
                // LH
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let address = self.registers[s].wrapping_add(immediate);

                // TODO: Load delay?
                // Should be sign-extended
                let value = self.mmu.read(address, 2) as i16;
                self.registers[t] = value as u32;
            }
            0b100010 => {
                panic!("LWL")
            }
            0b100011 => {
                // LW
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let address = self.registers[s].wrapping_add(immediate);

                // TODO: Load delay?
                let value = self.mmu.read(address, 4);
                self.registers[t] = value;
            }
            0b100100 => {
                // LBU
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let address = self.registers[s].wrapping_add(immediate);

                // TODO: Load delay?
                let value = self.mmu.read(address, 1);
                self.registers[t] = value;
            }
            0b100101 => {
                panic!("LHU")
            }
            0b100110 => {
                panic!("LWR")
            }
            0b101000 => {
                // SB
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;

                let address = self.registers[s].wrapping_add(immediate);
                let t = instruction.t() as usize;
                let value = self.registers[t];

                if self.cop0.is_cache_isolated() {
                    // TODO: Handle writing to the cache
                    return;
                }

                // TODO: Load delay?
                self.mmu.write(address, 1, value);
            }
            0b101001 => {
                // SH
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;

                let address = self.registers[s].wrapping_add(immediate);
                let t = instruction.t() as usize;
                let value = self.registers[t];

                if self.cop0.is_cache_isolated() {
                    // TODO: Handle writing to the cache
                    return;
                }

                // TODO: Load delay?
                self.mmu.write(address, 2, value);
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
                    return;
                }

                // TODO: Load delay?
                self.mmu.write(address, 4, value);
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
