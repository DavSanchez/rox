mod array;
pub mod chunk;
pub mod disassembler;
mod error;
pub mod opcode;
mod stack;
mod value;

use chunk::Chunk;
use error::{CompileError, InterpretError};
use opcode::OpCode;
use stack::ValueStack;

use crate::vm::error::RuntimeError;

#[derive(Debug, Default)]
pub struct Vm {
    stack: ValueStack,
}

impl Vm {
    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
        chunk.codes.iter().enumerate().try_for_each(|(i, opcode)| {
            let as_opcode = OpCode::try_from(*opcode).map_err(CompileError::UnknownOpcode)?;

            match as_opcode {
                OpCode::Return => {
                    let value = self.stack.pop();
                    println!("{value}");
                    Ok(())
                }
                OpCode::Constant => {
                    let constant_index =
                        *chunk.codes.get(i + 1).ok_or(RuntimeError::MalformedChunk)?;
                    let constant_value = chunk
                        .constants
                        .get(constant_index as usize)
                        .ok_or(RuntimeError::MalformedChunk)?;
                    self.stack.push(*constant_value);
                    Ok(())
                }
            }
        })
    }
}
