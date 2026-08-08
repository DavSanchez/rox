use std::fmt;

use thiserror::Error;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
#[error("Unknown opcode: {0}")]
pub struct UnknownOpcode(u8);

impl TryFrom<u8> for OpCode {
    type Error = UnknownOpcode;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(Self::Return),
            1 => Ok(Self::Constant),
            2 => Ok(Self::Negate),
            3 => Ok(Self::Add),
            4 => Ok(Self::Subtract),
            5 => Ok(Self::Multiply),
            6 => Ok(Self::Divide),
            7 => Ok(Self::Nil),
            8 => Ok(Self::True),
            9 => Ok(Self::False),
            10 => Ok(Self::Not),
            11 => Ok(Self::Equal),
            12 => Ok(Self::Greater),
            13 => Ok(Self::Less),
            _ => Err(UnknownOpcode(byte)),
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return => write!(f, "OP_RETURN"),
            Self::Constant => write!(f, "OP_CONSTANT"),
            Self::Negate => write!(f, "OP_NEGATE"),
            Self::Add => write!(f, "OP_ADD"),
            Self::Subtract => write!(f, "OP_SUBTRACT"),
            Self::Multiply => write!(f, "OP_MULTIPLY"),
            Self::Divide => write!(f, "OP_DIVIDE"),
            Self::Nil => write!(f, "OP_NIL"),
            Self::True => write!(f, "OP_TRUE"),
            Self::False => write!(f, "OP_FALSE"),
            Self::Not => write!(f, "OP_NOT"),
            Self::Equal => write!(f, "OP_EQUAL"),
            Self::Greater => write!(f, "OP_GREATER"),
            Self::Less => write!(f, "OP_LESS"),
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
        assert_eq!(format!("{}", OpCode::Negate), "OP_NEGATE");
        assert_eq!(format!("{}", OpCode::Add), "OP_ADD");
        assert_eq!(format!("{}", OpCode::Subtract), "OP_SUBTRACT");
        assert_eq!(format!("{}", OpCode::Multiply), "OP_MULTIPLY");
        assert_eq!(format!("{}", OpCode::Divide), "OP_DIVIDE");
        assert_eq!(format!("{}", OpCode::Nil), "OP_NIL");
        assert_eq!(format!("{}", OpCode::True), "OP_TRUE");
        assert_eq!(format!("{}", OpCode::False), "OP_FALSE");
        assert_eq!(format!("{}", OpCode::Not), "OP_NOT");
        assert_eq!(format!("{}", OpCode::Equal), "OP_EQUAL");
        assert_eq!(format!("{}", OpCode::Greater), "OP_GREATER");
        assert_eq!(format!("{}", OpCode::Less), "OP_LESS");
    }

    #[test]
    fn test_from_u8_valid() {
        assert!(matches!(OpCode::try_from(0), Ok(OpCode::Return)));
        assert!(matches!(OpCode::try_from(1), Ok(OpCode::Constant)));
        assert!(matches!(OpCode::try_from(2), Ok(OpCode::Negate)));
        assert!(matches!(OpCode::try_from(3), Ok(OpCode::Add)));
        assert!(matches!(OpCode::try_from(4), Ok(OpCode::Subtract)));
        assert!(matches!(OpCode::try_from(5), Ok(OpCode::Multiply)));
        assert!(matches!(OpCode::try_from(6), Ok(OpCode::Divide)));
        assert!(matches!(OpCode::try_from(7), Ok(OpCode::Nil)));
        assert!(matches!(OpCode::try_from(13), Ok(OpCode::Less)));
    }

    proptest! {
        #[test]
        fn prop_opcode_conversion(byte in 0u8..=255) {
            match byte {
                0 ..= 13 => prop_assert!(OpCode::try_from(byte).is_ok()),
                _ => prop_assert!(OpCode::try_from(byte).is_err()),
            }
        }
    }
}
