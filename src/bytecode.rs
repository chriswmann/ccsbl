use std::{
    fmt, fs,
    io::Write,
    path::{self, PathBuf},
};

use strum::FromRepr;
use tempfile::NamedTempFile;

use crate::errors::Error;

const MAGIC_BYTES: &[u8] = b"CCSBL";
const FILE_FORMAT_VERSION: u8 = 1;

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
pub enum Instr<'src> {
    Push { value: i64, line: usize },
    Pop { line: usize },
    Add { line: usize },
    Sub { line: usize },
    Mul { line: usize },
    Div { line: usize },
    Jmp { target: &'src str, line: usize },
    Print { line: usize },
    Halt { line: usize },
}

// Bytecode instructions
#[derive(Clone, Debug, PartialEq)]
pub enum AsmInstr {
    Push(i64),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Jmp(InstructionIndex),
    Print,
    Halt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstructionIndex(pub usize);

impl From<usize> for InstructionIndex {
    fn from(d: usize) -> Self {
        Self(d)
    }
}

impl fmt::Display for AsmInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl AsmInstr {
    // Append this instruction's bytes.
    pub fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.op() as u8);
        match self {
            AsmInstr::Push(value) => {
                out.extend_from_slice(&value.to_le_bytes());
            }
            // Represent the instruction index as u64 for cross-platform compatibility
            AsmInstr::Jmp(ind) => out.extend_from_slice(&(ind.0 as u64).to_le_bytes()),
            AsmInstr::Pop
            | AsmInstr::Add
            | AsmInstr::Sub
            | AsmInstr::Mul
            | AsmInstr::Div
            | AsmInstr::Print
            | AsmInstr::Halt => {}
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
                (Self::Push(value), Self::Push(value).size())
            }
            BytecodeOp::Jmp => {
                let (operand, _) = rest
                    .split_first_chunk::<8>()
                    .ok_or(Error::TruncatedByteCode { offset })?;
                let target = u64::from_le_bytes(*operand);
                let idx = usize::try_from(target)
                    .map_err(|_| Error::JumpTargetOutOfRange { target, offset })?;
                (Self::Jmp(idx.into()), Self::Jmp(idx.into()).size())
            }
            BytecodeOp::Pop => (Self::Pop, Self::Pop.size()),
            BytecodeOp::Add => (Self::Add, Self::Add.size()),
            BytecodeOp::Sub => (Self::Sub, Self::Sub.size()),
            BytecodeOp::Mul => (Self::Mul, Self::Mul.size()),
            BytecodeOp::Div => (Self::Div, Self::Div.size()),
            BytecodeOp::Print => (Self::Print, Self::Print.size()),
            BytecodeOp::Halt => (Self::Halt, Self::Halt.size()),
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

    fn size(&self) -> usize {
        match self {
            Self::Pop
            | Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::Print
            | Self::Halt => 1,
            Self::Push(_) => 1 + size_of::<i64>(),
            Self::Jmp(_) => 1 + size_of::<u64>(),
        }
    }
}

impl fmt::Display for InstructionIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// `Program` is the bytecode with a header
// that defines the encoding version.
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

    // Atomic save and sync the program to disk
    pub fn save<'src>(&self, path: &path::Path) -> Result<(), Error<'src>> {
        let dir = path
            .parent()
            .filter(|&p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| path::Path::new("."));

        let mut temp = NamedTempFile::new_in(dir).map_err(|e| Error::Io { source: e })?;

        let data = self.format_file();
        temp.write_all(&data).map_err(|e| Error::Io { source: e })?;

        temp.as_file()
            .sync_all()
            .map_err(|e| Error::Io { source: e })?;

        let _file = temp.persist(path).map_err(|err| Error::File {
            path: path.to_path_buf(),
            source: err.error,
        })?;

        fs::File::open(dir)
            .map_err(|err| Error::Io { source: err })?
            .sync_all()
            .map_err(|err| Error::Io { source: err })?;
        Ok(())
    }

    fn format_file(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(&self.code.len() + 5 + 1);
        buf.extend_from_slice(MAGIC_BYTES);
        buf.extend_from_slice(&[FILE_FORMAT_VERSION]);
        buf.extend_from_slice(&self.code);
        buf
    }

    pub fn load<'src>(path: &path::Path) -> Result<Self, Error<'src>> {
        let bytes = fs::read(path).map_err(|err| Error::Io { source: err })?;
        let (magic_bytes, rest) =
            bytes
                .split_at_checked(MAGIC_BYTES.len())
                .ok_or(Error::UnknownFileFormat {
                    path: path.to_path_buf(),
                })?;
        if !matches!(magic_bytes, MAGIC_BYTES) {
            return Err(Error::UnknownFileFormat {
                path: path.to_path_buf(),
            });
        }
        let (format_version, bytecode) =
            rest.split_at_checked(1).ok_or(Error::UnknownFileFormat {
                path: path.to_path_buf(),
            })?;
        let format_version = format_version.first().ok_or(Error::UnknownFileFormat {
            path: path.to_path_buf(),
        })?;
        if *format_version != FILE_FORMAT_VERSION {
            return Err(Error::UnknownFileFormat {
                path: path.to_path_buf(),
            });
        }
        Ok(Self::from_bytes(bytecode))
    }

    pub fn decode<'src>(&'src self) -> Result<Vec<AsmInstr>, Error<'src>> {
        let mut offset = 0;
        let mut asm = Vec::new();
        loop {
            if offset >= self.code.len() {
                return Ok(asm);
            }
            let (instr, consumed) = AsmInstr::decode(&self.code, offset)?;
            asm.push(instr);
            offset += consumed;
        }
    }
}

#[cfg(test)]
mod tests {

    use tempfile::{self, tempdir};

    use super::*;

    #[test]
    fn encode_operandless_instructions() {
        let instrs = vec![
            AsmInstr::Pop,
            AsmInstr::Print,
            AsmInstr::Add,
            AsmInstr::Sub,
            AsmInstr::Mul,
            AsmInstr::Div,
            AsmInstr::Halt,
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
        let push = AsmInstr::Push(-1_155_356_143);
        let expected = &[0x01, 0x11, 0xAA, 0x22, 0xBB, 0xFF, 0xFF, 0xFF, 0xFF];
        let mut result = Vec::new();
        push.encode(&mut result);
        assert_eq!(result, expected);
    }

    #[test]
    fn encode_push_boundary_values() {
        let mut result = Vec::new();
        let push = AsmInstr::Push(0);
        push.encode(&mut result);
        assert_eq!(result, &[1, 0, 0, 0, 0, 0, 0, 0, 0]);

        result.clear();
        let push = AsmInstr::Push(-1);
        push.encode(&mut result);
        assert_eq!(result, &[1, 255, 255, 255, 255, 255, 255, 255, 255]);

        result.clear();
        let push = AsmInstr::Push(i64::MIN);
        push.encode(&mut result);
        assert_eq!(result, &[1, 0, 0, 0, 0, 0, 0, 0, 128]);

        result.clear();
        let push = AsmInstr::Push(i64::MAX);
        push.encode(&mut result);
        assert_eq!(result, &[1, 255, 255, 255, 255, 255, 255, 255, 127]);
    }

    #[test]
    fn encode_jump_little_endian() {
        let mut result = Vec::new();
        let jump = AsmInstr::Jmp(24.into());
        jump.encode(&mut result);
        assert_eq!(result, &[20, 24, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn encode_preserves_existing_bytes() {
        let mut result = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 20, 47, 0, 0, 0];
        let print = AsmInstr::Print;
        print.encode(&mut result);
        assert_eq!(result, &[1, 0, 0, 0, 0, 0, 0, 0, 0, 20, 47, 0, 0, 0, 3]);
    }

    #[test]
    fn encode_multiple_instructions_in_order() {
        let mut result = Vec::new();
        let instrs = vec![
            AsmInstr::Push(0),
            AsmInstr::Push(2),
            AsmInstr::Push(-100),
            AsmInstr::Print,
            AsmInstr::Jmp(InstructionIndex(80)),
            AsmInstr::Pop,
        ];
        for instr in instrs {
            instr.encode(&mut result);
        }

        let expected = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 156, 255, 255, 255, 255, 255,
            255, 255, 3, 20, 80, 0, 0, 0, 0, 0, 0, 0, 2,
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
            20, 41, 0, 0, 0, 0, 0, 0, 0,   // Jmp(41)
            255, // Halt
        ];
        match AsmInstr::decode(&bytecode, 0) {
            Ok(instr) => assert_eq!(instr, (AsmInstr::Push(0), 9)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match AsmInstr::decode(&bytecode, 9) {
            Ok(instr) => assert_eq!(instr, (AsmInstr::Push(-1), 9)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match AsmInstr::decode(&bytecode, 18) {
            Ok(instr) => assert_eq!(instr, (AsmInstr::Pop, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match AsmInstr::decode(&bytecode, 19) {
            Ok(instr) => assert_eq!(instr, (AsmInstr::Print, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match AsmInstr::decode(&bytecode, 20) {
            Ok(instr) => assert_eq!(instr, (AsmInstr::Add, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match AsmInstr::decode(&bytecode, 21) {
            Ok(instr) => assert_eq!(instr, (AsmInstr::Jmp(41.into()), 9)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
        match AsmInstr::decode(&bytecode, 30) {
            Ok(instr) => assert_eq!(instr, (AsmInstr::Halt, 1)),
            Err(err) => panic!("Instr::decode failed with error {err}"),
        }
    }

    #[test]
    fn decode_returns_truncated_byte_error_when_bytes_are_truncated() {
        let code = &[0x01, 1, 0];
        let offset = 0;
        match AsmInstr::decode(code, offset) {
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
        match AsmInstr::decode(code, offset) {
            Err(Error::UnknownOpcode { opcode, offset }) => {
                assert_eq!(opcode, 0x79);
                assert_eq!(offset, 2);
            }
            Err(err) => panic!("Unknown opcode returned wrong error: {err}"),
            Ok((instr, _)) => panic!("Opcode 0x79 decoded to {instr}"),
        }
    }

    #[test]
    fn file_format_has_magic_bytes_and_expected_version() {
        let program = Program::from_bytes(&[0]);
        let result = program.format_file();
        assert_eq!(result, vec![67, 67, 83, 66, 76, 1, 0]);
    }

    #[test]
    fn loads_bytecode_correctly() {
        let dir = tempdir().expect("should be able to create test directory");
        let mut file = NamedTempFile::new_in(&dir)
            .expect("should be able to create temp file in test directory");
        let bytecode = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, // Push(0)
            1, 255, 255, 255, 255, 255, 255, 255, 255, // Push(-1)
            2,   // Pop
            3,   // Print
            4,   // Add
            20, 41, 0, 0, 0, 0, 0, 0, 0,   // Jmp(41)
            255, // Halt
        ];
        let program = Program::from_bytes(&bytecode);
        let data = program.format_file();
        file.write_all(&data)
            .expect("should be able to write test code to temp file");
        let loaded = Program::load(file.path()).expect("should be able to load test program");
        assert_eq!(loaded.code, bytecode);
    }
}
