mod array;
pub mod chunk;
pub mod disassembler;
mod error;
pub mod opcode;
mod stack;
mod value;

use std::ops::Neg;

use chunk::Chunk;
use error::{CompileError, InterpretError};
use opcode::OpCode;
use stack::ValueStack;
use value::Value;

use crate::vm::error::RuntimeError;

#[derive(Debug, Default)]
pub struct Vm {
    stack: ValueStack,
}

impl Vm {
    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
        let mut instruction_pointer = 0usize;

        loop {
            let code_u8 = chunk.codes[instruction_pointer];
            let opcode = OpCode::try_from(code_u8).map_err(CompileError::UnknownOpcode)?;

            match opcode {
                OpCode::Return => {
                    let value = self.stack.pop();
                    println!("{value}");
                    break Ok(());
                }
                OpCode::Negate => {
                    let value = self.stack.pop();
                    let negated = Value::from(f64::from(value).neg());
                    self.stack.push(negated);
                }
                OpCode::Constant => {
                    // Increment to get constant offset
                    instruction_pointer += 1;
                    let constant_index = chunk
                        .codes
                        .get(instruction_pointer)
                        .ok_or(RuntimeError::MalformedChunk)?;
                    let constant_value = chunk
                        .constants
                        .get(*constant_index as usize)
                        .ok_or(RuntimeError::MalformedChunk)?;
                    self.stack.push(*constant_value);
                }
            }
            instruction_pointer += 1;
        }
    }
}
