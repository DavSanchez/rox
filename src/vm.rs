mod array;
pub mod chunk;
pub mod disassembler;
pub mod error;
pub mod opcode;
mod stack;
pub mod value;

use std::io::{self, Stdout, Write};
use std::ops::{Add, Div, Mul, Neg, Sub};

use chunk::Chunk;
use error::{CompileError, RoxError};
use opcode::OpCode;
use stack::ValueStack;
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

#[allow(dead_code)]
impl<W: Write> Vm<W> {
    pub fn with_output(output: W) -> Self {
        Self {
            stack: ValueStack::default(),
            output,
        }
    }

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
        let _ = writeln!(self.output, "{value}");
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

#[allow(dead_code)]
pub fn default_output() -> Stdout {
    io::stdout()
}
