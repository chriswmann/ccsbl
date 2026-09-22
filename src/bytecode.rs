use strum::FromRepr;

#[derive(Clone, Copy, Debug, FromRepr, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Op {
    Pop = 0x01, // (a -- )
    Add = 0x02, // (a b -- a+b)
    Sub = 0x03, // (a b -- a-b)
    // Mul = 0x05,    // (a b -- a·b)
    // Div = 0x05,    // (a b -- a÷b)
    // Neg = 0x06,    // (a -- -a)
    // Mod = 0x07,    // (-a -- a)
    // Dup = 0x08,    // (a -- a a)
    // Over = 0x09,   // (a b -- a b a)
    // Swap = 0x0a,   // (a b -- b a)
    // Rot = 0x0b,    // (a b c -- b c a)
    // And = 0x0c,    // ( a b -- a & b )	Bitwise AND
    // Or = 0x0d,     // ( a b -- a | b )	Bitwise OR
    // Xor = 0x0e,    // ( a b -- a ^ b )	Bitwise exclusive OR
    // Not = 0x0f,    // ( a -- ~a )	Bitwise complement
    // LShift = 0x10, // ( a n -- a << n )	Shift a left by n bits
    // RShift = 0x11, // ( a n -- a >> n )	Shift a right by n bits
    Jmp = 0x12,   // (  -- )
    Print = 0x13, // (a b -- a) with b printed to stdout
    Halt = 0xFF,  // end program
}

#[derive(Clone, Debug)]
pub enum Instr {}
