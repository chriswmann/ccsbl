use core::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Op(Op),
    Value(i64),
    Ident(&'src str),
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Op(op) => write!(f, "Token of type Op: {op:?}"),
            Token::Value(value) => write!(f, "Token of type Value with value {value}"),
            Token::Ident(s) => write!(f, "Token of type Ident with value {s}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Span<'src> {
    pub token: Token<'src>,
    pub line: usize,
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
    // Neg,    // (a -- -a)
    // Mod,    // (-a -- a)
    // Dup,    // (a -- a a)
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
    // Jt,     // (a b -- a) with b printed to stdout
    Halt, // end program
}
