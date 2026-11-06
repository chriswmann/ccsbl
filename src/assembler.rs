use std::collections::HashMap;

use crate::{
    bytecode::{AsmInstr, Instr, InstructionIndex},
    error::Error,
    token::{Op, Spanned, Token, TokenKind},
};

#[derive(Debug, Default)]
pub struct Assembled<'src> {
    pub instrs: Vec<Instr<'src>>,
    pub symbols: HashMap<&'src str, InstructionIndex>,
}

pub fn assemble<'src>(
    spans: impl IntoIterator<Item = Spanned<Token<'src>>>,
) -> Result<Assembled<'src>, Vec<Error>> {
    let mut spans = spans.into_iter().peekable();
    let mut instrs = Vec::with_capacity(spans.size_hint().0);
    let mut symbols = HashMap::new();
    let mut errors = Vec::new();
    while let Some(Spanned { token, span }) = spans.next() {
        match token {
            Token::Op(op) => match op {
                Op::Pop => instrs.push(Instr::Pop { span }),
                Op::Print => instrs.push(Instr::Print { span }),
                Op::Add => instrs.push(Instr::Add { span }),
                Op::Sub => instrs.push(Instr::Sub { span }),
                Op::Mul => instrs.push(Instr::Mul { span }),
                Op::Div => instrs.push(Instr::Div { span }),
                Op::Neg => instrs.push(Instr::Neg { span }),
                Op::Mod => instrs.push(Instr::Mod { span }),
                Op::Dup => instrs.push(Instr::Dup { span }),
                Op::Jmp | Op::Jlt => {
                    if let Some(Spanned {
                        token: next_token,
                        span: next_span,
                    }) = spans.peek()
                    {
                        match next_token {
                            Token::Ident(ident) => {
                                match op {
                                    Op::Jmp => instrs.push(Instr::Jmp {
                                        target: ident,
                                        span,
                                    }),
                                    Op::Jlt => instrs.push(Instr::Jlt {
                                        target: ident,
                                        span,
                                    }),
                                    _ => unreachable!(
                                        "Already matched on op to identify that we have a jump of some sort"
                                    ),
                                }
                                spans.next();
                            }
                            Token::Op(_) | Token::Label(_) | Token::Value(_) => {
                                errors.push(Error::NotAJumpLabel {
                                    token: TokenKind::from(next_token),
                                    span: next_span.clone(),
                                });
                            }
                        }
                    } else {
                        errors.push(Error::MissingJumpLabel { span });
                    }
                }
                Op::Halt => instrs.push(Instr::Halt { span }),
            },
            Token::Ident(_) => {
                // Idents aren't implemented yet, so if we encounter one that isn't directly after a
                // jump, it is an error
                errors.push(Error::Token {
                    token: TokenKind::from(&token),
                    span,
                });
            }
            Token::Value(value) => instrs.push(Instr::Push { value, span }),
            Token::Label(label) => {
                if symbols
                    .insert(label, InstructionIndex::from(instrs.len()))
                    .is_some()
                {
                    errors.push(Error::NotUniqueLabel { span });
                }
            }
        }
    }
    if errors.is_empty() {
        return Ok(Assembled { instrs, symbols });
    }
    Err(errors)
}

pub fn resolve(assembled: Assembled<'_>) -> Result<Vec<AsmInstr>, Vec<Error>> {
    let mut asm = Vec::new();
    let mut errors = Vec::new();
    let instrs = assembled.instrs;
    let symbols = assembled.symbols;
    for instr in instrs {
        match instr {
            Instr::Push { value, .. } => asm.push(AsmInstr::Push(value)),
            Instr::Pop { .. } => asm.push(AsmInstr::Pop),
            Instr::Add { .. } => asm.push(AsmInstr::Add),
            Instr::Sub { .. } => asm.push(AsmInstr::Sub),
            Instr::Mul { .. } => asm.push(AsmInstr::Mul),
            Instr::Div { .. } => asm.push(AsmInstr::Div),
            Instr::Neg { .. } => asm.push(AsmInstr::Neg),
            Instr::Mod { .. } => asm.push(AsmInstr::Mod),
            Instr::Dup { .. } => asm.push(AsmInstr::Dup),
            Instr::Print { .. } => asm.push(AsmInstr::Print),
            Instr::Jmp { target, span } => match symbols.get(target) {
                Some(i) => asm.push(AsmInstr::Jmp(*i)),
                None => errors.push(Error::ResolveJumpTarget { span }),
            },
            Instr::Jlt { target, span } => match symbols.get(target) {
                Some(i) => asm.push(AsmInstr::Jlt(*i)),
                None => errors.push(Error::ResolveJumpTarget { span }),
            },
            Instr::Halt { span: _ } => asm.push(AsmInstr::Halt),
        }
    }
    if errors.is_empty() {
        return Ok(asm);
    }
    Err(errors)
}

#[cfg(test)]
mod tests {

    use crate::token::Span;

    use super::*;

    #[test]
    fn empty_spans_returns_empty_instrs() {
        let spans: Vec<Spanned<Token<'_>>> = Vec::new();
        let Assembled { instrs, symbols } = assemble(spans).unwrap();
        assert_eq!(symbols, HashMap::<&str, InstructionIndex>::new());
        assert_eq!(instrs, Vec::<Instr<'_>>::new());
    }

    #[test]
    fn assemble_returns_expected_output_for_given_input() {
        let spans = vec![
            Spanned {
                token: Token::Value(0),
                span: Span::new(0, 1),
            },
            Spanned {
                token: Token::Value(-2_000_000_000),
                span: Span::new(2, 13),
            },
            Spanned {
                token: Token::Op(Op::Pop),
                span: Span::new(14, 17),
            },
            Spanned {
                token: Token::Op(Op::Jmp),
                span: Span::new(18, 21),
            },
            Spanned {
                token: Token::Ident("target"),
                span: Span::new(22, 28),
            },
            Spanned {
                token: Token::Op(Op::Print),
                span: Span::new(29, 34),
            },
            Spanned {
                token: Token::Op(Op::Add),
                span: Span::new(35, 38),
            },
            Spanned {
                token: Token::Op(Op::Halt),
                span: Span::new(39, 43),
            },
            Spanned {
                token: Token::Value(0),
                span: Span::new(44, 45),
            },
        ];

        let expected_instrs = vec![
            Instr::Push {
                value: 0,
                span: Span::new(0, 1),
            },
            Instr::Push {
                value: -2_000_000_000,
                span: Span::new(2, 13),
            },
            Instr::Pop {
                span: Span::new(14, 17),
            },
            Instr::Jmp {
                target: "target",
                span: Span::new(18, 21),
            },
            Instr::Print {
                span: Span::new(29, 34),
            },
            Instr::Add {
                span: Span::new(35, 38),
            },
            Instr::Halt {
                span: Span::new(39, 43),
            },
            Instr::Push {
                value: 0,
                span: Span::new(44, 45),
            },
        ];
        let result = assemble(spans).unwrap();
        assert_eq!(result.instrs, expected_instrs);
        assert_eq!(result.symbols, HashMap::<&str, InstructionIndex>::new());
    }

    #[test]
    fn assemble_returns_expected_report_for_invalid_jump_operand() {
        let spans = vec![
            Spanned {
                token: Token::Op(Op::Jmp),
                span: Span::new(0, 3),
            },
            Spanned {
                token: Token::Op(Op::Pop),
                span: Span::new(4, 7),
            },
        ];
        match assemble(spans) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert!(
                    matches!(
                            &errors[0],
                            Error::NotAJumpLabel { token, span }
                            if *token == TokenKind::Op && *span == Span::new(4, 7)
                    ),
                    "got {:?}",
                    errors[0]
                );
            }
            Ok(assembled) => panic!("Expected error for invalid jump operand, got {assembled:?}"),
        }
    }

    #[test]
    fn assemble_returns_expected_report_for_missing_jump_operand() {
        let spans = vec![
            Spanned {
                token: Token::Op(Op::Jmp),
                span: Span::new(0, 3),
            },
            Spanned {
                token: Token::Op(Op::Halt),
                span: Span::new(4, 8),
            },
        ];
        match assemble(spans) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert!(
                    matches!(
                    &errors[0],
                       Error::NotAJumpLabel { token, span } if *token == TokenKind::Op && *span == Span::new(4, 8)
                    ),
                    "Jmp then halt result was {errors:?}",
                );
            }
            Ok(_) => panic!("Expected error for missing jump operand"),
        }

        let spans = vec![Spanned {
            token: Token::Op(Op::Jmp),
            span: Span::new(100, 103),
        }];
        match assemble(spans) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert!(
                    matches!(
                    &errors[0],
                        Error::MissingJumpLabel { span } if *span == Span::new(100, 103)
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
            Spanned {
                token: Token::Label("jump_back"),
                span: Span::new(0, 10),
            },
            Spanned {
                token: Token::Op(Op::Jmp),
                span: Span::new(11, 14),
            },
            Spanned {
                token: Token::Ident("jump_back"),
                span: Span::new(15, 24),
            },
        ];
        let expected_instrs = vec![Instr::Jmp {
            target: "jump_back",
            span: Span::new(11, 14),
        }];
        let expected_labels = HashMap::from([("jump_back", 0.into())]);
        let Assembled { instrs, symbols } = assemble(spans).unwrap();
        assert_eq!(instrs, expected_instrs);
        assert_eq!(symbols, expected_labels);
        let expected_asm = vec![AsmInstr::Jmp(0.into())];
        let asm = resolve(Assembled { instrs, symbols }).unwrap();
        assert_eq!(asm, expected_asm);
    }

    #[test]
    fn jump_forward_is_handled_correctly() {
        let spans = vec![
            Spanned {
                token: Token::Op(Op::Jmp),
                span: Span::new(0, 3),
            },
            Spanned {
                token: Token::Ident("jump_forward"),
                span: Span::new(4, 16),
            },
            Spanned {
                token: Token::Label("jump_forward"),
                span: Span::new(17, 20),
            },
        ];
        let expected_instrs = vec![Instr::Jmp {
            target: "jump_forward",
            span: Span::new(0, 3),
        }];
        let expected_symbols = HashMap::from([("jump_forward", 1.into())]);
        let Assembled { instrs, symbols } = assemble(spans).unwrap();
        assert_eq!(instrs, expected_instrs);
        assert_eq!(symbols, expected_symbols);
        let expected_asm = vec![AsmInstr::Jmp(1.into())];
        let asm = resolve(Assembled { instrs, symbols }).unwrap();
        assert_eq!(asm, expected_asm);
    }

    #[test]
    fn jump_twice_is_handled_correctly() {
        let spans = vec![
            Spanned {
                token: Token::Label("jump_back"),
                span: Span::new(0, 9),
            },
            Spanned {
                token: Token::Op(Op::Jmp),
                span: Span::new(10, 13),
            },
            Spanned {
                token: Token::Ident("jump_back"),
                span: Span::new(14, 23),
            },
            Spanned {
                token: Token::Op(Op::Jmp),
                span: Span::new(24, 27),
            },
            Spanned {
                token: Token::Ident("jump_back"),
                span: Span::new(28, 27),
            },
        ];
        let expected_instrs = vec![
            Instr::Jmp {
                target: "jump_back",
                span: Span::new(10, 13),
            },
            Instr::Jmp {
                target: "jump_back",
                span: Span::new(24, 27),
            },
        ];
        let expected_symbols = HashMap::from([("jump_back", 0.into())]);
        let Assembled { instrs, symbols } = assemble(spans).unwrap();
        assert_eq!(instrs, expected_instrs);
        assert_eq!(symbols, expected_symbols);
        let expected_asm = vec![AsmInstr::Jmp(0.into()), AsmInstr::Jmp(0.into())];
        let asm = resolve(Assembled { instrs, symbols }).unwrap();
        assert_eq!(asm, expected_asm);
    }

    #[test]
    fn jlt_is_handled_correctly() {
        let spans = vec![
            Spanned {
                token: Token::Label("jump_back"),
                span: Span::new(0, 9),
            },
            Spanned {
                token: Token::Value(1),
                span: Span::new(10, 11),
            },
            Spanned {
                token: Token::Value(2),
                span: Span::new(12, 13),
            },
            Spanned {
                token: Token::Op(Op::Jlt),
                span: Span::new(14, 17),
            },
            Spanned {
                token: Token::Ident("jump_back"),
                span: Span::new(18, 27),
            },
            Spanned {
                token: Token::Op(Op::Jlt),
                span: Span::new(28, 31),
            },
            Spanned {
                token: Token::Ident("jump_back"),
                span: Span::new(32, 41),
            },
        ];
        let expected_instrs = vec![
            Instr::Push {
                value: 1,
                span: Span::new(10, 11),
            },
            Instr::Push {
                value: 2,
                span: Span::new(12, 13),
            },
            Instr::Jlt {
                target: "jump_back",
                span: Span::new(14, 17),
            },
            Instr::Jlt {
                target: "jump_back",
                span: Span::new(28, 31),
            },
        ];
        let expected_symbols = HashMap::from([("jump_back", 0.into())]);
        let Assembled { instrs, symbols } = assemble(spans).unwrap();
        assert_eq!(instrs, expected_instrs);
        assert_eq!(symbols, expected_symbols);
        let expected_asm = vec![
            AsmInstr::Push(1),
            AsmInstr::Push(2),
            AsmInstr::Jlt(0.into()),
            AsmInstr::Jlt(0.into()),
        ];
        let asm = resolve(Assembled { instrs, symbols }).unwrap();
        assert_eq!(asm, expected_asm);
    }
}
