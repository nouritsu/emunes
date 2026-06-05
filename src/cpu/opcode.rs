use num_enum::{FromPrimitive, IntoPrimitive};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive, IntoPrimitive)]
pub enum Opcode {
    LDA = 0xA9,
    BRK = 0x00,

    #[num_enum(catch_all)]
    Unknown(u8),
}
