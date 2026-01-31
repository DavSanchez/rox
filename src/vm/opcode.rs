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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", OpCode::Return), "OP_RETURN");
        assert_eq!(format!("{}", OpCode::Constant), "OP_CONSTANT");
    }

    #[test]
    fn test_from_u8_valid() {
        assert!(matches!(OpCode::try_from(0), Ok(OpCode::Return)));
        assert!(matches!(OpCode::try_from(1), Ok(OpCode::Constant)));
    }

    proptest! {
        #[test]
        fn prop_opcode_conversion(byte in 0u8..=255) {
            match byte {
                0 | 1 => prop_assert!(OpCode::try_from(byte).is_ok()),
                _ => prop_assert!(OpCode::try_from(byte).is_err()),
            }
        }
    }
}
