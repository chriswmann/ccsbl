use strum::FromRepr;

#[derive(Debug, FromRepr, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Op {
    Pop = 0x01,
    Add,
    Inc,
    Dec,
    Sub,
    Mul,
    Div,
    Print,

    // with argument
    Push,
}
