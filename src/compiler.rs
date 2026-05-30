pub mod scanner;

use std::io::{self, Write};

use scanner::{ScanError, Scanner};

pub fn compile<W: Write>(source: &str, w: &mut W) -> Result<(), CompileError> {
    let mut scanner = Scanner::new(source);
    let mut line = 0;

    while let Some(result) = scanner.scan_token() {
        let token = result?;
        if token.line != line {
            write!(w, "{:4} ", token.line)?;
            line = token.line;
        } else {
            write!(w, "   | ")?;
        }
        writeln!(w, "{:2} '{}'", token.token_type as u8, token.start)?;
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_expression() {
        let source = "print 1 + 2;";
        let mut output = Vec::new();
        compile(source, &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let expected = "   1 31 'print'\n   | 21 '1'\n   |  7 '+'\n   | 21 '2'\n   |  8 ';'\n";
        assert_eq!(output_str, expected);
    }
}
