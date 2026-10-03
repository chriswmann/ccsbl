use std::collections::HashMap;

use crate::{
    bytecode::{
        AsmInstr::{self, Jmp},
        Instr, InstructionIndex,
    },
    errors::Error,
    token::{Op, Span, Token},
};

#[derive(Debug, Default)]
pub struct Assembled<'src> {
    pub instrs: Vec<Instr<'src>>,
    pub labels: HashMap<&'src str, InstructionIndex>,
}

// Not worrying about stack underflow or overflow yet
// Jumps are resolved in `resolve` rather than backpatching.
pub fn assemble<'src>(spans: &[Span<'src>]) -> Result<Assembled<'src>, Vec<Error<'src>>> {
    if spans.is_empty() {
        return Ok(Assembled::default());
    }
    let mut instrs = Vec::with_capacity(spans.len());
    let mut labels = HashMap::new();
    let mut errors = Vec::new();
    let mut idx = 0;
    loop {
        if idx == spans.len() {
            if errors.is_empty() {
                return Ok(Assembled { instrs, labels });
            }
            return Err(errors);
        }
        let Span { token, line } = spans[idx].clone();
        match token {
            Token::Op(op) => match op {
                Op::Pop => instrs.push(Instr::Pop { line }),
                Op::Print => instrs.push(Instr::Print { line }),
                Op::Add => instrs.push(Instr::Add { line }),
                Op::Sub => instrs.push(Instr::Sub { line }),
                Op::Mul => instrs.push(Instr::Mul { line }),
                Op::Div => instrs.push(Instr::Div { line }),
                Op::Jmp => {
                    if (idx + 1) < spans.len() {
                        let Span {
                            token: next_token,
                            line: next_line,
                        } = spans[idx + 1].clone();
                        match next_token {
                            Token::Ident(ident) => {
                                instrs.push(Instr::Jmp {
                                    target: ident,
                                    line,
                                });
                                idx += 1;
                            }
                            Token::Op(_) | Token::Value(_) | Token::Label(_) => {
                                errors.push(Error::NotAJumpLabel {
                                    token: next_token,
                                    line: next_line,
                                });
                            }
                        }
                    } else {
                        errors.push(Error::MissingJumpLabel { line });
                    }
                }
                Op::Halt => {
                    instrs.push(Instr::Halt { line });
                }
            },
            Token::Value(value) => instrs.push(Instr::Push { value, line }),
            Token::Label(label) => {
                if labels.insert(label, instrs.len().into()).is_some() {
                    errors.push(Error::NotUniqueLabel { label, line });
                }
            }
            // Idents aren't implemented yet
            token @ Token::Ident(_ident) => errors.push(Error::Token { token, line }),
        }
        idx += 1;
    }
}

fn resolve<'src>(
    instrs: &[Instr<'src>],
    labels: &HashMap<&'src str, InstructionIndex>,
) -> Result<Vec<AsmInstr>, Vec<Error<'src>>> {
    let mut asm = Vec::new();
    let mut errors = Vec::new();
    for instr in instrs {
        match instr {
            Instr::Push { value, .. } => asm.push(AsmInstr::Push(*value)),
            Instr::Pop { .. } => asm.push(AsmInstr::Pop),
            Instr::Add { .. } => asm.push(AsmInstr::Add),
            Instr::Sub { .. } => asm.push(AsmInstr::Sub),
            Instr::Mul { .. } => asm.push(AsmInstr::Mul),
            Instr::Div { .. } => asm.push(AsmInstr::Div),
            Instr::Print { .. } => asm.push(AsmInstr::Print),
            Instr::Jmp { target, line } => match labels.get(target) {
                Some(i) => asm.push(Jmp(*i)),
                None => errors.push(Error::ResolveJumpTarget {
                    target,
                    line: *line,
                }),
            },
            Instr::Halt { line: _ } => asm.push(AsmInstr::Halt),
        }
    }
    if errors.is_empty() {
        return Ok(asm);
    }
    Err(errors)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn empty_spans_returns_empty_instrs() {
        let spans = Vec::new();
        let Assembled { instrs, labels } = assemble(&spans).unwrap();
        assert_eq!(labels, HashMap::<&str, InstructionIndex>::new());
        assert_eq!(instrs, Vec::<Instr>::new());
    }

    #[test]
    fn returns_expected_output_for_given_input() {
        let spans = vec![
            Span {
                token: Token::Value(0),
                line: 2,
            },
            Span {
                token: Token::Value(-2_000_000_000),
                line: 2,
            },
            Span {
                token: Token::Op(Op::Pop),
                line: 2,
            },
            Span {
                token: Token::Op(Op::Jmp),
                line: 3,
            },
            Span {
                token: Token::Ident("target"),
                line: 3,
            },
            Span {
                token: Token::Op(Op::Print),
                line: 3,
            },
            Span {
                token: Token::Op(Op::Add),
                line: 4,
            },
            Span {
                token: Token::Op(Op::Halt),
                line: 4,
            },
            Span {
                token: Token::Value(0),
                line: 5,
            },
        ];

        let expected_instrs = vec![
            Instr::Push { value: 0, line: 2 },
            Instr::Push {
                value: -2_000_000_000,
                line: 2,
            },
            Instr::Pop { line: 2 },
            Instr::Jmp {
                target: "target",
                line: 3,
            },
            Instr::Print { line: 3 },
            Instr::Add { line: 4 },
            Instr::Halt { line: 4 },
            Instr::Push { value: 0, line: 5 },
        ];
        let result = assemble(&spans).unwrap();
        assert_eq!(result.instrs, expected_instrs);
        assert_eq!(result.labels, HashMap::<&str, InstructionIndex>::new());
    }

    #[test]
    fn returns_expected_report_for_invalid_jump_operand() {
        let spans = vec![
            Span {
                token: Token::Op(Op::Jmp),
                line: 0,
            },
            Span {
                token: Token::Op(Op::Pop),
                line: 1,
            },
        ];
        match assemble(&spans) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert!(matches!(
                        &errors[0],
                        Error::NotAJumpLabel { token, line }
                        if *token == Token::Op(Op::Pop) && *line == 1
                ));
            }
            Ok(assembled) => panic!("Expected error for invalid jump operand, got {assembled:?}"),
        }
    }

    #[test]
    fn returns_expected_report_for_missing_jump_operand() {
        let spans = vec![
            Span {
                token: Token::Op(Op::Jmp),
                line: 0,
            },
            Span {
                token: Token::Op(Op::Halt),
                line: 1,
            },
        ];
        match assemble(&spans) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert!(
                    matches!(
                    &errors[0],
                       Error::NotAJumpLabel { token, line } if *token == Token::Op(Op::Halt) && *line == 1
                    ),
                    "Jmp then halt result was {errors:?}",
                );
            }
            Ok(_) => panic!("Expected error for missing jump operand"),
        }

        let spans = vec![Span {
            token: Token::Op(Op::Jmp),
            line: 100,
        }];
        match assemble(&spans) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert!(
                    matches!(
                    &errors[0],
                        Error::MissingJumpLabel { line } if *line == 100
                    ),
                    "Result was {errors:?}",
                );
            }
            Ok(_) => panic!("Expected missing jump label to return Error"),
        }
    }

    #[test]
    fn jump_back_is_handled_correctly() {
        let spans = vec![
            Span {
                token: Token::Label("jump_back"),
                line: 0,
            },
            Span {
                token: Token::Op(Op::Jmp),
                line: 1,
            },
            Span {
                token: Token::Ident("jump_back"),
                line: 2,
            },
        ];
        let expected_instrs = vec![Instr::Jmp {
            target: "jump_back",
            line: 1,
        }];
        let expected_labels = HashMap::from([("jump_back", 0.into())]);
        let Assembled { instrs, labels } = assemble(&spans).unwrap();
        assert_eq!(instrs, expected_instrs);
        assert_eq!(labels, expected_labels);
        let expected_asm = vec![AsmInstr::Jmp(0.into())];
        let asm = resolve(&instrs, &labels).unwrap();
        assert_eq!(asm, expected_asm);
    }

    #[test]
    fn jump_forward_is_handled_correctly() {
        let spans = vec![
            Span {
                token: Token::Op(Op::Jmp),
                line: 1,
            },
            Span {
                token: Token::Ident("jump_forward"),
                line: 2,
            },
            Span {
                token: Token::Label("jump_forward"),
                line: 3,
            },
        ];
        let expected_instrs = vec![Instr::Jmp {
            target: "jump_forward",
            line: 1,
        }];
        let expected_labels = HashMap::from([("jump_forward", 1.into())]);
        let Assembled { instrs, labels } = assemble(&spans).unwrap();
        assert_eq!(instrs, expected_instrs);
        assert_eq!(labels, expected_labels);
        let expected_asm = vec![AsmInstr::Jmp(1.into())];
        let asm = resolve(&instrs, &labels).unwrap();
        assert_eq!(asm, expected_asm);
    }

    #[test]
    fn jump_twice_is_handled_correctly() {
        let spans = vec![
            Span {
                token: Token::Label("jump_back"),
                line: 0,
            },
            Span {
                token: Token::Op(Op::Jmp),
                line: 1,
            },
            Span {
                token: Token::Ident("jump_back"),
                line: 2,
            },
            Span {
                token: Token::Op(Op::Jmp),
                line: 3,
            },
            Span {
                token: Token::Ident("jump_back"),
                line: 4,
            },
        ];
        let expected_instrs = vec![
            Instr::Jmp {
                target: "jump_back",
                line: 1,
            },
            Instr::Jmp {
                target: "jump_back",
                line: 3,
            },
        ];
        let expected_labels = HashMap::from([("jump_back", 0.into())]);
        let Assembled { instrs, labels } = assemble(&spans).unwrap();
        assert_eq!(instrs, expected_instrs);
        assert_eq!(labels, expected_labels);
        let expected_asm = vec![AsmInstr::Jmp(0.into()), AsmInstr::Jmp(0.into())];
        let asm = resolve(&instrs, &labels).unwrap();
        assert_eq!(asm, expected_asm);
    }
}
