use crate::{errors::Error, ops::Op};
use tracing::debug;

#[derive(Debug)]
pub enum Token {
    Op(Op),
    Value(i64),
    Label(String),
}

// Tokenise the code. At the moment we don't handle comments
// or care about the type of whitespace. We only have a single
// type (i64) and a limited set of instructions.
pub fn tokenise(code: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    for s in code.split_whitespace() {
        match s {
            s if s.ends_with(':') => {
                debug!("Token: {s}");
                tokens.push(Token::Label(s.to_owned()));
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
                    return Err(Error::SyntaxError(other.to_string()));
                }
            }
        }
    }
    Ok(tokens)
}
