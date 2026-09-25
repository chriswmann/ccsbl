use crate::token::{Op, Span, Token};

#[derive(Debug, PartialEq)]
struct Lexeme<'src> {
    pub text: &'src str,
    pub line: usize,
}

// Split the code into Lexemes, keeping line number for improved compiler
// errors. Empty lines are not filtered, so that the line number matches the
// input file to make locating errors easier.
fn scan(code: &str) -> Vec<Lexeme<'_>> {
    let mut lexemes = Vec::new();
    for (line_num, line) in code.lines().map(str::trim).enumerate() {
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
fn classify<'src>(lexemes: &[Lexeme<'src>]) -> Vec<Span<'src>> {
    let mut spans = Vec::new();
    for &Lexeme { text, line } in lexemes {
        match text {
            "#" => break,
            "add" | "+" => spans.push(Span {
                token: Token::Op(Op::Add),
                line,
            }),
            "sub" | "-" => spans.push(Span {
                token: Token::Op(Op::Sub),
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

    // #[test]
    // fn tokenises_jmp_labels_correctly() {
    // }
    //
    // #[test]
    // fn symbol_aliases_match_named_ops() {
    //     assert_eq!(scanner("+"), scanner("add"));
    //     assert_eq!(scanner("-"), scanner("sub"));
    // }
    //
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
            classify(&scan("-2 pop # this comment should be ignored")),
            vec![
                Span {
                    token: Token::Value(-2),
                    line: 0,
                },
                Span {
                    token: Token::Op(Op::Pop),
                    line: 0
                },
            ]
        );
    }

    // #[test]
    // fn example_programs() {
    //     let expected = vec![
    //         Span {
    //             token: Token::Value(1),
    //             line: 0,
    //         },
    //         Span {
    //             token: Token::Value(2),
    //             line: 1,
    //         },
    //         Span {
    //             token: Token::Value(3),
    //             line: 2,
    //         },
    //         Span {
    //             token: Token::Op(Op::Print),
    //             line: 3,
    //         },
    //         Span {
    //             token: Token::Op(Op::Pop),
    //             line: 4,
    //         },
    //         Span {
    //             token: Token::Op(Op::Print),
    //             line: 5,
    //         },
    //         Span {
    //             token: Token::Op(Op::Halt),
    //             line: 6,
    //         },
    //     ];
    //     assert_eq!(
    //         scanner(include_str!("../tests/test_no_errors.ccl")),
    //         expected
    //     );
    // }
}
