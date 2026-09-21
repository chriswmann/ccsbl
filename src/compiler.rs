use crate::{errors::Error, ops::Op};
use tracing::debug;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op(Op),
    Value(i64),
    Label(String),
}

// Tokenise the code. At the moment we don't allow more than
// one token per line nor do we handle comments
// There is only one type (i64) and a limited set of instructions.
pub fn tokenise(code: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    for (line_no, line) in code
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .enumerate()
    {
        for token in line.split_whitespace() {
            match token {
                "#" => break,
                token if token.ends_with(':') => {
                    let token = token.strip_suffix(':').expect(
                        "Should be able to strip ':' after checking `token.ends_with(':')`",
                    );
                    debug!("Token: {token}");
                    tokens.push(Token::Label(token.to_owned()));
                }
                "add" | "+" => tokens.push(Token::Op(Op::Add)),
                "sub" | "-" => tokens.push(Token::Op(Op::Sub)),
                "pop" => tokens.push(Token::Op(Op::Pop)),
                "print" => tokens.push(Token::Op(Op::Print)),
                "halt" => tokens.push(Token::Op(Op::Halt)),
                other => {
                    if let Ok(num) = &str::parse::<i64>(other) {
                        tokens.push(Token::Value(*num));
                    } else {
                        return Err(Error::Token {
                            token: other.to_string(),
                            line_number: line_no,
                        });
                    }
                }
            }
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenises_labels_values_and_ops_in_order() {
        let code = "start:\n1\n2\nadd\nprint\nhalt";
        let expected = vec![
            Token::Label("start".to_string()),
            Token::Value(1),
            Token::Value(2),
            Token::Op(Op::Add),
            Token::Op(Op::Print),
            Token::Op(Op::Halt),
        ];
        assert_eq!(tokenise(code).unwrap(), expected);
    }

    #[test]
    fn symbol_aliases_match_named_ops() {
        assert_eq!(tokenise("+").unwrap(), tokenise("add").unwrap());
        assert_eq!(tokenise("-").unwrap(), tokenise("sub").unwrap());
    }

    #[test]
    fn negative_number_is_a_value_not_sub() {
        assert_eq!(tokenise("-2").unwrap(), vec![Token::Value(-2)]);
    }

    #[test]
    fn blank_lines_and_surrounding_spaces_are_ignored() {
        assert_eq!(tokenise("\n\n\n 1 ").unwrap(), vec![Token::Value(1)]);
    }

    #[test]
    fn empty_input_gives_no_tokens() {
        assert_eq!(tokenise("").unwrap(), Vec::<Token>::new());
    }

    #[test]
    fn comments_are_ignored() {
        let expected = vec![Token::Value(-2), Token::Op(Op::Pop)];
        assert_eq!(
            tokenise("-2 pop # everything here is an ignored comment").unwrap(),
            expected
        );
    }

    #[test]
    fn unknown_token_reports_token_and_line() {
        let result = tokenise("1\n2\nunknown_token\nhalt");
        match result {
            Err(Error::Token { token, line_number }) => {
                assert_eq!(token, "unknown_token");
                assert_eq!(line_number, 2);
            }
            other => panic!("expected Error::Token, got {other:?}"),
        }
    }

    #[test]
    fn example_programs() {
        assert!(tokenise(include_str!("../tests/test_no_errors.ccl")).is_ok());
        assert!(matches!(
            tokenise(include_str!("../tests/test_unknown_token.ccl")),
            Err(Error::Token { line_number: 7, .. })
        ));
    }
}
