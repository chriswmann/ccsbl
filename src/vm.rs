use std::fmt;
use std::io::Write;

use tracing::debug;

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

    fn pop(&mut self) -> Result<i64, RuntimeError> {
        match self.0.pop() {
            Some(value) => Ok(value),
            None => Err(RuntimeError::StackUnderFlow),
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
        match instr.clone() {
            AsmInstr::Add => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                let c = a.checked_add(b).ok_or(RuntimeError::Overflow)?;
                stack.push(c);
            }
            AsmInstr::Sub => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                let c = b.checked_sub(a).ok_or(RuntimeError::Overflow)?;
                stack.push(c);
            }
            AsmInstr::Mul => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                let p = a.checked_mul(b).ok_or(RuntimeError::Overflow)?;
                stack.push(p);
            }
            AsmInstr::Div => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                if a == 0 {
                    return Err(RuntimeError::DivideByZero);
                }
                let d = b.checked_div(a).ok_or(RuntimeError::Overflow)?;
                stack.push(d);
            }
            AsmInstr::Neg => {
                let a = stack.pop()?;
                let n = a.checked_neg().ok_or(RuntimeError::Overflow)?;
                stack.push(n);
            }
            AsmInstr::Mod => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                if a == 0 {
                    return Err(RuntimeError::DivideByZero);
                }
                let n = b.checked_rem(a).ok_or(RuntimeError::Overflow)?;
                stack.push(n);
            }
            AsmInstr::Print => {
                let value = stack.pop()?;
                print(value, w)?;
            }
            AsmInstr::Pop => {
                stack.pop()?;
            }
            AsmInstr::Push(value) => {
                stack.push(value);
            }
            AsmInstr::Jmp(instr_index) => {
                idx = instr_index.0;
                continue;
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
        let asm = vec![AsmInstr::Push(0), AsmInstr::Pop, AsmInstr::Pop];
        let result = execute(&asm, &mut Vec::new());
        assert!(
            matches!(&result, Err(RuntimeError::StackUnderFlow),),
            "expected StackUnderFlow but got {result:?}",
        );
        let asm = vec![AsmInstr::Push(0), AsmInstr::Pop, AsmInstr::Pop];
        let result = execute(&asm, &mut Vec::new());
        assert!(
            matches!(&result, Err(RuntimeError::StackUnderFlow),),
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
}
