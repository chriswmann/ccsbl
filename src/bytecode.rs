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

// Source code instructions
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

// Bytecode instructions
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

impl From<u32> for InstructionIndex {
    fn from(d: u32) -> Self {
        Self(d)
    }
}

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
    pub fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.op() as u8);
        match self {
            Instr::Push(value) => {
                out.extend_from_slice(&value.to_le_bytes());
            }
            Instr::Jmp(ind) => out.extend_from_slice(&ind.0.to_le_bytes()),
            Instr::Pop
            | Instr::Add
            | Instr::Sub
            | Instr::Mul
            | Instr::Div
            | Instr::Print
            | Instr::Halt => {}
        }
    }

    // Decode a bytecode instruction at the given offset, returning the `Instr`
    // and the number of bytes it consumed.
    pub fn decode(code: &[u8], offset: usize) -> Result<(Self, usize), Error<'_>> {
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
                let (operand, _) = rest
                    .split_first_chunk::<4>()
                    .ok_or(Error::TruncatedByteCode { offset })?;
                let idx = u32::from_le_bytes(*operand);
                (Self::Jmp(idx.into()), 1 + size_of::<u32>())
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
    fn encode_operandless_instructions() {
        let instrs = vec![
            Instr::Pop,
            Instr::Print,
            Instr::Add,
            Instr::Sub,
            Instr::Mul,
            Instr::Div,
            Instr::Halt,
        ];
        let expected: Vec<u8> = vec![2, 3, 4, 5, 6, 7, 255];
        let mut result = Vec::new();
        for instr in instrs {
            instr.encode(&mut result);
        }
        assert_eq!(result, expected);
    }

    #[test]
    fn encode_push_little_endian() {
        let push = Instr::Push(-1_155_356_143);
        let expected = &[0x01, 0x11, 0xAA, 0x22, 0xBB, 0xFF, 0xFF, 0xFF, 0xFF];
        let mut result = Vec::new();
        push.encode(&mut result);
        assert_eq!(result, expected);
    }

    #[test]
    fn encode_push_boundary_values() {
        let mut result = Vec::new();
        let push = Instr::Push(0);
        push.encode(&mut result);
        assert_eq!(result, &[1, 0, 0, 0, 0, 0, 0, 0, 0]);

        result.clear();
        let push = Instr::Push(-1);
        push.encode(&mut result);
        assert_eq!(result, &[1, 255, 255, 255, 255, 255, 255, 255, 255]);

        result.clear();
        let push = Instr::Push(i64::MIN);
        push.encode(&mut result);
        assert_eq!(result, &[1, 0, 0, 0, 0, 0, 0, 0, 128]);

        result.clear();
        let push = Instr::Push(i64::MAX);
        push.encode(&mut result);
        assert_eq!(result, &[1, 255, 255, 255, 255, 255, 255, 255, 127]);
    }

    #[test]
    fn encode_jump_little_endian() {
        let mut result = Vec::new();
        let jump = Instr::Jmp(24.into());
        jump.encode(&mut result);
        assert_eq!(result, &[20, 24, 0, 0, 0]);
    }

    #[test]
    fn encode_preserves_existing_bytes() {
        let mut result = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 20, 47, 0, 0, 0];
        let print = Instr::Print;
        print.encode(&mut result);
        assert_eq!(result, &[1, 0, 0, 0, 0, 0, 0, 0, 0, 20, 47, 0, 0, 0, 3]);
    }

    #[test]
    fn encode_multiple_instructions_in_order() {
        let mut result = Vec::new();
        let instrs = vec![
            Instr::Push(0),
            Instr::Push(2),
            Instr::Push(-100),
            Instr::Print,
            Instr::Jmp(InstructionIndex(80)),
            Instr::Pop,
        ];
        for instr in instrs {
            instr.encode(&mut result);
        }

        let expected = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 156, 255, 255, 255, 255, 255,
            255, 255, 3, 20, 80, 0, 0, 0, 2,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn decode_multiple_instructions() {
        let bytecode = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, // Push(0)
            1, 255, 255, 255, 255, 255, 255, 255, 255, // Push(-1)
            2,   // Pop
            3,   // Print
            4,   // Add
            20, 41, 0, 0, 0,   // Jmp(41)
            255, // Halt
        ];
        match Instr::decode(&bytecode, 0) {
            Ok(instr) => assert_eq!(instr, (Instr::Push(0), 9)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match Instr::decode(&bytecode, 9) {
            Ok(instr) => assert_eq!(instr, (Instr::Push(-1), 9)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match Instr::decode(&bytecode, 18) {
            Ok(instr) => assert_eq!(instr, (Instr::Pop, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match Instr::decode(&bytecode, 19) {
            Ok(instr) => assert_eq!(instr, (Instr::Print, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match Instr::decode(&bytecode, 20) {
            Ok(instr) => assert_eq!(instr, (Instr::Add, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match Instr::decode(&bytecode, 21) {
            Ok(instr) => assert_eq!(instr, (Instr::Jmp(41.into()), 5)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match Instr::decode(&bytecode, 26) {
            Ok(instr) => assert_eq!(instr, (Instr::Halt, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
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
