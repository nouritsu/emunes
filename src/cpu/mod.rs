mod instruction;
mod status;

#[cfg(test)]
mod tests;

use instruction::{AddressingMode, Mnemonic, OPCODES, Operand};
use status::Status;

#[derive(Debug)]
pub struct Cpu {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub stack_pointer: u8,
    pub status: Status,
    pub program_counter: u16,
    memory: [u8; 0x10000],
}

enum Flow {
    Halt,
    Continue,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: 0,
            status: Status::default(),
            program_counter: 0,
            memory: [0; 0x10000],
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = 0xFD;
        self.status = Status::default();
        self.program_counter = self.mem_read_u16(0xFFFC); // from reset vector
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        let start = 0x8000;
        let end = 0x8000 + program.len();

        self.memory[start..end].copy_from_slice(&program);
        self.mem_write_u16(0xFFFC, 0x8000); // reset vector -> program start
    }

    pub fn run(&mut self) {
        loop {
            let byte = self.mem_read(self.program_counter);
            let opcode = OPCODES[byte as usize].expect("unknown instruction");
            self.program_counter += 1;

            let operand = self.resolve(opcode.mode);
            match self.apply(opcode.mnemonic, operand) {
                Flow::Halt => return,
                Flow::Continue => continue,
            }
        }
    }

    // Memory Helpers
    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn mem_read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr.wrapping_add(1)) as u16;

        (hi << 8) | lo
    }

    pub fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xff) as u8;
        let hi = (data >> 8) as u8;

        self.mem_write(addr, lo);
        self.mem_write(addr.wrapping_add(1), hi);
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.mem_read(self.program_counter);
        self.program_counter += 1;
        byte
    }

    fn next_word(&mut self) -> u16 {
        let word = self.mem_read_u16(self.program_counter);
        self.program_counter += 2;
        word
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write(0x0100 + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }
    fn stack_pull(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(0x0100 + self.stack_pointer as u16)
    }

    fn stack_push_u16(&mut self, data: u16) {
        self.stack_push((data >> 8) as u8);
        self.stack_push((data & 0xff) as u8);
    }

    fn stack_pull_u16(&mut self) -> u16 {
        let lo = self.stack_pull() as u16;
        let hi = self.stack_pull() as u16;

        (hi << 8) | lo
    }

    // Instruction Helpers
    fn resolve(&mut self, mode: AddressingMode) -> Operand {
        use AddressingMode as Mode;
        match mode {
            Mode::Implied => Operand::Implied,

            Mode::Accumulator => Operand::Accumulator,

            Mode::ZeroPage => Operand::Address(self.next_byte() as u16),
            Mode::ZeroPageX => {
                Operand::Address(self.next_byte().wrapping_add(self.register_x) as u16)
            }
            Mode::ZeroPageY => {
                Operand::Address(self.next_byte().wrapping_add(self.register_y) as u16)
            }

            Mode::Absolute => Operand::Address(self.next_word()),
            Mode::AbsoluteX => {
                Operand::Address(self.next_word().wrapping_add(self.register_x as u16))
            }
            Mode::AbsoluteY => {
                Operand::Address(self.next_word().wrapping_add(self.register_y as u16))
            }

            Mode::Indirect => {
                let ptr = self.next_word();
                let lo = self.mem_read(ptr) as u16;
                let hi = self.mem_read((ptr & 0xFF00) | (ptr.wrapping_add(1) & 0x00FF)) as u16;

                Operand::Address((hi << 8) | lo)
            }
            Mode::IndirectX => {
                let base = self.next_byte().wrapping_add(self.register_x);
                let lo = self.mem_read(base as u16) as u16;
                let hi = self.mem_read(base.wrapping_add(1) as u16) as u16;

                Operand::Address((hi << 8) | lo)
            }
            Mode::IndirectY => {
                let base = self.next_byte();
                let lo = self.mem_read(base as u16) as u16;
                let hi = self.mem_read(base.wrapping_add(1) as u16) as u16;

                Operand::Address(((hi << 8) | lo).wrapping_add(self.register_y as u16))
            }

            Mode::Immediate => {
                let addr = self.program_counter;
                self.program_counter += 1;
                Operand::Address(addr)
            }

            Mode::Relative => {
                let offset = self.next_byte() as i8; // i8 to sign extend when cast to u16
                Operand::Address(self.program_counter.wrapping_add(offset as u16))
            }
        }
    }

    fn apply(&mut self, mnemonic: Mnemonic, operand: Operand) -> Flow {
        match mnemonic {
            // Load / Store
            Mnemonic::LDA => {
                let byte = self.op_read(operand);
                self.register_a = byte;
                self.update_zn(byte);
            }
            Mnemonic::LDX => {
                let byte = self.op_read(operand);
                self.register_x = byte;
                self.update_zn(byte);
            }
            Mnemonic::LDY => {
                let byte = self.op_read(operand);
                self.register_y = byte;
                self.update_zn(byte);
            }
            Mnemonic::STA => self.op_write(operand, self.register_a),
            Mnemonic::STX => self.op_write(operand, self.register_x),
            Mnemonic::STY => self.op_write(operand, self.register_y),

            // Register Transfers
            Mnemonic::TAX => {
                self.register_x = self.register_a;
                self.update_zn(self.register_x);
            }
            Mnemonic::TAY => {
                self.register_y = self.register_a;
                self.update_zn(self.register_y);
            }
            Mnemonic::TXA => {
                self.register_a = self.register_x;
                self.update_zn(self.register_a);
            }
            Mnemonic::TYA => {
                self.register_a = self.register_y;
                self.update_zn(self.register_a);
            }
            Mnemonic::TSX => {
                self.register_x = self.stack_pointer;
                self.update_zn(self.register_x);
            }
            Mnemonic::TXS => self.stack_pointer = self.register_x,

            // Stack
            Mnemonic::PHA => self.stack_push(self.register_a),
            Mnemonic::PHP => self.stack_push(self.status.to_byte(true)),
            Mnemonic::PLA => {
                self.register_a = self.stack_pull();
                self.update_zn(self.register_a);
            }
            Mnemonic::PLP => self.status = Status::from_byte(self.stack_pull()),

            // Logical
            Mnemonic::AND => {
                self.register_a &= self.op_read(operand);
                self.update_zn(self.register_a);
            }
            Mnemonic::EOR => {
                self.register_a ^= self.op_read(operand);
                self.update_zn(self.register_a);
            }
            Mnemonic::ORA => {
                self.register_a |= self.op_read(operand);
                self.update_zn(self.register_a);
            }
            Mnemonic::BIT => {
                let byte = self.op_read(operand);
                self.status.zero = (self.register_a & byte) == 0;
                self.status.overflow = byte & 0b0100_0000 != 0;
                self.status.negative = byte & 0b1000_0000 != 0;
            }

            // Arithmetic
            Mnemonic::ADC => self.add(self.op_read(operand)),
            Mnemonic::SBC => self.add(!self.op_read(operand)),
            Mnemonic::CMP => self.compare(self.register_a, self.op_read(operand)),
            Mnemonic::CPX => self.compare(self.register_x, self.op_read(operand)),
            Mnemonic::CPY => self.compare(self.register_y, self.op_read(operand)),

            // Increments / Decrements
            Mnemonic::INC => {
                let value = self.op_read(operand).wrapping_add(1);
                self.op_write(operand, value);
                self.update_zn(value);
            }
            Mnemonic::INX => {
                self.register_x = self.register_x.wrapping_add(1);
                self.update_zn(self.register_x);
            }
            Mnemonic::INY => {
                self.register_y = self.register_y.wrapping_add(1);
                self.update_zn(self.register_y);
            }
            Mnemonic::DEC => {
                let value = self.op_read(operand).wrapping_sub(1);
                self.op_write(operand, value);
                self.update_zn(value);
            }
            Mnemonic::DEX => {
                self.register_x = self.register_x.wrapping_sub(1);
                self.update_zn(self.register_x);
            }
            Mnemonic::DEY => {
                self.register_y = self.register_y.wrapping_sub(1);
                self.update_zn(self.register_y);
            }

            // Shifts
            Mnemonic::ASL => {
                let value = self.op_read(operand);
                let result = value << 1;

                self.op_write(operand, result);
                self.status.carry = value & 0b1000_0000 != 0;
                self.update_zn(result);
            }
            Mnemonic::LSR => {
                let value = self.op_read(operand);
                let result = value >> 1;

                self.op_write(operand, result);
                self.status.carry = value & 0b0000_0001 != 0;
                self.update_zn(result);
            }
            Mnemonic::ROL => {
                let value = self.op_read(operand);
                let result = (value << 1) | (self.status.carry as u8);

                self.op_write(operand, result);
                self.status.carry = value & 0b1000_0000 != 0;
                self.update_zn(result);
            }
            Mnemonic::ROR => {
                let value = self.op_read(operand);
                let result = (value >> 1) | ((self.status.carry as u8) << 7);

                self.op_write(operand, result);
                self.status.carry = value & 0b0000_0001 != 0;
                self.update_zn(result);
            }

            // Jumps / Calls
            Mnemonic::JMP => self.jump(operand),
            Mnemonic::JSR => {
                self.stack_push_u16(self.program_counter.wrapping_sub(1));
                self.jump(operand);
            }
            Mnemonic::RTS => self.program_counter = self.stack_pull_u16().wrapping_add(1),

            // Branches
            Mnemonic::BCS => self.branch(self.status.carry, operand),
            Mnemonic::BCC => self.branch(!self.status.carry, operand),
            Mnemonic::BEQ => self.branch(self.status.zero, operand),
            Mnemonic::BNE => self.branch(!self.status.zero, operand),
            Mnemonic::BMI => self.branch(self.status.negative, operand),
            Mnemonic::BPL => self.branch(!self.status.negative, operand),
            Mnemonic::BVS => self.branch(self.status.overflow, operand),
            Mnemonic::BVC => self.branch(!self.status.overflow, operand),

            // Status Flag Changes
            Mnemonic::CLC => self.status.carry = false,
            Mnemonic::CLD => self.status.decimal = false,
            Mnemonic::CLI => self.status.interrupt = false,
            Mnemonic::CLV => self.status.overflow = false,
            Mnemonic::SEC => self.status.carry = true,
            Mnemonic::SED => self.status.decimal = true,
            Mnemonic::SEI => self.status.interrupt = true,

            // System
            Mnemonic::BRK => return Flow::Halt,
            Mnemonic::NOP => { /* no operation */ }
            Mnemonic::RTI => {
                self.status = Status::from_byte(self.stack_pull());
                self.program_counter = self.stack_pull_u16();
            }
        }

        Flow::Continue
    }

    fn jump(&mut self, operand: Operand) {
        if let Operand::Address(addr) = operand {
            self.program_counter = addr;
        } else {
            unreachable!();
        }
    }

    fn branch(&mut self, condition: bool, operand: Operand) {
        if condition {
            self.jump(operand); // a branch is a conditional jump
        }
    }

    fn add(&mut self, value: u8) {
        let sum = self.register_a as u16 + value as u16 + self.status.carry as u16;
        let result = sum as u8;

        self.status.carry = sum > 0xFF;
        self.status.overflow = (self.register_a ^ result) & (value ^ result) & 0b1000_0000 != 0;
        self.register_a = result;
        self.update_zn(result);
    }

    fn compare(&mut self, register: u8, value: u8) {
        self.status.carry = register >= value;
        self.update_zn(register.wrapping_sub(value));
    }

    // Operand Helpers
    fn op_read(&self, operand: Operand) -> u8 {
        match operand {
            Operand::Implied => panic!("instruction has no operand to read"),
            Operand::Accumulator => self.register_a,
            Operand::Address(addr) => self.mem_read(addr),
        }
    }

    fn op_write(&mut self, operand: Operand, data: u8) {
        match operand {
            Operand::Implied => panic!("instruction has no operand to write"),
            Operand::Accumulator => self.register_a = data,
            Operand::Address(addr) => self.mem_write(addr, data),
        }
    }

    // Flag Helpers
    fn update_zn(&mut self, value: u8) {
        self.status.zero = value == 0;
        self.status.negative = value & 0b1000_0000 != 0;
    }
}
