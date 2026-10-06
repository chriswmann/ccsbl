use std::fmt;

use tracing::debug;

use crate::{bytecode::AsmInstr, errors::Error};

#[derive(Default)]
struct Stack(Vec<i64>);

impl Stack {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, value: i64) {
        self.0.push(value);
    }

    fn pop(&mut self) -> Result<i64, Error<'static>> {
        match self.0.pop() {
            Some(value) => Ok(value),
            None => Err(Error::StackUnderFlow),
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

pub fn execute<'src>(asm: &[AsmInstr]) -> Result<(), Error<'src>> {
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
                let c = a.checked_add(b).ok_or(Error::Overflow)?;
                stack.push(c);
            }
            AsmInstr::Sub => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                let c = b.checked_sub(a).ok_or(Error::Overflow)?;
                stack.push(c);
            }
            AsmInstr::Mul => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                let p = a.checked_mul(b).ok_or(Error::Overflow)?;
                stack.push(p);
            }
            AsmInstr::Div => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                if a == 0 {
                    return Err(Error::DivideByZero);
                }
                let d = b.checked_div(a).ok_or(Error::Overflow)?;
                stack.push(d);
            }
            AsmInstr::Print => {
                let value = stack.pop()?;
                println!("{value}");
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
        let result = execute(&asm);
        assert!(
            matches!(&result, Err(Error::StackUnderFlow),),
            "expected StackUnderFlow but got {result:?}",
        );
        let asm = vec![AsmInstr::Push(0), AsmInstr::Pop, AsmInstr::Pop];
        let result = execute(&asm);
        assert!(
            matches!(&result, Err(Error::StackUnderFlow),),
            "expected StackUnderFlow but got {result:?}",
        );
    }
}
