pub mod codegen;
mod parser;
pub mod scanner;

use std::io;

pub use parser::ParseError;
use parser::Parser;
use scanner::ScanError;

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Parse error(s):\n{}", format_parse_errors(.0))]
    Parse(Vec<ParseError>),
}

fn format_parse_errors(errors: &[ParseError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn compile(source: &str) -> Result<crate::vm::chunk::Chunk, CompileError> {
    let parser = Parser::new(source);
    match parser.compile() {
        Ok(chunk) => Ok(chunk),
        Err(errors) => Err(CompileError::Parse(errors)),
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::Vm;

    fn run_capture(source: &str) -> String {
        let mut vm = Vm::with_output(Vec::<u8>::new());
        vm.interpret(source).unwrap();
        let output = vm.into_output();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn evaluate_chapter17_official() {
        assert_eq!(run_capture("(5 - (3 - 1)) + -1"), "2\n");
    }

    #[test]
    fn unary_binds_tighter_than_add() {
        assert_eq!(run_capture("-1 + 2"), "1\n");
    }

    #[test]
    fn grouping_respected() {
        assert_eq!(run_capture("(1 + 2) * 3"), "9\n");
    }

    #[test]
    fn left_associative_subtraction() {
        assert_eq!(run_capture("10 - 3 - 2"), "5\n");
    }

    #[test]
    fn division_and_multiplication_same_precedence() {
        assert_eq!(run_capture("8 / 2 * 4"), "16\n");
    }

    #[test]
    fn parse_error_carries_line_and_message() {
        let mut vm = Vm::with_output(Vec::<u8>::new());
        let err = vm.interpret("(1 +").unwrap_err();
        match err {
            crate::vm::error::RoxError::Compile(crate::vm::error::CompileError::Parse(report)) => {
                assert!(!report.0.is_empty());
                let e = &report.0[0];
                assert_eq!(e.line, 1);
                assert!(e.message.contains("Expect expression"));
                assert!(e.to_string().starts_with("[line 1] Error"));
            }
            other => panic!("expected Parse error, got {other:?}"),
        }
    }
}
