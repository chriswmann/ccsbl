use std::fmt;
use std::io::Write;

use tracing::debug;

use crate::bytecode::InstructionIndex;
use crate::error::{BadJumpTarget, JumpType};
use crate::ui::print;
use crate::{bytecode::AsmInstr, error::RuntimeError};

#[derive(Default)]
struct Stack(Vec<i64>);

impl Stack {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, value: i64) {
        self.0.push(value);
    }

    fn pop(&mut self, instr: AsmInstr, ind: InstructionIndex) -> Result<i64, RuntimeError> {
        match self.0.pop() {
            Some(value) => Ok(value),
            None => Err(RuntimeError::StackUnderFlow { instr, ind }),
        }
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, value) in self.0.iter().enumerate() {
            let _ = writeln!(f, "\n    {i:6} {value:12}");
        }
        Ok(())
    }
}

pub fn execute<W: Write>(asm: &[AsmInstr], w: &mut W) -> Result<(), RuntimeError> {
    let mut stack = Stack::new();
    let mut idx = 0;
    loop {
        debug!("stack: {}", stack.to_string());
        let Some(instr) = asm.get(idx) else {
            return Ok(());
        };
        match instr {
            &instr @ AsmInstr::Add => {
                let a = stack.pop(instr, idx.into())?;
                let b = stack.pop(instr, idx.into())?;
                let c = a.checked_add(b).ok_or(RuntimeError::StackOverflow {
                    instr,
                    ind: idx.into(),
                })?;
                stack.push(c);
            }
            &instr @ AsmInstr::Sub => {
                let a = stack.pop(instr, idx.into())?;
                let b = stack.pop(instr, idx.into())?;
                let c = b.checked_sub(a).ok_or(RuntimeError::StackOverflow {
                    instr,
                    ind: idx.into(),
                })?;
                stack.push(c);
            }
            &instr @ AsmInstr::Mul => {
                let a = stack.pop(instr, idx.into())?;
                let b = stack.pop(instr, idx.into())?;
                let p = a.checked_mul(b).ok_or(RuntimeError::StackOverflow {
                    instr,
                    ind: idx.into(),
                })?;
                stack.push(p);
            }
            &instr @ AsmInstr::Div => {
                let a = stack.pop(instr, idx.into())?;
                let b = stack.pop(instr, idx.into())?;
                if a == 0 {
                    return Err(RuntimeError::DivideByZero {
                        instr,
                        ind: idx.into(),
                    });
                }
                let d = b.checked_div(a).ok_or(RuntimeError::StackOverflow {
                    instr,
                    ind: idx.into(),
                })?;
                stack.push(d);
            }
            &instr @ AsmInstr::Neg => {
                let a = stack.pop(instr, idx.into())?;
                let n = a.checked_neg().ok_or(RuntimeError::StackOverflow {
                    instr,
                    ind: idx.into(),
                })?;
                stack.push(n);
            }
            &instr @ AsmInstr::Mod => {
                let a = stack.pop(instr, idx.into())?;
                let b = stack.pop(instr, idx.into())?;
                if a == 0 {
                    return Err(RuntimeError::DivideByZero {
                        instr,
                        ind: idx.into(),
                    });
                }
                let n = b.checked_rem(a).ok_or(RuntimeError::StackOverflow {
                    instr,
                    ind: idx.into(),
                })?;
                stack.push(n);
            }
            &instr @ AsmInstr::Dup => {
                let a = stack.pop(instr, idx.into())?;
                let b = a;
                stack.push(a);
                stack.push(b);
            }
            &instr @ AsmInstr::Print => {
                let value = stack.pop(instr, idx.into())?;
                print(value, w)?;
            }
            &instr @ AsmInstr::Pop => {
                stack.pop(instr, idx.into())?;
            }
            AsmInstr::Push(value) => {
                stack.push(*value);
            }
            AsmInstr::Jmp(instr_index) => {
                if instr_index.0 > asm.len() {
                    return Err(RuntimeError::JumpTargetOutOfProgram(BadJumpTarget::from(
                        JumpType::Jmp,
                        instr_index.0.into(),
                        idx.into(),
                    )));
                }
                idx = instr_index.0;
                continue;
            }
            &instr @ AsmInstr::Jlt(instr_index) => {
                if instr_index.0 > asm.len() {
                    return Err(RuntimeError::JumpTargetOutOfProgram(BadJumpTarget::from(
                        JumpType::Jmp,
                        instr_index.0.into(),
                        idx.into(),
                    )));
                }
                let a = stack.pop(instr, idx.into())?;
                let b = stack.pop(instr, idx.into())?;
                if b < a {
                    idx = instr_index.0;
                    continue;
                }
            }
            AsmInstr::Halt => {
                return Ok(());
            }
        }
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_reports_stack_underflow_when_stack_is_exhausted() {
        let asm = vec![AsmInstr::Pop];
        let result = execute(&asm, &mut Vec::new());
        assert!(
            matches!(&result, Err(RuntimeError::StackUnderFlow { instr, ind })
            if *instr == AsmInstr::Pop && *ind == InstructionIndex::from(0)
            ),
            "expected StackUnderFlow but got {result:?}",
        );
        let asm = vec![AsmInstr::Push(0), AsmInstr::Pop, AsmInstr::Pop];
        let result = execute(&asm, &mut Vec::new());
        assert!(
            matches!(&result, Err(RuntimeError::StackUnderFlow { instr, ind })
            if *instr == AsmInstr::Pop && *ind == InstructionIndex::from(2)
            ),
            "expected StackUnderFlow but got {result:?}",
        );
    }

    #[test]
    fn execute_prints_values_with_new_line() {
        let asm = vec![
            AsmInstr::Push(10),
            AsmInstr::Push(20),
            AsmInstr::Print,
            AsmInstr::Print,
            AsmInstr::Push(23),
            AsmInstr::Print,
        ];
        let mut buf = Vec::new();
        let expected = b"20\n10\n23\n";
        execute(&asm, &mut buf).unwrap();
        assert_eq!(buf, expected);
    }

    #[test]
    fn mod_truncates_towards_zero() {
        // a = (a / b) * b + (a % b)
        let a = -5;
        let b = 4;
        let asm = vec![
            AsmInstr::Push(a),
            AsmInstr::Push(b),
            AsmInstr::Div,
            AsmInstr::Push(b),
            AsmInstr::Mul,
            AsmInstr::Push(a),
            AsmInstr::Push(b),
            AsmInstr::Mod,
            AsmInstr::Add,
            AsmInstr::Print,
        ];
        let mut buf = Vec::new();
        let expected = b"-5\n";
        execute(&asm, &mut buf).unwrap();
        assert_eq!(buf, expected);
    }

    #[test]
    fn divide_by_zero_reports_error() {
        let asm = vec![AsmInstr::Push(1), AsmInstr::Push(0), AsmInstr::Div];
        let mut buf = Vec::new();
        match execute(&asm, &mut buf) {
            Ok(()) => {
                panic!(
                    "expected divide by zero to return RuntimeError::DivideByZero but got Ok(())"
                )
            }
            Err(RuntimeError::DivideByZero { instr, ind }) => {
                assert_eq!(instr, AsmInstr::Div);
                assert_eq!(ind, InstructionIndex::from(2));
            }
            Err(err) => {
                panic!("expected divide by zero to return RuntimeError::DivideByZero but got {err}")
            }
        }
    }
    #[test]
    fn mod_by_zero_reports_error() {
        let asm = vec![AsmInstr::Push(1), AsmInstr::Push(0), AsmInstr::Mod];
        let mut buf = Vec::new();
        match execute(&asm, &mut buf) {
            Ok(()) => {
                panic!("expected mod by zero to return RuntimeError::DivideByZero but got Ok(())")
            }
            Err(RuntimeError::DivideByZero { instr, ind }) => {
                assert_eq!(instr, AsmInstr::Mod);
                assert_eq!(ind, InstructionIndex::from(2));
            }
            Err(err) => {
                panic!("expected mod by zero to return RuntimeError::DivideByZero but got {err}")
            }
        }
    }
}
