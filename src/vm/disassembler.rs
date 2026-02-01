use std::io::{self, Write};

use super::chunk::Chunk;
use super::opcode::{OpCode, UnknownOpcode};
use thiserror::Error;

pub struct Disassembler<'a> {
    chunk: &'a Chunk,
    name: &'a str,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk, name: &'a str) -> Self {
        Self { chunk, name }
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), DisassembleError> {
        writeln!(w, "== {} ==", self.name)?;

        let mut offset = 0;
        while offset < self.chunk.codes.count() {
            offset = self.disassemble_instruction(w, offset)?;
        }
        Ok(())
    }

    fn disassemble_instruction<W: Write>(
        &self,
        w: &mut W,
        offset: usize,
    ) -> Result<usize, DisassembleError> {
        let opcode = self.chunk.codes[offset];
        let opcode_enum = OpCode::try_from(opcode)?;

        // Write offset
        write!(w, "{offset:04} ")?;
        // Write line number or trailing pipe for same line
        if offset > 0 && self.chunk.lines[offset] == self.chunk.lines[offset - 1] {
            write!(w, "   | ")?;
        } else {
            write!(w, "{:4} ", self.chunk.lines[offset])?;
        }

        // Write actual instruction
        match opcode_enum {
            OpCode::Constant => {
                let constant_index = self.chunk.codes[offset + 1];
                let constant_value = &self.chunk.constants[constant_index as usize];
                writeln!(w, "{opcode_enum:-16} {constant_index:4} '{constant_value}'")?;
                Ok(offset + 2)
            }
            opcode => {
                writeln!(w, "{opcode}")?;
                Ok(offset + 1)
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum DisassembleError {
    #[error(transparent)]
    UnknownOpcode(#[from] UnknownOpcode),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::opcode::OpCode;

    #[test]
    fn test_disassemble_return() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::Return, 123);

        let disassembler = Disassembler::new(&chunk, "test");
        let mut buffer = Vec::new();
        disassembler.write(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("123"));
        assert!(output.contains("OP_RETURN"));
        assert!(output.contains("=="));
    }

    #[test]
    fn test_disassemble_constant() {
        let mut chunk = Chunk::new();
        let idx = chunk.write_constant(42.0).unwrap();
        chunk.write_opcode(OpCode::Constant, 100);
        chunk.write_byte(idx, 100);

        let disassembler = Disassembler::new(&chunk, "constant test");
        let mut buffer = Vec::new();
        disassembler.write(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("OP_CONSTANT"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_disassemble_multiple_instructions() {
        let mut chunk = Chunk::new();

        // Line 1: Constant 1.2
        let constant_idx = chunk.write_constant(1.2).unwrap();
        chunk.write_opcode(OpCode::Constant, 1);
        chunk.write_byte(constant_idx, 1);

        // Line 1: Return (should show |)
        chunk.write_opcode(OpCode::Return, 1);

        // Line 2: Return (should show 2)
        chunk.write_opcode(OpCode::Return, 2);

        let disassembler = Disassembler::new(&chunk, "test chunk");
        let mut buffer = Vec::new();
        disassembler.write(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines[0], "== test chunk ==");

        // 0000    1 OP_CONSTANT         0 '1.2'
        assert!(lines[1].contains("0000"));
        assert!(lines[1].contains("1 OP_CONSTANT"));
        assert!(lines[1].contains("1.2"));

        // 0002    | OP_RETURN
        assert!(lines[2].contains("0002"));
        assert!(lines[2].contains("| OP_RETURN"));

        // 0003    2 OP_RETURN
        assert!(lines[3].contains("0003"));
        assert!(lines[3].contains("2 OP_RETURN"));
    }
}
