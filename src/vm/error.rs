use thiserror::Error;

use crate::compiler::ParseError;
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
    UnknownOpcode(#[from] UnknownOpcode),

    #[error(transparent)]
    Scan(#[from] ScanError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Parse(#[from] ParseErrorReport),
}

#[derive(Debug, Error)]
#[error("Parse error(s):\n{}", format_parse_errors(.0))]
pub struct ParseErrorReport(pub Vec<ParseError>);

fn format_parse_errors(errors: &[ParseError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Error)]
pub enum RuntimeError {}

impl From<crate::compiler::CompileError> for RoxError {
    fn from(err: crate::compiler::CompileError) -> Self {
        RoxError::Compile(err.into())
    }
}

impl From<crate::compiler::CompileError> for CompileError {
    fn from(err: crate::compiler::CompileError) -> Self {
        match err {
            crate::compiler::CompileError::Scan(e) => CompileError::Scan(e),
            crate::compiler::CompileError::Io(e) => CompileError::Io(e),
            crate::compiler::CompileError::Parse(errors) => {
                CompileError::Parse(ParseErrorReport(errors))
            }
        }
    }
}
