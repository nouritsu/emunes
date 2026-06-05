#[derive(Debug, Clone, Copy, Default)]
pub struct Status {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl Status {
    pub fn to_byte(self, break_flag: bool) -> u8 {
        (self.carry as u8)
            | (self.zero as u8) << 1
            | (self.interrupt as u8) << 2
            | (self.decimal as u8) << 3
            | (break_flag as u8) << 4
            | 1 << 5
            | (self.overflow as u8) << 6
            | (self.negative as u8) << 7
    }

    pub fn from_byte(b: u8) -> Self {
        Status {
            carry: b & 0b0000_0001 != 0,
            zero: b & 0b0000_0010 != 0,
            interrupt: b & 0b0000_0100 != 0,
            decimal: b & 0b0000_1000 != 0,
            // bit 4 ignored
            // bit 5 ignored
            overflow: b & 0b0100_0000 != 0,
            negative: b & 0b1000_0000 != 0,
        }
    }
}
