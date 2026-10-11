use std::{fs, io::Write, path};

use tempfile::NamedTempFile;

use crate::{
    bytecode::AsmInstr,
    error::{BytecodeError, FileError},
};

const MAGIC_BYTES: &[u8] = b"CCSBL";
const FILE_FORMAT_VERSION: u8 = 1;

// `Program` is the bytecode with a header
// that defines the encoding version.
#[derive(Debug)]
pub struct Program {
    header: Vec<u8>,
    code: Vec<u8>,
}

impl Program {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut header = Vec::new();
        header.extend_from_slice(MAGIC_BYTES);
        header.push(FILE_FORMAT_VERSION);
        Self {
            header,
            code: bytes.to_vec(),
        }
    }

    // Atomic save and sync the program to disk
    pub fn save<'src>(&self, path: &path::Path) -> Result<(), FileError> {
        let dir = path
            .parent()
            .filter(|&p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| path::Path::new("."));

        let mut temp = NamedTempFile::new_in(dir).map_err(|e| FileError::Io { source: e })?;

        let data = self.format_file();
        temp.write_all(&data)
            .map_err(|e| FileError::Io { source: e })?;

        temp.as_file()
            .sync_all()
            .map_err(|e| FileError::Io { source: e })?;

        let _file = temp.persist(path).map_err(|err| FileError::File {
            path: path.to_path_buf(),
            source: err.error,
        })?;

        fs::File::open(dir)
            .map_err(|err| FileError::Io { source: err })?
            .sync_all()
            .map_err(|err| FileError::Io { source: err })?;
        Ok(())
    }

    fn format_file(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(&self.code.len() + 5 + 1);
        buf.extend_from_slice(&self.header);
        buf.extend_from_slice(&self.code);
        buf
    }

    pub fn load<'src>(path: &path::Path) -> Result<Self, FileError> {
        let bytes = fs::read(path).map_err(|err| FileError::Io { source: err })?;
        let (magic_bytes, rest) =
            bytes
                .split_at_checked(MAGIC_BYTES.len())
                .ok_or(FileError::UnknownFileFormat {
                    path: path.to_path_buf(),
                })?;
        if !matches!(magic_bytes, MAGIC_BYTES) {
            return Err(FileError::UnknownFileFormat {
                path: path.to_path_buf(),
            });
        }
        let (format_version, bytecode) =
            rest.split_at_checked(1)
                .ok_or(FileError::UnknownFileFormat {
                    path: path.to_path_buf(),
                })?;
        let format_version = format_version.first().ok_or(FileError::UnknownFileFormat {
            path: path.to_path_buf(),
        })?;
        if *format_version != FILE_FORMAT_VERSION {
            return Err(FileError::UnknownFileFormat {
                path: path.to_path_buf(),
            });
        }
        Ok(Self::from_bytes(bytecode))
    }

    fn encode(asm: &[AsmInstr]) -> Self {
        let mut buf = Vec::with_capacity(asm.len() + 50);
        for instr in asm {
            instr.encode(&mut buf);
        }
        Self::from_bytes(&buf)
    }

    pub fn decode(&self) -> Result<Vec<AsmInstr>, BytecodeError> {
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
    use tempfile::tempdir;

    use super::*;

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
