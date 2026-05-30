use thiserror::Error;

use crate::compiler::scanner::ScanError;
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
    UnknownOpcode(UnknownOpcode),

    #[error(transparent)]
    Scan(#[from] ScanError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum RuntimeError {}

impl From<crate::compiler::CompileError> for CompileError {
    fn from(err: crate::compiler::CompileError) -> Self {
        match err {
            crate::compiler::CompileError::Scan(e) => CompileError::Scan(e),
            crate::compiler::CompileError::Io(e) => CompileError::Io(e),
        }
    }
}
