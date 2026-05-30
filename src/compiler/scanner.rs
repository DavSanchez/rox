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

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            0
        } else {
            self.source.as_bytes()[self.current]
        }
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            0
        } else {
            self.source.as_bytes()[self.current + 1]
        }
    }

    fn match_char(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
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

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Result<Token<'a>, ScanError> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(self.error("Unterminated string."));
        }

        self.advance();
        Ok(self.make_token(TokenType::String))
    }

    fn identifier(&mut self) -> Token<'a> {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        let len = self.current - self.start;
        let bytes = self.source.as_bytes();

        if len == 0 {
            return TokenType::Identifier;
        }

        match bytes[self.start] {
            b'a' if len == 3 && self.check_keyword(1, b"nd") => TokenType::And,
            b'c' if len == 5 && self.check_keyword(1, b"lass") => TokenType::Class,
            b'e' if len == 4 && self.check_keyword(1, b"lse") => TokenType::Else,
            b'i' if len == 2 && self.check_keyword(1, b"f") => TokenType::If,
            b'n' if len == 3 && self.check_keyword(1, b"il") => TokenType::Nil,
            b'o' if len == 2 && self.check_keyword(1, b"r") => TokenType::Or,
            b'p' if len == 5 && self.check_keyword(1, b"rint") => TokenType::Print,
            b'r' if len == 6 && self.check_keyword(1, b"eturn") => TokenType::Return,
            b's' if len == 5 && self.check_keyword(1, b"uper") => TokenType::Super,
            b'v' if len == 3 && self.check_keyword(1, b"ar") => TokenType::Var,
            b'w' if len == 5 && self.check_keyword(1, b"hile") => TokenType::While,
            b'f' if len >= 2 => match bytes[self.start + 1] {
                b'a' if len == 5 && self.check_keyword(2, b"lse") => TokenType::False,
                b'o' if len == 3 && self.check_keyword(2, b"r") => TokenType::For,
                b'u' if len == 3 && self.check_keyword(2, b"n") => TokenType::Fun,
                _ => TokenType::Identifier,
            },
            b't' if len >= 2 => match bytes[self.start + 1] {
                b'h' if len == 4 && self.check_keyword(2, b"is") => TokenType::This,
                b'r' if len == 4 && self.check_keyword(2, b"ue") => TokenType::True,
                _ => TokenType::Identifier,
            },
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, rest: &[u8]) -> bool {
        let bytes = self.source.as_bytes();
        let keyword_start = self.start + start;
        let keyword_end = keyword_start + rest.len();

        if keyword_end > self.current {
            return false;
        }

        &bytes[keyword_start..keyword_end] == rest
    }

    fn number(&mut self) -> Token<'a> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' && is_digit(self.peek_next()) {
            self.advance();
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }
}

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_alpha(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token<'a>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return None;
        }

        let c = self.advance();

        let token_type = match c {
            b'(' => TokenType::LeftParen,
            b')' => TokenType::RightParen,
            b'{' => TokenType::LeftBrace,
            b'}' => TokenType::RightBrace,
            b';' => TokenType::Semicolon,
            b',' => TokenType::Comma,
            b'.' => TokenType::Dot,
            b'-' => TokenType::Minus,
            b'+' => TokenType::Plus,
            b'/' => TokenType::Slash,
            b'*' => TokenType::Star,
            b'!' => {
                if self.match_char(b'=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            b'=' => {
                if self.match_char(b'=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            b'<' => {
                if self.match_char(b'=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            b'>' => {
                if self.match_char(b'=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            b'"' => return Some(self.string()),
            _ if is_digit(c) => return Some(Ok(self.number())),
            _ if is_alpha(c) => return Some(Ok(self.identifier())),
            _ => return Some(Err(self.error("Unexpected character."))),
        };

        Some(Ok(self.make_token(token_type)))
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
    fn whitespace_only_yields_no_tokens() {
        let mut scanner = Scanner::new("   \n\t  \n");
        assert!(scanner.next().is_none());
    }

    #[test]
    fn comment_is_skipped() {
        let mut scanner = Scanner::new("// this is a comment\n42");
        let token = scanner.next().unwrap().unwrap();
        assert_eq!(token.token_type, TokenType::Number);
        assert_eq!(token.start, "42");
        assert_eq!(token.line, 2);
    }

    #[test]
    fn single_char_tokens() {
        let mut scanner = Scanner::new("(){},.-+/*;");
        let expected = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Slash,
            TokenType::Star,
            TokenType::Semicolon,
        ];
        for expected_type in expected {
            let token = scanner.next().unwrap().unwrap();
            assert_eq!(token.token_type, expected_type);
        }
        assert!(scanner.next().is_none());
    }

    #[test]
    fn two_char_tokens() {
        let mut scanner = Scanner::new("!= == <= >= ! = < >");
        let expected = vec![
            TokenType::BangEqual,
            TokenType::EqualEqual,
            TokenType::LessEqual,
            TokenType::GreaterEqual,
            TokenType::Bang,
            TokenType::Equal,
            TokenType::Less,
            TokenType::Greater,
        ];
        for expected_type in expected {
            let token = scanner.next().unwrap().unwrap();
            assert_eq!(token.token_type, expected_type);
        }
        assert!(scanner.next().is_none());
    }

    #[test]
    fn number_literal() {
        let mut scanner = Scanner::new("123");
        let token = scanner.next().unwrap().unwrap();
        assert_eq!(token.token_type, TokenType::Number);
        assert_eq!(token.start, "123");
    }

    #[test]
    fn number_with_decimal() {
        let mut scanner = Scanner::new("123.456");
        let token = scanner.next().unwrap().unwrap();
        assert_eq!(token.token_type, TokenType::Number);
        assert_eq!(token.start, "123.456");
    }

    #[test]
    fn number_without_decimal_part() {
        let mut scanner = Scanner::new("123.");
        let token = scanner.next().unwrap().unwrap();
        assert_eq!(token.token_type, TokenType::Number);
        assert_eq!(token.start, "123");
        let next = scanner.next().unwrap().unwrap();
        assert_eq!(next.token_type, TokenType::Dot);
    }

    #[test]
    fn string_literal() {
        let mut scanner = Scanner::new("\"hello\"");
        let token = scanner.next().unwrap().unwrap();
        assert_eq!(token.token_type, TokenType::String);
        assert_eq!(token.start, "\"hello\"");
    }

    #[test]
    fn string_with_newline() {
        let mut scanner = Scanner::new("\"hello\nworld\"");
        let token = scanner.next().unwrap().unwrap();
        assert_eq!(token.token_type, TokenType::String);
        assert_eq!(token.start, "\"hello\nworld\"");
        assert_eq!(token.line, 2);
    }

    #[test]
    fn unterminated_string() {
        let mut scanner = Scanner::new("\"hello");
        let result = scanner.next().unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Unterminated string.");
    }

    #[test]
    fn line_tracking() {
        let mut scanner = Scanner::new("1\n2\n3");
        let t1 = scanner.next().unwrap().unwrap();
        assert_eq!(t1.line, 1);
        let t2 = scanner.next().unwrap().unwrap();
        assert_eq!(t2.line, 2);
        let t3 = scanner.next().unwrap().unwrap();
        assert_eq!(t3.line, 3);
    }

    #[test]
    fn test_identifiers() {
        let scanner = Scanner::new("foo bar_baz _test123");
        let tokens: Vec<_> = scanner.collect();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(
            tokens[0],
            Ok(Token {
                token_type: TokenType::Identifier,
                start: "foo",
                ..
            })
        ));
        assert!(matches!(
            tokens[1],
            Ok(Token {
                token_type: TokenType::Identifier,
                start: "bar_baz",
                ..
            })
        ));
        assert!(matches!(
            tokens[2],
            Ok(Token {
                token_type: TokenType::Identifier,
                start: "_test123",
                ..
            })
        ));
    }

    #[test]
    fn test_keywords() {
        let scanner = Scanner::new(
            "and class else false for fun if nil or print return super this true var while",
        );
        let expected = vec![
            TokenType::And,
            TokenType::Class,
            TokenType::Else,
            TokenType::False,
            TokenType::For,
            TokenType::Fun,
            TokenType::If,
            TokenType::Nil,
            TokenType::Or,
            TokenType::Print,
            TokenType::Return,
            TokenType::Super,
            TokenType::This,
            TokenType::True,
            TokenType::Var,
            TokenType::While,
        ];
        let tokens: Vec<_> = scanner.collect();
        assert_eq!(tokens.len(), expected.len());
        for (token, expected_type) in tokens.iter().zip(expected.iter()) {
            assert!(matches!(token, Ok(Token { token_type, .. }) if token_type == expected_type));
        }
    }

    #[test]
    fn test_keyword_prefixes_are_identifiers() {
        let scanner = Scanner::new("andy classy elseif falsehood format funny");
        let tokens: Vec<_> = scanner.collect();
        assert_eq!(tokens.len(), 6);
        for token in tokens {
            assert!(matches!(
                token,
                Ok(Token {
                    token_type: TokenType::Identifier,
                    ..
                })
            ));
        }
    }

    #[test]
    fn error_includes_line_number() {
        let mut scanner = Scanner::new("@");
        if let Some(Err(e)) = scanner.next() {
            assert_eq!(e.line, 1);
            assert_eq!(e.message, "Unexpected character.");
        } else {
            panic!("Expected error");
        }
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
        let mut scanner = Scanner::new("@@@");
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
