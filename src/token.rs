use core::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Op(Op),
    Value(i64),
    Label(&'src str),
    Ident(&'src str),
}

impl<'src> Token<'src> {
    pub fn new(text: &'src str) -> Self {
        match text {
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
        }
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Op(op) => write!(f, "Token of type Op: {op:?}"),
            Token::Value(value) => write!(f, "Token of type Value with value {value}"),
            Token::Label(label) => write!(f, "Token of type Label with label {label}"),
            Token::Ident(s) => write!(f, "Token of type Ident with value {s}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub token: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            token: f(self.token),
            span: self.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Op,
    Value,
    Label,
    Ident,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenKind::*;
        let kind = match self {
            Op => "Op",
            Value => "Value",
            Label => "Label",
            Ident => "Ident",
        };
        f.write_str(kind)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    // Push,  // ( -- a)
    Pop,   // (a -- )
    Print, // (a -- )
    Add,   // (a b -- a+b)
    Sub,   // (a b -- a-b)
    Mul,   // (a b -- a·b)
    Div,   // (a b -- a÷b)
    Neg,   // (a -- -a)
    Mod,   // (a b -- a%b)
    Dup,   // (a -- a a)
    // Ove,    // (a b -- a b a)
    // Swap,   // (a b -- b a)
    // Rot,    // (a b c -- b c a)
    // And,    // ( a b -- a & b )	Bitwise AND
    // Or,     // ( a b -- a | b )	Bitwise OR
    // Xor,    // ( a b -- a ^ b )	Bitwise exclusive OR
    // Not,    // ( a -- ~a )	Bitwise complement
    // LShift, // ( a n -- a << n )	Shift a left by n bits
    // RShift, // ( a n -- a >> n )	Shift a right by n bits
    Jmp, // (  -- )
    Jlt, // (a b -- ) jump if a < b
    // Jgt,     // (a b -- ) jump if a > b
    Halt, // end program
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_new_gives_expected_result() {
        let lexemes = vec![
            ("add", Token::Op(Op::Add)),
            ("+", Token::Op(Op::Add)),
            ("sub", Token::Op(Op::Sub)),
            ("-", Token::Op(Op::Sub)),
            ("mul", Token::Op(Op::Mul)),
            ("*", Token::Op(Op::Mul)),
            ("div", Token::Op(Op::Div)),
            ("/", Token::Op(Op::Div)),
            ("mod", Token::Op(Op::Mod)),
            ("%", Token::Op(Op::Mod)),
            ("neg", Token::Op(Op::Neg)),
        ];

        for (lexeme, expected) in lexemes {
            assert_eq!(Token::new(lexeme), expected);
        }
    }
}
