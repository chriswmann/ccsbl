use crate::{
    bytecode::AsmInstr,
    errors::Error,
    token::{Op, Span, Token},
};

// Not worrying about stack being empty or too deep yet
pub fn assemble<'src>(spans: &[Span<'src>]) -> Result<Vec<AsmInstr<'src>>, Error<'src>> {
    if spans.is_empty() {
        return Ok(Vec::new());
    }
    let mut instrs = Vec::with_capacity(spans.len());
    let mut idx = 0;
    loop {
        if idx == spans.len() {
            return Ok(instrs);
        }
        let Span { token, line } = spans[idx].clone();
        match token {
            Token::Op(op) => match op {
                Op::Pop => instrs.push(AsmInstr::Pop { line }),
                Op::Print => instrs.push(AsmInstr::Print { line }),
                Op::Add => instrs.push(AsmInstr::Add { line }),
                Op::Sub => instrs.push(AsmInstr::Sub { line }),
                Op::Mul => instrs.push(AsmInstr::Mul { line }),
                Op::Div => instrs.push(AsmInstr::Div { line }),
                Op::Jmp => {
                    if (idx + 1) < spans.len() {
                        let Span {
                            token: next_token,
                            line: next_line,
                        } = spans[idx + 1].clone();
                        match next_token {
                            Token::Ident(ident) => {
                                instrs.push(AsmInstr::Jmp { label: ident, line });
                                idx += 1;
                            }
                            Token::Op(_) | Token::Value(_) => {
                                return Err(Error::NotAJumpLabel {
                                    token: next_token,
                                    line: next_line,
                                });
                            }
                        }
                    } else {
                        return Err(Error::MissingJumpLabel { line });
                    }
                }
                Op::Halt => {
                    instrs.push(AsmInstr::Halt { line });
                    return Ok(instrs);
                }
            },
            Token::Value(value) => instrs.push(AsmInstr::Push { value, line }),
            // At the current stage of the language, if we encounter an identifier that doesn't
            // follow a jump, this is an error.
            token @ Token::Ident(_) => return Err(Error::Token { token, line }),
        }
        idx += 1;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn empty_spans_returns_empty_instrs() {
        let spans = Vec::new();
        assert!(assemble(&spans).is_ok());
        assert_eq!(assemble(&spans).unwrap(), Vec::<AsmInstr>::new());
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
            // Expected to be missing from the output since the previous
            // token is Halt.
            Span {
                token: Token::Value(0),
                line: 2,
            },
        ];

        let expected = vec![
            AsmInstr::Push { value: 0, line: 2 },
            AsmInstr::Push {
                value: -2_000_000_000,
                line: 2,
            },
            AsmInstr::Pop { line: 2 },
            AsmInstr::Jmp {
                label: "target",
                line: 3,
            },
            AsmInstr::Print { line: 3 },
            AsmInstr::Add { line: 4 },
            AsmInstr::Halt { line: 4 },
        ];
        assert_eq!(assemble(&spans).unwrap(), expected);
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
        let result = assemble(&spans);
        assert!(matches!(
            &result,
            Err(Error::NotAJumpLabel { token, line }) if token == &Token::Op(Op::Pop) && *line == 1,
        ));
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
        let result = assemble(&spans);
        assert!(
            matches!(
            &result,
                Err(Error::NotAJumpLabel { token, line }) if *token == Token::Op(Op::Halt) && *line == 1
            ),
            "Jmp then halt result was {result:?}"
        );

        let spans = vec![Span {
            token: Token::Op(Op::Jmp),
            line: 100,
        }];
        let result = assemble(&spans);
        assert!(
            matches!(
            &result,
                Err(Error::MissingJumpLabel { line }) if *line == 100
            ),
            "Result was {result:?}"
        );
    }
}
