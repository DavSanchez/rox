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
