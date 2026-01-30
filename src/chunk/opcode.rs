use std::fmt;

#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Return => write!(f, "OP_RETURN"),
        }
    }
}
