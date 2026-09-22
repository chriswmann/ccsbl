use crate::bytecode::Op;

#[derive(Debug, PartialEq)]
pub enum Token<'src> {
    Op(Op),
    Value(i64),
    Label(&'src str),
    Ident(&'src str),
}

#[derive(Debug, PartialEq)]
pub struct Span<'src> {
    pub token: Token<'src>,
    pub line: usize,
}

// Compile a source file. Tokenise, then emit

// Tokenise the code. At the moment we don't allow more than
// one token per line nor do we handle comments
// There is only one type (i64) and a limited set of instructions.
fn scanner(code: &str) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    for (line_no, line) in code
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .enumerate()
    {
        for lexeme in line.split_whitespace() {
            match lexeme {
                "#" => break,
                lexeme if lexeme.ends_with(':') => spans.push(Span {
                    token: Token::Label(lexeme),
                    line: line_no,
                }),
                "add" | "+" => spans.push(Span {
                    token: Token::Op(Op::Add),
                    line: line_no,
                }),
                "sub" | "-" => spans.push(Span {
                    token: Token::Op(Op::Sub),
                    line: line_no,
                }),
                "pop" => spans.push(Span {
                    token: Token::Op(Op::Pop),
                    line: line_no,
                }),
                "print" => spans.push(Span {
                    token: Token::Op(Op::Print),
                    line: line_no,
                }),
                "jmp" => spans.push(Span {
                    token: Token::Op(Op::Jmp),
                    line: line_no,
                }),
                "halt" => spans.push(Span {
                    token: Token::Op(Op::Halt),
                    line: line_no,
                }),
                other => {
                    if let Ok(num) = &str::parse::<i64>(other) {
                        spans.push(Span {
                            token: Token::Value(*num),
                            line: line_no,
                        });
                    } else {
                        spans.push(Span {
                            token: Token::Ident(other),
                            line: line_no,
                        });
                    }
                }
            }
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenises_labels_values_and_ops_in_order() {
        let code = "start:\n1\n2\nadd\nprint\nhalt";
        let expected = vec![
            Span {
                token: Token::Label("start:"),
                line: 0,
            },
            Span {
                token: Token::Value(1),
                line: 1,
            },
            Span {
                token: Token::Value(2),
                line: 2,
            },
            Span {
                token: Token::Op(Op::Add),
                line: 3,
            },
            Span {
                token: Token::Op(Op::Print),
                line: 4,
            },
            Span {
                token: Token::Op(Op::Halt),
                line: 5,
            },
        ];
        assert_eq!(scanner(code), expected);
    }

    #[test]
    fn tokenises_jmp_labels_correctly() {
        let code = "start:\n1\n2\nadd\nprint\n1 2 jmp start halt";
        let expected = vec![
            Span {
                token: Token::Label("start:"),
                line: 0,
            },
            Span {
                token: Token::Value(1),
                line: 1,
            },
            Span {
                token: Token::Value(2),
                line: 2,
            },
            Span {
                token: Token::Op(Op::Add),
                line: 3,
            },
            Span {
                token: Token::Op(Op::Print),
                line: 4,
            },
            Span {
                token: Token::Value(1),
                line: 5,
            },
            Span {
                token: Token::Value(2),
                line: 5,
            },
            Span {
                token: Token::Op(Op::Jmp),
                line: 5,
            },
            Span {
                token: Token::Ident("start"),
                line: 5,
            },
            Span {
                token: Token::Op(Op::Halt),
                line: 5,
            },
        ];
        assert_eq!(scanner(code), expected);
    }

    #[test]
    fn symbol_aliases_match_named_ops() {
        assert_eq!(scanner("+"), scanner("add"));
        assert_eq!(scanner("-"), scanner("sub"));
    }

    #[test]
    fn negative_number_is_a_value_not_sub() {
        assert_eq!(
            scanner("-2"),
            vec![Span {
                token: Token::Value(-2),
                line: 0
            }]
        );
    }

    #[test]
    fn blank_lines_and_surrounding_spaces_are_ignored() {
        assert_eq!(
            scanner("\n\n\n 1 "),
            vec![Span {
                token: Token::Value(1),
                line: 0
            }]
        );
    }

    #[test]
    fn empty_input_gives_no_tokens() {
        assert_eq!(scanner(""), Vec::<Span>::new());
    }

    #[test]
    fn comments_are_ignored() {
        let expected = vec![
            Span {
                token: Token::Value(-2),
                line: 0,
            },
            Span {
                token: Token::Op(Op::Pop),
                line: 0,
            },
        ];
        assert_eq!(
            scanner("-2 pop # everything here is an ignored comment"),
            expected
        );
    }

    #[test]
    fn example_programs() {
        let expected = vec![
            Span {
                token: Token::Value(1),
                line: 0,
            },
            Span {
                token: Token::Value(2),
                line: 1,
            },
            Span {
                token: Token::Value(3),
                line: 2,
            },
            Span {
                token: Token::Op(Op::Print),
                line: 3,
            },
            Span {
                token: Token::Op(Op::Pop),
                line: 4,
            },
            Span {
                token: Token::Op(Op::Print),
                line: 5,
            },
            Span {
                token: Token::Op(Op::Halt),
                line: 6,
            },
        ];
        assert_eq!(
            scanner(include_str!("../tests/test_no_errors.ccl")),
            expected
        );
    }
}
