use crate::vm::chunk::{Chunk, ExceededConstantCount};
use crate::vm::opcode::OpCode;
use crate::vm::value::Value;

pub fn emit_byte(chunk: &mut Chunk, byte: u8, line: usize) {
    chunk.write_byte(byte, line);
}

pub fn emit_bytes(chunk: &mut Chunk, b1: u8, b2: u8, line: usize) {
    emit_byte(chunk, b1, line);
    emit_byte(chunk, b2, line);
}

pub fn emit_return(chunk: &mut Chunk, line: usize) {
    emit_byte(chunk, OpCode::Return as u8, line);
}

pub fn make_constant(chunk: &mut Chunk, value: Value) -> Result<u8, ExceededConstantCount> {
    chunk.write_constant(value)
}
