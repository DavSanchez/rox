use thiserror::Error;

use crate::array::Array;
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
pub struct ParseErrorReport(pub Array<ParseError>);

fn format_parse_errors(errors: &Array<ParseError>) -> String {
    errors.iter().fold(String::new(), |mut output, error| {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&error.to_string());
        output
    })
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
