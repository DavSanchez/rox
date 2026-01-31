use std::fmt;

use thiserror::Error;

#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant,
}

#[derive(Debug, Error)]
#[error("Unknown opcode: {0}")]
pub struct UnknownOpcode(u8);

impl TryFrom<u8> for OpCode {
    type Error = UnknownOpcode;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(OpCode::Return),
            1 => Ok(OpCode::Constant),
            _ => Err(UnknownOpcode(byte)),
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Return => write!(f, "OP_RETURN"),
            OpCode::Constant => write!(f, "OP_CONSTANT"),
        }
    }
}
