use thiserror::Error;

use crate::vm::opcode::UnknownOpcode;

#[derive(Debug, Error)]
pub enum InterpretError {
    #[error("Compile error: TODO")]
    Compile(#[from] CompileError),

    #[allow(dead_code)]
    #[error("Runtime error: TODO")]
    Runtime,
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    UnknownOpcode(UnknownOpcode),
}
