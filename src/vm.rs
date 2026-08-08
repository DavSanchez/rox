pub mod chunk;
pub mod disassembler;
pub mod error;
pub mod opcode;
mod stack;
pub mod value;

use chunk::Chunk;
use error::{CompileError, RoxError, RuntimeError};
use opcode::OpCode;
use stack::ValueStack;
use std::io::{self, Stdout, Write};
use value::Value;

use crate::compiler;

pub struct Vm<W: Write = Stdout> {
    stack: ValueStack,
    output: W,
}

impl Default for Vm<Stdout> {
    fn default() -> Self {
        Self {
            stack: ValueStack::default(),
            output: io::stdout(),
        }
    }
}

impl<W: Write> std::fmt::Debug for Vm<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vm")
            .field("stack", &self.stack)
            .finish_non_exhaustive()
    }
}

impl<W: Write> Vm<W> {
    #[cfg(test)]
    pub fn with_output(output: W) -> Self {
        Self {
            stack: ValueStack::default(),
            output,
        }
    }

    #[cfg(test)]
    pub fn into_output(self) -> W {
        self.output
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), RoxError> {
        let chunk = compiler::compile(source)?;
        self.run(&chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> Result<(), RoxError> {
        let mut instruction_pointer = 0usize;

        loop {
            let code_u8 = chunk.codes[instruction_pointer];
            let opcode = OpCode::try_from(code_u8).map_err(CompileError::UnknownOpcode)?;

            match opcode {
                OpCode::Return => {
                    self.interpret_return();
                    break Ok(());
                }
                OpCode::Negate => self.interpret_negate(chunk, instruction_pointer)?,
                OpCode::Constant => self.interpret_constant(&mut instruction_pointer, chunk),
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Not => {
                    let value = self.stack.pop();
                    self.stack.push(Value::Bool(value.is_falsey()));
                }
                OpCode::Equal => {
                    let right = self.stack.pop();
                    let left = self.stack.pop();
                    self.stack.push(Value::Bool(left.values_equal(right)));
                }
                OpCode::Greater => {
                    self.interpret_binary_op(chunk, instruction_pointer, |left, right| {
                        Value::Bool(left > right)
                    })?;
                }
                OpCode::Less => {
                    self.interpret_binary_op(chunk, instruction_pointer, |left, right| {
                        Value::Bool(left < right)
                    })?;
                }
                OpCode::Add => {
                    self.interpret_binary_op(chunk, instruction_pointer, |left, right| {
                        Value::Number(left + right)
                    })?;
                }
                OpCode::Subtract => {
                    self.interpret_binary_op(chunk, instruction_pointer, |left, right| {
                        Value::Number(left - right)
                    })?;
                }
                OpCode::Multiply => {
                    self.interpret_binary_op(chunk, instruction_pointer, |left, right| {
                        Value::Number(left * right)
                    })?;
                }
                OpCode::Divide => {
                    self.interpret_binary_op(chunk, instruction_pointer, |left, right| {
                        Value::Number(left / right)
                    })?;
                }
            }
            instruction_pointer += 1;
        }
    }

    fn interpret_return(&mut self) {
        let value = self.stack.pop();
        let _ = writeln!(self.output, "{value}");
    }

    fn interpret_negate(
        &mut self,
        chunk: &Chunk,
        instruction_pointer: usize,
    ) -> Result<(), RoxError> {
        let value = self.stack.peek(0);
        let Some(number) = value.as_number() else {
            return Err(self.runtime_error(
                chunk,
                instruction_pointer,
                "Operand must be a number.",
            ));
        };
        self.stack.pop();
        self.stack.push(Value::Number(-number));
        Ok(())
    }

    fn interpret_constant(&mut self, instruction_pointer: &mut usize, chunk: &Chunk) {
        // Increment to get constant offset
        *instruction_pointer += 1;
        let constant_index = chunk.codes[*instruction_pointer] as usize;
        let constant_value = chunk.constants[constant_index];
        self.stack.push(constant_value);
    }

    fn interpret_binary_op(
        &mut self,
        chunk: &Chunk,
        instruction_pointer: usize,
        op: impl FnOnce(f64, f64) -> Value,
    ) -> Result<(), RoxError> {
        let right = self.stack.peek(0);
        let left = self.stack.peek(1);
        let (Some(left), Some(right)) = (left.as_number(), right.as_number()) else {
            return Err(self.runtime_error(
                chunk,
                instruction_pointer,
                "Operands must be numbers.",
            ));
        };
        self.stack.pop();
        self.stack.pop();
        self.stack.push(op(left, right));
        Ok(())
    }

    fn runtime_error(
        &mut self,
        chunk: &Chunk,
        instruction_pointer: usize,
        message: &'static str,
    ) -> RoxError {
        self.stack.reset();
        RuntimeError {
            message,
            line: chunk.lines[instruction_pointer],
        }
        .into()
    }
}
