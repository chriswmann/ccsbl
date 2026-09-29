use std::fmt;

use strum::FromRepr;

use crate::errors::Error;

#[derive(Clone, Copy, Debug, FromRepr, PartialEq)]
#[repr(u8)]
pub enum BytecodeOp {
    Push = 0x01,
    Pop = 0x02,   // (a -- )
    Print = 0x03, // (a b -- a) with b printed to stdout
    Add = 0x04,   // (a b -- a+b)
    Sub = 0x05,   // (a b -- a-b)
    Mul = 0x06,   // (a b -- a·b)
    Div = 0x07,   // (a b -- a÷b)
    // Neg = 0x08,    // (a -- -a)
    // Mod = 0x09,    // (-a -- a)
    // Dup = 0x0a,    // (a -- a a)
    // Over = 0x0b,   // (a b -- a b a)
    // Swap = 0x0c,   // (a b -- b a)
    // Rot = 0x0d,    // (a b c -- b c a)
    // And = 0x0e,    // ( a b -- a & b )	Bitwise AND
    // Or = 0x0f,     // ( a b -- a | b )	Bitwise OR
    // Xor = 0x10,    // ( a b -- a ^ b )	Bitwise exclusive OR
    // Not = 0x11,    // ( a -- ~a )	Bitwise complement
    // LShift = 0x12, // ( a n -- a << n )	Shift a left by n bits
    // RShift = 0x13, // ( a n -- a >> n )	Shift a right by n bits
    Jmp = 0x14, // (  -- )
    // Jt = 0x15  // ( -- )
    Halt = 0xFF, // end program
}

#[derive(Clone, Debug, PartialEq)]
pub enum AsmInstr<'src> {
    Push { value: i64, line: usize },
    Pop { line: usize },
    Add { line: usize },
    Sub { line: usize },
    Mul { line: usize },
    Div { line: usize },
    // Jump with label
    Jmp { label: &'src str, line: usize },
    Print { line: usize },
    Halt { line: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instr {
    Push(i64),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    // Jump offset as u32 so serialisation is consistent across platforms
    Jmp(InstructionIndex),
    Print,
    Halt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstructionIndex(u32);

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Push(value) => write!(f, "Push({value})"),
            Self::Pop => write!(f, "Pop"),
            Self::Add => write!(f, "Add"),
            Self::Sub => write!(f, "Sub"),
            Self::Mul => write!(f, "Mul"),
            Self::Div => write!(f, "Div"),
            Self::Jmp(offset) => write!(f, "Jump to {offset}"),
            Self::Print => write!(f, "Print"),
            Self::Halt => write!(f, "Halt"),
        }
    }
}

impl Instr {
    // Append this instruction's bytes.
    pub fn encode(&self, out: &mut Vec<u8>) -> Result<(), Error> {
        out.push(self.op() as u8);
        Ok(())
    }

    pub fn decode(code: &[u8], offset: usize) -> Result<(Self, usize), Error> {
        let bytes = code
            .get(offset..)
            .ok_or(Error::TruncatedByteCode { offset })?;
        let (&opcode, rest) = bytes
            .split_first()
            .ok_or(Error::TruncatedByteCode { offset })?;
        let op = BytecodeOp::from_repr(opcode).ok_or(Error::UnknownOpcode { opcode, offset })?;
        let (instr, consumed) = match op {
            BytecodeOp::Push => {
                let (operand, _) = rest
                    .split_first_chunk::<8>()
                    .ok_or(Error::TruncatedByteCode { offset })?;
                let value = i64::from_le_bytes(*operand);
                (Self::Push(value), 1 + size_of::<i64>())
            }
            BytecodeOp::Jmp => {
                todo!("Do later after refactoring scan and assemble stages")
            }
            BytecodeOp::Pop => (Self::Pop, 1),
            BytecodeOp::Add => (Self::Add, 1),
            BytecodeOp::Sub => (Self::Sub, 1),
            BytecodeOp::Mul => (Self::Mul, 1),
            BytecodeOp::Div => (Self::Div, 1),
            BytecodeOp::Print => (Self::Print, 1),
            BytecodeOp::Halt => (Self::Halt, 1),
        };
        Ok((instr, consumed))
    }

    fn op(&self) -> BytecodeOp {
        match self {
            Self::Push(_) => BytecodeOp::Push,
            Self::Pop => BytecodeOp::Pop,
            Self::Add => BytecodeOp::Add,
            Self::Sub => BytecodeOp::Sub,
            Self::Mul => BytecodeOp::Mul,
            Self::Div => BytecodeOp::Div,
            Self::Jmp(_) => BytecodeOp::Jmp,
            Self::Print => BytecodeOp::Print,
            Self::Halt => BytecodeOp::Halt,
        }
    }
}

impl fmt::Display for InstructionIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Program {
    code: Vec<u8>,
}

impl Program {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            code: bytes.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "not yet implemented: Do later after refactoring scan and assemble stages"
    )]
    fn encode_uses_little_endian_for_pushed_value_and_jmp_offset() {
        // Arrange expected
        let mut expected = vec![BytecodeOp::Push as u8];
        let value = 42_i64.to_le_bytes();
        expected.extend_from_slice(&value);
        expected.push(BytecodeOp::Jmp as u8);
        let label = 100_u32.to_le_bytes();
        expected.extend_from_slice(&label);
        // Arrange SUT
        let push = Instr::Push(42);

        let jump = todo!("Do later after refactoring scan and assemble stages");
        let mut out: Vec<u8> = Vec::new();
        // Act
        push.encode(&mut out);
        // TODO: jump
        assert_eq!(out, expected);
    }

    #[test]
    fn decode_returns_truncated_byte_error_when_bytes_are_truncated() {
        let code = &[0x01, 1, 0];
        let offset = 0;
        match Instr::decode(code, offset) {
            Err(Error::TruncatedByteCode {
                offset: offset_result,
            }) => {
                assert_eq!(offset_result, offset);
            }
            Err(err) => panic!("Truncated code returned wrong error: {err}"),
            Ok((instr, _)) => panic!("Truncated code decoded to {instr}"),
        }
    }

    #[test]
    fn decode_returns_unknown_opcode_when_opcode_bytes_not_recognised() {
        let code = &[0x01, 0x02, 0x79];
        let offset = 2;
        match Instr::decode(code, offset) {
            Err(Error::UnknownOpcode { opcode, offset }) => {
                assert_eq!(opcode, 0x79);
                assert_eq!(offset, 2);
            }
            Err(err) => panic!("Unknown opcode returned wrong error: {err}"),
            Ok((instr, _)) => panic!("Opcode 0x79 decoded to {instr}"),
        }
    }
}
