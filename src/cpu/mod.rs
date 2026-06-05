mod opcode;
mod status;

pub use opcode::Opcode;
use status::Status;

#[derive(Default, Debug)]
pub struct Cpu {
    pub register_a: u8,
    pub status: Status,
    pub program_counter: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    fn update_zn(&mut self, value: u8) {
        self.status.zero = value == 0;
        self.status.negative = value & 0b1000_0000 != 0;
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let op = program[self.program_counter as usize];
            self.program_counter += 1;

            use Opcode as Op;
            match Op::from(op) {
                Op::LDA => {
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.register_a = param;
                    self.update_zn(self.register_a);
                }

                Op::BRK => return,

                Op::Unknown(o) => panic!("unknown opcode: {:#04X}", o),
            }
        }
    }
}
