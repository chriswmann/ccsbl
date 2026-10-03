use crate::{bytecode::AsmInstr, errors::Error};

pub fn execute<'src>(instrs: &[AsmInstr]) -> Result<(), Error<'src>> {
    let mut stack = Vec::<i64>::new();
    let mut idx = 0;
    loop {
        let Some(instr) = instrs.get(idx) else {
            return Ok(());
        };
        match instr.clone() {
            AsmInstr::Add => {
                let a = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let b = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let c = a.checked_add(b).ok_or(Error::Overflow)?;
                stack.push(c);
            }
            AsmInstr::Sub => {
                let a = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let b = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let c = b.checked_sub(a).ok_or(Error::Overflow)?;
                stack.push(c);
            }
            AsmInstr::Mul => {
                let a = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let b = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let p = a.checked_mul(b).ok_or(Error::Overflow)?;
                stack.push(p);
            }
            AsmInstr::Div => {
                let a = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let b = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                let d = b.checked_div(a).ok_or(Error::DivideByZero)?;
                stack.push(d);
            }
            AsmInstr::Print => {
                let value = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
                println!("{value}");
            }
            AsmInstr::Pop => {
                let _ = stack.pop().ok_or(Error::StackUnderFlow {
                    instr: instr.clone(),
                })?;
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
