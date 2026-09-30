use crate::token::{Op, Span, Token};

#[derive(Debug, PartialEq)]
pub struct Lexeme<'src> {
    pub text: &'src str,
    pub line: usize,
}

// Split the code into Lexemes, keeping line number for improved compiler
// errors. Empty lines are not filtered, so that the line number matches the
// input file to make locating errors easier.
// Split on '#' and keep just the first part of the line, to discard comments.
pub fn scan(code: &str) -> Vec<Lexeme<'_>> {
    let mut lexemes = Vec::new();
    for (line_num, line) in code.lines().map(str::trim).enumerate() {
        let line = line.split('#').next().unwrap();
        for text in line.split_whitespace() {
            lexemes.push(Lexeme {
                text,
                line: line_num,
            });
        }
    }
    lexemes
}

// Classify each Lexeme into a Token, keeping the line number for
// error messages.
pub fn classify<'src>(lexemes: &[Lexeme<'src>]) -> Vec<Span<'src>> {
    let mut spans = Vec::new();
    for &Lexeme { text, line } in lexemes {
        match text {
            "add" | "+" => spans.push(Span {
                token: Token::Op(Op::Add),
                line,
            }),
            "sub" | "-" => spans.push(Span {
                token: Token::Op(Op::Sub),
                line,
            }),
            "mul" | "*" => spans.push(Span {
                token: Token::Op(Op::Mul),
                line,
            }),
            "div" | "/" => spans.push(Span {
                token: Token::Op(Op::Div),
                line,
            }),
            "pop" => spans.push(Span {
                token: Token::Op(Op::Pop),
                line,
            }),
            "print" => spans.push(Span {
                token: Token::Op(Op::Print),
                line,
            }),
            "jmp" => spans.push(Span {
                token: Token::Op(Op::Jmp),
                line,
            }),
            "halt" => spans.push(Span {
                token: Token::Op(Op::Halt),
                line,
            }),
            other => {
                if let Ok(num) = &str::parse::<i64>(other) {
                    spans.push(Span {
                        token: Token::Value(*num),
                        line,
                    });
                } else {
                    spans.push(Span {
                        token: Token::Ident(other),
                        line,
                    });
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
    fn scans_lexemes_and_line_numbers_correctly() {
        let code = "start:\n1\n2\n\n\nadd\nprint\nhalt";
        let expected = vec![
            Lexeme {
                text: "start:",
                line: 0,
            },
            Lexeme { text: "1", line: 1 },
            Lexeme { text: "2", line: 2 },
            Lexeme {
                text: "add",
                line: 5,
            },
            Lexeme {
                text: "print",
                line: 6,
            },
            Lexeme {
                text: "halt",
                line: 7,
            },
        ];
        assert_eq!(scan(code), expected);
    }

    #[test]
    fn scanned_blank_lines_and_surrounding_spaces_are_ignored() {
        assert_eq!(scan("\n\n\n 1 "), vec![Lexeme { text: "1", line: 3 }]);
    }

    #[test]
    fn negative_number_is_a_value_not_sub() {
        let lexemes = vec![
            Lexeme { text: "1", line: 0 },
            Lexeme { text: "-", line: 0 },
            Lexeme { text: "2", line: 0 },
            Lexeme {
                text: "-2",
                line: 1,
            },
        ];
        let expected = vec![
            Span {
                token: Token::Value(1),
                line: 0,
            },
            Span {
                token: Token::Op(Op::Sub),
                line: 0,
            },
            Span {
                token: Token::Value(2),
                line: 0,
            },
            Span {
                token: Token::Value(-2),
                line: 1,
            },
        ];
        assert_eq!(classify(&lexemes), expected);
    }

    #[test]
    fn empty_input_gives_no_tokens() {
        let lexemes = scan("");
        assert_eq!(classify(&lexemes), Vec::<Span>::new());
    }

    #[test]
    fn comments_are_ignored() {
        assert_eq!(
            classify(&scan("-2 pop # this comment should be ignored\n0")),
            vec![
                Span {
                    token: Token::Value(-2),
                    line: 0,
                },
                Span {
                    token: Token::Op(Op::Pop),
                    line: 0
                },
                Span {
                    token: Token::Value(0),
                    line: 1,
                }
            ]
        );
    }

    #[test]
    fn arithmetic_aliases_are_classified_correctly() {
        let lexemes = vec![
            Lexeme {
                text: "add",
                line: 0,
            },
            Lexeme { text: "+", line: 0 },
            Lexeme {
                text: "sub",
                line: 0,
            },
            Lexeme { text: "-", line: 0 },
            Lexeme {
                text: "mul",
                line: 0,
            },
            Lexeme { text: "*", line: 0 },
            Lexeme {
                text: "div",
                line: 0,
            },
            Lexeme { text: "/", line: 0 },
        ];
        assert_eq!(
            classify(&lexemes),
            vec![
                Span {
                    token: Token::Op(Op::Add),
                    line: 0
                },
                Span {
                    token: Token::Op(Op::Add),
                    line: 0
                },
                Span {
                    token: Token::Op(Op::Sub),
                    line: 0
                },
                Span {
                    token: Token::Op(Op::Sub),
                    line: 0
                },
                Span {
                    token: Token::Op(Op::Mul),
                    line: 0
                },
                Span {
                    token: Token::Op(Op::Mul),
                    line: 0
                },
                Span {
                    token: Token::Op(Op::Div),
                    line: 0
                },
                Span {
                    token: Token::Op(Op::Div),
                    line: 0
                },
            ]
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
                token: Token::Value(2),
                line: 6,
            },
            Span {
                token: Token::Value(2),
                line: 6,
            },
            Span {
                token: Token::Op(Op::Mul),
                line: 6,
            },
            Span {
                token: Token::Op(Op::Print),
                line: 7,
            },
            Span {
                token: Token::Op(Op::Halt),
                line: 8,
            },
        ];
        assert_eq!(
            classify(&scan(include_str!("../tests/test_no_errors.ccl"))),
            expected
        );
    }
}
