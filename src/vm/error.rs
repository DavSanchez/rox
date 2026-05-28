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
}

#[derive(Debug, Error)]
pub enum RuntimeError {}
