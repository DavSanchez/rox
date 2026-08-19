use thiserror::Error;

use crate::vm::opcode::UnknownOpcode;

#[derive(Debug, Error)]
pub enum RoxError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Compile(#[from] CompileError),

    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    UnknownOpcode(#[from] UnknownOpcode),

    #[error(transparent)]
    Compiler(#[from] crate::compiler::CompileError),
}

#[derive(Debug, Error)]
#[error("{message}\n[line {line}] in script")]
pub struct RuntimeError {
    pub message: &'static str,
    pub line: usize,
}

impl From<crate::compiler::CompileError> for RoxError {
    fn from(err: crate::compiler::CompileError) -> Self {
        RoxError::Compile(CompileError::Compiler(err))
    }
}
