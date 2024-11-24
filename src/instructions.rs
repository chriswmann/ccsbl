use crate::ops::Op;

#[derive(Debug, Clone, Copy)]
pub struct Instr {
    pub op: Op,
    pub value: i64,
}
