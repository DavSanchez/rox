pub mod scanner;

use std::io::{self, Write};

use scanner::{ScanError, Scanner, Token};

pub fn write_tokens<'a, I, W>(tokens: I, w: &mut W) -> Result<(), CompileError>
where
    I: IntoIterator<Item = Result<Token<'a>, ScanError>>,
    W: Write,
{
    let mut line = 0;
    tokens.into_iter().try_for_each(|result| {
        let token = result?;
        if token.line != line {
            write!(w, "{:4} ", token.line)?;
            line = token.line;
        } else {
            write!(w, "   | ")?;
        }
        writeln!(w, "{:2} '{}'", token.token_type as u8, token.start)?;
        Ok(())
    })
}

pub fn compile<W: Write>(source: &str, w: &mut W) -> Result<(), CompileError> {
    let scanner = Scanner::new(source);
    write_tokens(scanner, w)
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
    use crate::compiler::scanner::TokenType;

    #[test]
    fn test_write_tokens_format() {
        let tokens = vec![
            Ok(Token {
                token_type: TokenType::Print,
                start: "print",
                line: 1,
            }),
            Ok(Token {
                token_type: TokenType::Number,
                start: "1",
                line: 1,
            }),
            Ok(Token {
                token_type: TokenType::Plus,
                start: "+",
                line: 1,
            }),
            Ok(Token {
                token_type: TokenType::Number,
                start: "2",
                line: 1,
            }),
            Ok(Token {
                token_type: TokenType::Semicolon,
                start: ";",
                line: 1,
            }),
            Ok(Token {
                token_type: TokenType::Eof,
                start: "",
                line: 2,
            }),
        ];

        let mut output = Vec::new();
        write_tokens(tokens, &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let expected =
            "   1 31 'print'\n   | 21 '1'\n   |  7 '+'\n   | 21 '2'\n   |  8 ';'\n   2 39 ''\n";
        assert_eq!(output_str, expected);
    }
}
