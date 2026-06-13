mod instruction;
mod status;

use crate::cpu::instruction::{AddressingMode, Mnemonic, OPCODES, Operand};
use status::Status;

#[derive(Debug)]
pub struct Cpu {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: Status,
    pub program_counter: u16,
    memory: [u8; 0x10000],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: Status::default(),
            program_counter: 0,
            memory: [0; 0x10000],
        }
    }

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

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        let start = 0x8000;
        let end = 0x8000 + program.len();

        self.memory[start..end].copy_from_slice(&program);
        self.program_counter = 0x8000;
    }

    pub fn run(&mut self) {
        loop {
            let byte = self.mem_read(self.program_counter);
            let opcode = OPCODES[byte as usize].expect("unknown instruction");
            self.program_counter += 1;

            let operand = self.resolve(opcode.mode);
            self.apply(opcode.mnemonic, operand);
        }
    }

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

    fn apply(&mut self, mnemonic: Mnemonic, operand: Operand) {}
}
