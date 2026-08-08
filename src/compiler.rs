pub mod codegen;
mod parser;
pub mod scanner;

use std::io;

use crate::array::Array;

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
    Parse(Array<ParseError>),
}

fn format_parse_errors(errors: &Array<ParseError>) -> String {
    errors.iter().fold(String::new(), |mut output, error| {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&error.to_string());
        output
    })
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
    use crate::array::Array;
    use crate::vm::Vm;

    fn run_capture(source: &str) -> Array<u8> {
        let mut vm = Vm::with_output(Array::default());
        vm.interpret(source).unwrap();
        vm.into_output()
    }

    #[test]
    fn evaluate_chapter17_official() {
        assert_eq!(&*run_capture("(5 - (3 - 1)) + -1"), b"2\n");
    }

    #[test]
    fn unary_binds_tighter_than_add() {
        assert_eq!(&*run_capture("-1 + 2"), b"1\n");
    }

    #[test]
    fn grouping_respected() {
        assert_eq!(&*run_capture("(1 + 2) * 3"), b"9\n");
    }

    #[test]
    fn left_associative_subtraction() {
        assert_eq!(&*run_capture("10 - 3 - 2"), b"5\n");
    }

    #[test]
    fn division_and_multiplication_same_precedence() {
        assert_eq!(&*run_capture("8 / 2 * 4"), b"16\n");
    }

    #[test]
    fn typed_literals_are_printed() {
        assert_eq!(&*run_capture("true"), b"true\n");
        assert_eq!(&*run_capture("false"), b"false\n");
        assert_eq!(&*run_capture("nil"), b"nil\n");
    }

    #[test]
    fn logical_not_uses_lox_truthiness() {
        assert_eq!(&*run_capture("!true"), b"false\n");
        assert_eq!(&*run_capture("!false"), b"true\n");
        assert_eq!(&*run_capture("!nil"), b"true\n");
        assert_eq!(&*run_capture("!0"), b"false\n");
    }

    #[test]
    fn equality_and_comparisons() {
        assert_eq!(&*run_capture("1 == 1"), b"true\n");
        assert_eq!(&*run_capture("1 != 2"), b"true\n");
        assert_eq!(&*run_capture("nil == nil"), b"true\n");
        assert_eq!(&*run_capture("true == 1"), b"false\n");
        assert_eq!(&*run_capture("1 < 2"), b"true\n");
        assert_eq!(&*run_capture("2 >= 2"), b"true\n");
    }

    #[test]
    fn invalid_numeric_operand_is_runtime_error() {
        let mut vm = Vm::with_output(Array::default());
        let error = vm.interpret("-true").unwrap_err();
        assert_eq!(
            error.to_string(),
            "Operand must be a number.\n[line 1] in script"
        );

        let error = vm.interpret("true + 1").unwrap_err();
        assert_eq!(
            error.to_string(),
            "Operands must be numbers.\n[line 1] in script"
        );
    }

    #[test]
    fn parse_error_carries_line_and_message() {
        let mut vm = Vm::with_output(Array::default());
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
