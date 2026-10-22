use crate::token::{Op, Span, Spanned, Token};

#[derive(Debug)]
struct Scanner<'src> {
    source: &'src str,
    offset: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, offset: 0 }
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Spanned<&'src str>;

    fn next(&mut self) -> Option<Self::Item> {
        let base = self.offset;
        let mut chars = self.source[base..].char_indices();

        // Skip any leading whitespace
        let Some((relative_start, _)) = chars.find(|(_, ch)| !ch.is_whitespace()) else {
            self.offset = self.source.len();
            return None;
        };

        let start = base + relative_start;

        // Find the next whitespace or the end of the source (ignoring comments for now)
        let end = chars
            .find(|(_, ch)| ch.is_whitespace())
            .map_or(self.source.len(), |(relative_end, _)| base + relative_end);
        self.offset = end;

        Some(Spanned {
            token: &self.source[start..end],
            span: Span::new(start, end),
        })
    }
}

pub fn scan(source: &str) -> impl Iterator<Item = Spanned<&str>> {
    Scanner::new(source)
}

// Classify each Lexeme into a Token, keeping the line number for
// error messages.
pub fn classify<'src>(lexemes: &[Lexeme<'src>]) -> Vec<Spanned<'src>> {
    let mut spans = Vec::new();
    for &Lexeme { text, line } in lexemes {
        let token = match text {
            "add" | "+" => Token::Op(Op::Add),
            "sub" | "-" => Token::Op(Op::Sub),
            "mul" | "*" => Token::Op(Op::Mul),
            "div" | "/" => Token::Op(Op::Div),
            "neg" | "negate" => Token::Op(Op::Neg),
            "%" | "mod" => Token::Op(Op::Mod),
            "dup" => Token::Op(Op::Dup),
            "pop" => Token::Op(Op::Pop),
            "print" => Token::Op(Op::Print),
            "jmp" => Token::Op(Op::Jmp),
            "jlt" => Token::Op(Op::Jlt),
            "halt" => Token::Op(Op::Halt),
            other => {
                if let Some(label) = other.strip_suffix(':') {
                    Token::Label(label)
                } else if let Ok(num) = str::parse::<i64>(other) {
                    Token::Value(num)
                } else {
                    Token::Ident(other)
                }
            }
        };
        spans.push(Spanned::new(token, line));
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
            Spanned {
                token: Token::Value(1),
                line: 0,
            },
            Spanned {
                token: Token::Op(Op::Sub),
                line: 0,
            },
            Spanned {
                token: Token::Value(2),
                line: 0,
            },
            Spanned {
                token: Token::Value(-2),
                line: 1,
            },
        ];
        assert_eq!(classify(&lexemes), expected);
    }

    #[test]
    fn empty_input_gives_no_tokens() {
        let lexemes = scan("");
        assert_eq!(classify(&lexemes), Vec::<Spanned<'_>>::new());
    }

    #[test]
    fn comments_are_ignored() {
        assert_eq!(
            classify(&scan("-2 pop # this comment should be ignored\n0")),
            vec![
                Spanned {
                    token: Token::Value(-2),
                    line: 0,
                },
                Spanned {
                    token: Token::Op(Op::Pop),
                    line: 0
                },
                Spanned {
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
                Spanned {
                    token: Token::Op(Op::Add),
                    line: 0
                },
                Spanned {
                    token: Token::Op(Op::Add),
                    line: 0
                },
                Spanned {
                    token: Token::Op(Op::Sub),
                    line: 0
                },
                Spanned {
                    token: Token::Op(Op::Sub),
                    line: 0
                },
                Spanned {
                    token: Token::Op(Op::Mul),
                    line: 0
                },
                Spanned {
                    token: Token::Op(Op::Mul),
                    line: 0
                },
                Spanned {
                    token: Token::Op(Op::Div),
                    line: 0
                },
                Spanned {
                    token: Token::Op(Op::Div),
                    line: 0
                },
            ]
        );
    }

    #[test]
    fn labels_and_idents_are_classified_correctly() {
        let lexemes = vec![
            Lexeme {
                text: "an_ident",
                line: 0,
            },
            Lexeme {
                text: "a_label:",
                line: 0,
            },
            Lexeme {
                text: "another_label:",
                line: 2,
            },
            Lexeme {
                text: "another_ident",
                line: 4,
            },
        ];
        let expected = vec![
            Spanned {
                token: Token::Ident("an_ident"),
                line: 0,
            },
            Spanned {
                token: Token::Label("a_label"),
                line: 0,
            },
            Spanned {
                token: Token::Label("another_label"),
                line: 2,
            },
            Spanned {
                token: Token::Ident("another_ident"),
                line: 4,
            },
        ];
        let result = classify(&lexemes);
        assert_eq!(result, expected);
    }

    #[test]
    fn example_program_test_no_errors() {
        let expected = vec![
            Spanned {
                token: Token::Value(1),
                line: 0,
            },
            Spanned {
                token: Token::Value(2),
                line: 1,
            },
            Spanned {
                token: Token::Value(3),
                line: 2,
            },
            Spanned {
                token: Token::Op(Op::Print),
                line: 3,
            },
            Spanned {
                token: Token::Op(Op::Pop),
                line: 4,
            },
            Spanned {
                token: Token::Op(Op::Print),
                line: 5,
            },
            Spanned {
                token: Token::Value(2),
                line: 6,
            },
            Spanned {
                token: Token::Value(2),
                line: 6,
            },
            Spanned {
                token: Token::Op(Op::Mul),
                line: 6,
            },
            Spanned {
                token: Token::Op(Op::Print),
                line: 7,
            },
            Spanned {
                token: Token::Value(1),
                line: 8,
            },
            Spanned {
                token: Token::Value(2),
                line: 8,
            },
            Spanned {
                token: Token::Op(Op::Jmp),
                line: 9,
            },
            Spanned {
                token: Token::Ident("end"),
                line: 9,
            },
            Spanned {
                token: Token::Op(Op::Halt),
                line: 10,
            },
            Spanned {
                token: Token::Label("end"),
                line: 11,
            },
            Spanned {
                token: Token::Op(Op::Print),
                line: 12,
            },
            Spanned {
                token: Token::Op(Op::Pop),
                line: 13,
            },
            Spanned {
                token: Token::Value(4),
                line: 14,
            },
            Spanned {
                token: Token::Op(Op::Neg),
                line: 15,
            },
            Spanned {
                token: Token::Value(3),
                line: 16,
            },
            Spanned {
                token: Token::Op(Op::Mod),
                line: 17,
            },
            Spanned {
                token: Token::Op(Op::Print),
                line: 18,
            },
        ];
        assert_eq!(
            classify(&scan(include_str!("../tests/test_no_errors.ccl"))),
            expected
        );
    }

    #[test]
    fn example_program_test_conditional_loops() {
        let expected = vec![
            Spanned {
                token: Token::Value(1),
                line: 0,
            },
            Spanned {
                token: Token::Label("start"),
                line: 1,
            },
            Spanned {
                token: Token::Value(1),
                line: 2,
            },
            Spanned {
                token: Token::Op(Op::Add),
                line: 2,
            },
            Spanned {
                token: Token::Op(Op::Dup),
                line: 3,
            },
            Spanned {
                token: Token::Value(4),
                line: 4,
            },
            Spanned {
                token: Token::Op(Op::Jlt),
                line: 5,
            },
            Spanned {
                token: Token::Ident("start"),
                line: 5,
            },
            Spanned {
                token: Token::Op(Op::Print),
                line: 6,
            },
        ];
        assert_eq!(
            classify(&scan(include_str!("../tests/test_conditional_loops.ccl"))),
            expected
        );
    }
}
