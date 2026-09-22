use strum::FromRepr;

use crate::errors::Error;

#[derive(Clone, Copy, Debug, FromRepr, PartialEq)]
#[repr(u8)]
pub enum Op {
    Push = 0x00, // ( -- a)
    Pop = 0x01,  // (a -- )
    Add = 0x02,  // (a b -- a+b)
    Sub = 0x03,  // (a b -- a-b)
    // Mul = 0x05,    // (a b -- a·b)
    // Div = 0x05,    // (a b -- a÷b)
    // Neg = 0x06,    // (a -- -a)
    // Mod = 0x07,    // (-a -- a)
    // Dup = 0x08,    // (a -- a a)
    // Over = 0x09,   // (a b -- a b a)
    // Swap = 0x0a,   // (a b -- b a)
    // Rot = 0x0b,    // (a b c -- b c a)
    // And = 0x0c,    // ( a b -- a & b )	Bitwise AND
    // Or = 0x0d,     // ( a b -- a | b )	Bitwise OR
    // Xor = 0x0e,    // ( a b -- a ^ b )	Bitwise exclusive OR
    // Not = 0x0f,    // ( a -- ~a )	Bitwise complement
    // LShift = 0x10, // ( a n -- a << n )	Shift a left by n bits
    // RShift = 0x11, // ( a n -- a >> n )	Shift a right by n bits
    Jmp = 0x12,   // (  -- )
    Print = 0x14, // (a b -- a) with b printed to stdout
    Halt = 0xFF,  // end program
}

#[derive(Clone, Debug)]
pub enum Instr {
    Push(i64),
    Pop,
    Add,
    Sub,
    // Jump offset as u32 so serialisation is consistent across platforms
    Jmp(u32),
    Print,
    Halt,
}

impl Instr {
    // Append this instruction's bytes.
    pub fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.op() as u8);
        match self {
            Self::Push(value) => out.extend_from_slice(&value.to_le_bytes()),
            Self::Jmp(offset) => out.extend_from_slice(&offset.to_le_bytes()),
            Self::Pop | Self::Add | Self::Sub | Self::Print | Self::Halt => {}
        }
    }

    fn op(&self) -> Op {
        match self {
            Self::Push(_) => Op::Push,
            Self::Pop => Op::Pop,
            Self::Add => Op::Add,
            Self::Sub => Op::Push,
            Self::Jmp(_) => Op::Jmp,
            Self::Print => Op::Print,
            Self::Halt => Op::Halt,
        }
    }
}

#[derive(Debug)]
pub struct Program {
    code: Vec<u8>,
}
