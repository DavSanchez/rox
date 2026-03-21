mod array;
pub mod chunk;
pub mod disassembler;
mod error;
pub mod opcode;
mod value;

use chunk::Chunk;
use error::{CompileError, InterpretError};
use opcode::OpCode;

#[derive(Debug, Default)]
pub struct Vm {
    chunk: Chunk,
}

impl Vm {
    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), InterpretError> {
        self.chunk = chunk;

        let codes = &self.chunk.codes;

        codes.iter().enumerate().try_for_each(|(i, opcode)| {
            let as_opcode = OpCode::try_from(*opcode).map_err(CompileError::UnknownOpcode)?;

            match as_opcode {
                OpCode::Return => Ok(()),
                OpCode::Constant => {
                    let constant_index = codes[i + 1];
                    let constant_value = &self.chunk.constants[constant_index as usize];
                    println!("{constant_value}");
                    Ok(())
                }
            }
        })
    }
}
