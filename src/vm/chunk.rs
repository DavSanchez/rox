use std::num::TryFromIntError;

use thiserror::Error;

use super::array::Array;
use super::opcode::OpCode;
use super::value::Value;

pub struct Chunk {
    pub codes: Array<u8>,
    pub lines: Array<usize>,
    pub constants: Array<Value>,
}

#[derive(Debug, Error)]
#[error("Exceeded constant count. Maximum is 256.")]
pub struct ExceededConstantCount(#[from] TryFromIntError);

impl Chunk {
    pub fn new() -> Self {
        Self {
            codes: Array::new(),
            lines: Array::new(),
            constants: Array::new(),
        }
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write_byte(opcode as u8, line);
    }

    pub fn write_byte(&mut self, byte: u8, line: usize) {
        // Writing a byte also records the line number,
        // ensuring that both grow together.
        self.codes.push(byte);
        self.lines.push(line);
    }

    pub fn write_constant(&mut self, value: Value) -> Result<u8, ExceededConstantCount> {
        // We must return an u8 because this is the operand size for OP_CONSTANT.
        // so we cannot actually have more than 256 constants for this VM, for now!
        let index = self.constants.count();
        // We stop the VM if we exceed 256 constants.
        let safe_index = u8::try_from(index)?;
        // Otherwise we continue
        self.constants.push(value);
        Ok(safe_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_write_opcode() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::Return, 123);
        assert_eq!(chunk.codes.count(), 1);
        assert_eq!(chunk.lines.count(), 1);
        assert_eq!(chunk.codes[0], OpCode::Return as u8);
        assert_eq!(chunk.lines[0], 123);
    }

    #[test]
    fn test_constant_limit() {
        let mut chunk = Chunk::new();
        for i in 0..256 {
            assert!(chunk.write_constant(i as f64).is_ok());
        }
        assert!(chunk.write_constant(1.0).is_err());
    }

    proptest! {
        #[test]
        fn prop_lines_sync(ops in proptest::collection::vec(any::<u8>(), 0..100)) {
            let mut chunk = Chunk::new();
            for (i, op) in ops.iter().enumerate() {
                chunk.write_byte(*op, i);
            }
            prop_assert_eq!(chunk.codes.count(), chunk.lines.count());
            prop_assert_eq!(chunk.codes.count(), ops.len());
        }
    }
}
