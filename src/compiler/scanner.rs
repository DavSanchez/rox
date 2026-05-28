use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub start: &'a str,
    pub line: usize,
}

#[derive(Debug, Error)]
#[error("[line {line}] Error: {message}")]
pub struct ScanError {
    pub message: &'static str,
    pub line: usize,
}

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> u8 {
        let c = self.source.as_bytes()[self.current];
        self.current += 1;
        c
    }

    fn make_token(&self, token_type: TokenType) -> Token<'a> {
        Token {
            token_type,
            start: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error(&self, message: &'static str) -> ScanError {
        ScanError {
            message,
            line: self.line,
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token<'a>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.start = self.current;

        if self.is_at_end() {
            return None;
        }

        self.advance();
        Some(Err(self.error("Unexpected character.")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_yields_no_tokens() {
        let mut scanner = Scanner::new("");
        assert!(scanner.next().is_none());
    }

    #[test]
    fn unrecognized_character_produces_error() {
        let mut scanner = Scanner::new("@");
        let result = scanner.next();
        assert!(matches!(result, Some(Err(_))));
    }

    #[test]
    fn error_includes_line_number() {
        let mut scanner = Scanner::new("a");
        if let Some(Err(e)) = scanner.next() {
            assert_eq!(e.line, 1);
            assert_eq!(e.message, "Unexpected character.");
        } else {
            panic!("Expected error");
        }
    }

    #[test]
    fn multiple_unrecognized_characters_produce_multiple_errors() {
        let scanner = Scanner::new("abc");
        let results: Vec<_> = scanner.collect();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_err()));
    }

    #[test]
    fn error_display_format() {
        let err = ScanError {
            message: "Unexpected character.",
            line: 42,
        };
        assert_eq!(format!("{}", err), "[line 42] Error: Unexpected character.");
    }

    #[test]
    fn iterator_can_be_used_with_try_for_each() {
        let mut scanner = Scanner::new("");
        let result: Result<(), ScanError> = scanner.try_for_each(|_| Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    fn iterator_propagates_errors_with_try_for_each() {
        let mut scanner = Scanner::new("abc");
        let mut count = 0;
        let result: Result<(), ScanError> = scanner.try_for_each(|item| {
            count += 1;
            item?;
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(count, 1);
    }
}
