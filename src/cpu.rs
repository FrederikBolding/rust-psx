use crate::mmu::MMU;

#[derive(Clone, Copy)]
struct InstructionCacheLine {
    valid: usize, // The index of the first valid word
    tag: u32,
    data: [u32; 4], // Four words each cache line
}

impl InstructionCacheLine {
    pub fn new() -> Self {
        Self {
            valid: 0xFFFF,
            tag: 0,
            data: [0; 4],
        }
    }
}

pub struct CPU {
    registers: [u32; 32], // R0..R31
    // Since the CPU is pipelined, we need to keep track of multiple program counters to properly handle branches
    current_pc: u32, // The currently executing instruction
    pc: u32,         // Points to the next instruction, NOT the currently executing instruction
    next_pc: u32,    // Points to the following instruction after pc
    hi: u32,         // Registers used for mult and div results
    lo: u32,         // Registers used for mult and div results
    mmu: MMU,
    cop0: Coprocessor,
    next_load: (u32, u32), // Temporarily store loaded values between instruction execution
    instruction_cache: [InstructionCacheLine; 256],
}

const START_PC: u32 = 0xBFC00000;

impl CPU {
    pub fn new(mmu: MMU) -> Self {
        Self {
            registers: [0; 32],
            current_pc: 0,
            pc: START_PC,
            next_pc: START_PC.wrapping_add(4),
            hi: 0,
            lo: 0,
            mmu,
            cop0: Coprocessor::new(),
            next_load: (0, 0),
            instruction_cache: [InstructionCacheLine::new(); 256],
        }
    }

    fn load_instruction(&self) -> Instruction {
        // TODO: If the instruction cache is used one step != one cycle
        if self.mmu.is_instruction_cache_enabled() && self.pc < 0xa0000000 {
            // Cache tag is bit 12..30
            let tag = self.pc & 0x7FFFF000;

            // Line is bit 4..11
            let line = ((self.pc >> 4) & 0xFF) as usize;

            // Line is bit 2..3
            let index = ((self.pc >> 2) & 3) as usize;

            let mut line = self.instruction_cache[line];

            // Refetch instruction if cache is invalid
            if (tag != line.tag) || (line.valid > index) || (line.valid > 4) {
                let mut address = self.pc;
                for i in index..4 {
                    let instruction = self.mmu.read(address, 4);
                    line.data[i] = instruction;

                    address += 4;
                }

                line.tag = tag;
                line.valid = index;
            }

            return Instruction(line.data[index]);
        }

        let word = self.mmu.read(self.pc, 4);

        Instruction(word)
    }

    pub fn step(&mut self) {
        let instruction = self.load_instruction();

        self.current_pc = self.pc;
        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);

        self.execute(instruction);

        // Each instruction takes one cycle
        self.mmu.step(1);
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

                    self.finish_load();

                    self.registers[d] = value;
                }
                0b000010 => {
                    // SRL
                    let shift = instruction.immediate_shift();
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[t] >> shift;

                    self.finish_load();

                    self.registers[d] = value as u32;
                }
                0b000011 => {
                    // SRA
                    let shift = instruction.immediate_shift();
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = (self.registers[t] as i32) >> shift;

                    self.finish_load();

                    self.registers[d] = value as u32;
                }
                0b000100 => {
                    panic!("SSLV")
                }
                0b000110 => {
                    // SRLV
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[t] >> (self.registers[s] & 0x1F);

                    self.finish_load();

                    self.registers[d] = value;
                }
                0b000111 => {
                    panic!("SRAV")
                }
                0b001000 => {
                    // JR
                    let next = self.registers[instruction.s() as usize];

                    self.finish_load();

                    self.next_pc = next;
                }
                0b001001 => {
                    // JALR
                    let s = instruction.s() as usize;
                    let d = instruction.d() as usize;

                    let return_address = self.next_pc;

                    self.next_pc = self.registers[s];

                    self.finish_load();

                    self.registers[d] = return_address;
                }
                0b001100 => {
                    panic!("SYSCALL")
                }
                0b001101 => {
                    panic!("BREAK")
                }
                0b010000 => {
                    // MFHI
                    let d = instruction.d() as usize;

                    self.finish_load();

                    self.registers[d] = self.hi;
                }
                0b010001 => {
                    panic!("MTHI")
                }
                0b010010 => {
                    // MFLO
                    let d = instruction.d() as usize;

                    self.finish_load();

                    self.registers[d] = self.lo;
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
                    // DIV
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;

                    let numerator = self.registers[s] as i32;
                    let denominator = self.registers[t] as i32;

                    self.finish_load();

                    // TODO: Handle these cases
                    if denominator == 0 {
                        panic!("Division by zero");
                    } else if denominator == -1 && numerator as u32 == (i32::MIN as u32) {
                        panic!("Division by -1");
                    }

                    // Default case
                    self.hi = (numerator % denominator) as u32;
                    self.lo = (numerator / denominator) as u32;
                }
                0b011011 => {
                    // DIVU
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;

                    let numerator = self.registers[s];
                    let denominator = self.registers[t];

                    self.finish_load();

                    // TODO: Handle this case
                    if denominator == 0 {
                        panic!("Division by zero");
                    }

                    // Default case
                    self.hi = numerator % denominator;
                    self.lo = numerator / denominator;
                }
                0b100000 => {
                    // ADD
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let a = self.registers[s] as i32;
                    let b = self.registers[t] as i32;

                    self.finish_load();

                    match a.checked_add(b) {
                        Some(value) => self.registers[d] = value as u32,
                        None => panic!("Overflow not handled"),
                    }
                }
                0b100001 => {
                    // ADDU
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let a = self.registers[s];
                    let b = self.registers[t];

                    let value = a.wrapping_add(b);

                    self.finish_load();

                    self.registers[d] = value;
                }
                0b100010 => {
                    // SUB
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let a = self.registers[s] as i32;
                    let b = self.registers[t] as i32;

                    self.finish_load();

                    match a.checked_sub(b) {
                        Some(value) => self.registers[d] = value as u32,
                        None => panic!("Underflow not handled"),
                    }
                }
                0b100011 => {
                    // SUBU
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let a = self.registers[s];
                    let b = self.registers[t];

                    self.finish_load();

                    self.registers[d] = a.wrapping_sub(b);
                }
                0b100100 => {
                    // AND
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[s] & self.registers[t];

                    self.finish_load();

                    self.registers[d] = value;
                }
                0b100101 => {
                    // OR
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = self.registers[s] | self.registers[t];

                    self.finish_load();

                    self.registers[d] = value;
                }
                0b100110 => {
                    panic!("XOR")
                }
                0b100111 => {
                    panic!("NOR")
                }
                0b101010 => {
                    // SLT
                    let s = instruction.s() as usize;
                    let t = instruction.t() as usize;
                    let d = instruction.d() as usize;

                    let value = if (self.registers[s] as i32) < (self.registers[t] as i32) {
                        1
                    } else {
                        0
                    };

                    self.finish_load();

                    self.registers[d] = value;
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

                    self.finish_load();

                    self.registers[d] = value;
                }
                _ => {
                    panic!("Invalid instruction")
                }
            },
            0b000001 => {
                match instruction.t() {
                    0b00000 => {
                        // BLTZ
                        let s = instruction.s() as usize;

                        let value = (self.registers[s] as i32) < 0;

                        if value {
                            let immediate = instruction.immediate_sign_extended();
                            self.next_pc = self.pc.wrapping_add(immediate << 2);
                        }

                        self.finish_load();
                    }
                    0b00001 => {
                        // BGEZ
                        let s = instruction.s() as usize;

                        let value = (self.registers[s] as i32) >= 0;

                        if value {
                            let immediate = instruction.immediate_sign_extended();
                            self.next_pc = self.pc.wrapping_add(immediate << 2);
                        }

                        self.finish_load();
                    }
                    0b10000 => {
                        panic!("BLTZAL");
                    }
                    0b10001 => {
                        panic!("BGEZAL");
                    }
                    t => panic!("Unsupported branching instruction: {}", t),
                }
            }
            0b000010 => {
                // J
                let jump = instruction.immediate_jump();
                self.next_pc = (self.pc & 0xF0000000) | jump;

                self.finish_load();
            }
            0b000011 => {
                // JAL
                let return_address = self.next_pc;

                let jump = instruction.immediate_jump();
                self.next_pc = (self.pc & 0xF0000000) | jump;

                self.finish_load();

                self.registers[31] = return_address;
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

                self.finish_load();
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

                self.finish_load();
            }
            0b000110 => {
                // BLEZ
                let s = instruction.s() as usize;

                let value = (self.registers[s] as i32) <= 0;

                if value {
                    let immediate = instruction.immediate_sign_extended();
                    self.next_pc = self.pc.wrapping_add(immediate << 2);
                }

                self.finish_load();
            }
            0b000111 => {
                // BGTZ
                let s = instruction.s() as usize;

                let value = (self.registers[s]) as i32 > 0;

                if value {
                    let immediate = instruction.immediate_sign_extended();
                    self.next_pc = self.pc.wrapping_add(immediate << 2);
                }

                self.finish_load();
            }
            0b001000 => {
                // ADDI
                let immediate = instruction.immediate_sign_extended() as i32;
                let t = instruction.t() as usize;
                let s = instruction.s() as usize;

                let a = self.registers[s] as i32;

                self.finish_load();

                match a.checked_add(immediate) {
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

                self.finish_load();

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

                self.finish_load();

                self.registers[t] = value;
            }
            0b001011 => {
                // SLTIU
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = if self.registers[s] < immediate { 1 } else { 0 };

                self.finish_load();

                self.registers[t] = value;
            }
            0b001100 => {
                // ANDI
                let immediate = instruction.immediate();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = self.registers[s] & immediate;

                self.finish_load();

                self.registers[t] = value;
            }
            0b001101 => {
                // ORI
                let immediate = instruction.immediate();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let value = self.registers[s] | immediate;

                self.finish_load();

                self.registers[t] = value;
            }
            0b001110 => {
                panic!("XORI")
            }
            0b001111 => {
                // LUI
                let immediate = instruction.immediate();
                let t = instruction.t() as usize;

                let value = immediate << 16;

                self.finish_load();

                self.registers[t] = value;
            }
            0b010000 => {
                // COP0
                let coprocessor_opcode = instruction.coprocessor_opcode();
                match coprocessor_opcode {
                    0b00000 => {
                        // MFC0
                        let r = instruction.t() as usize;
                        let cop0_r = instruction.d() as usize;

                        match cop0_r {
                            3 | 5 | 6 | 7 | 9 | 11 => {
                                // No-op, ignoring breakpoints for now
                            }
                            12 => {
                                self.setup_load(r as u32, self.cop0.status);
                            }
                            _ => panic!("Unsupported COP0 register {}", cop0_r),
                        }
                    }
                    0b00100 => {
                        // MTC0
                        let r = instruction.t() as usize;
                        let cop0_r = instruction.d() as usize;

                        let value = self.registers[r];

                        self.finish_load();

                        match cop0_r {
                            3 | 5 | 6 | 7 | 9 | 11 => {
                                // No-op, ignoring breakpoints for now
                            }
                            12 => {
                                self.cop0.status = value;
                            }
                            13 => {
                                self.cop0.cause = (self.cop0.cause & !0x300) | (value & 0x300);
                            }
                            _ => panic!("Unsupported COP0 register {}", cop0_r),
                        }
                    }
                    0b10000 => {
                        // RFE
                        self.finish_load();
                        let mode = self.cop0.status & 0x3F;
                        self.cop0.status = (self.cop0.status & !0xF) | (mode >> 2);
                    }
                    o => {
                        panic!("Unhandled coprocessor opcode {}", o);
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

                // Should be sign-extended
                let value = self.mmu.read(address, 1) as i8;
                self.setup_load(t as u32, value as u32);
            }
            0b100001 => {
                // LH
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let address = self.registers[s].wrapping_add(immediate);

                // Should be sign-extended
                let value = self.mmu.read(address, 2) as i16;
                self.setup_load(t as u32, value as u32);
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

                let value = self.mmu.read(address, 4);
                self.setup_load(t as u32, value as u32);
            }
            0b100100 => {
                // LBU
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let address = self.registers[s].wrapping_add(immediate);

                let value = self.mmu.read(address, 1);
                self.setup_load(t as u32, value as u32);
            }
            0b100101 => {
                // LHU
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;
                let t = instruction.t() as usize;

                let address = self.registers[s].wrapping_add(immediate);

                let value = self.mmu.read(address, 2);
                self.setup_load(t as u32, value as u32);
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

                self.finish_load();

                if self.cop0.is_cache_isolated() {
                    self.store_instruction_cache(address, value);
                    return;
                }

                self.mmu.write(address, 1, value);
            }
            0b101001 => {
                // SH
                let immediate = instruction.immediate_sign_extended();
                let s = instruction.s() as usize;

                let address = self.registers[s].wrapping_add(immediate);
                let t = instruction.t() as usize;
                let value = self.registers[t];

                self.finish_load();

                if self.cop0.is_cache_isolated() {
                    self.store_instruction_cache(address, value);
                    return;
                }

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

                self.finish_load();

                if self.cop0.is_cache_isolated() {
                    self.store_instruction_cache(address, value);
                    return;
                }

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

    fn setup_load(&mut self, register: u32, value: u32) {
        if self.next_load.0 != register {
            self.registers[self.next_load.0 as usize] = self.next_load.1;
        }

        self.next_load = (register, value);
    }

    fn finish_load(&mut self) {
        self.registers[self.next_load.0 as usize] = self.next_load.1;

        self.next_load = (0, 0);
    }

    fn store_instruction_cache(&mut self, address: u32, value: u32) {
        let line = ((address >> 4) & 0xFF) as usize;
        let index = ((address >> 2) & 3) as usize;

        let mut cache_line = self.instruction_cache[line];

        if self.mmu.is_instruction_cache_tag_test_mode() {
            cache_line.tag = value;
        } else {
            cache_line.data[index] = value;
        }

        cache_line.valid = 4;
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
        self.s()
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
