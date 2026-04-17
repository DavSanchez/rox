mod array;
pub mod chunk;
pub mod disassembler;
mod error;
pub mod opcode;
mod stack;
mod value;

use std::ops::{Add, Div, Mul, Neg, Sub};

use chunk::Chunk;
use error::{CompileError, InterpretError};
use opcode::OpCode;
use stack::ValueStack;
use value::Value;

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
                    self.interpret_return();
                    break Ok(());
                }
                OpCode::Negate => self.interpret_negate(),
                OpCode::Constant => self.interpret_constant(&mut instruction_pointer, chunk),
                OpCode::Add => self.interpret_binary_op(Value::add),
                OpCode::Subtract => self.interpret_binary_op(Value::sub),
                OpCode::Multiply => self.interpret_binary_op(Value::mul),
                OpCode::Divide => self.interpret_binary_op(Value::div),
            }
            instruction_pointer += 1;
        }
    }

    fn interpret_return(&mut self) {
        let value = self.stack.pop();
        println!("{value}");
    }

    fn interpret_negate(&mut self) {
        let value = self.stack.pop();
        let negated = Value::from(f64::from(value).neg());
        self.stack.push(negated);
    }

    fn interpret_constant(&mut self, instruction_pointer: &mut usize, chunk: &Chunk) {
        // Increment to get constant offset
        *instruction_pointer += 1;
        let constant_index = chunk.codes[*instruction_pointer] as usize;
        let constant_value = chunk.constants[constant_index];
        self.stack.push(constant_value);
    }

    fn interpret_binary_op(&mut self, op: impl Fn(Value, Value) -> Value) {
        let v2 = self.stack.pop();
        let v1 = self.stack.pop();
        self.stack.push(op(v1, v2));
    }
}
