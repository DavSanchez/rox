use thiserror::Error;

use crate::vm::opcode::UnknownOpcode;

#[derive(Debug, Error)]
pub enum InterpretError {
    #[error("Compile error: {0}")]
    Compile(#[from] CompileError),

    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    UnknownOpcode(UnknownOpcode),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Chunk is malformed")]
    MalformedChunk,
}
