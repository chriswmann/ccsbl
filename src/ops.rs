use strum::FromRepr;

#[derive(Clone, Copy, Debug, FromRepr, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Op {
    Pop = 0x01,
    Add,
    Sub,
    Print,
    Halt,
}
