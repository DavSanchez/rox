use std::fmt;

use thiserror::Error;

use crate::vm::chunk::Chunk;
use crate::vm::opcode::OpCode;
use crate::vm::value::Value;

use super::codegen;
use super::scanner::{ScanError, Scanner, Token, TokenType};

const NUM_TOKEN_TYPES: usize = 39;

#[derive(Debug, Clone, Error)]
#[error("[line {line}] {message}")]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

type ParseFn = for<'src> fn(&mut Parser<'src>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call | Self::Primary => Self::Primary,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
    pub precedence: Precedence,
}

const fn rules() -> [ParseRule; NUM_TOKEN_TYPES] {
    [
        // LeftParen
        ParseRule {
            prefix: Some(grouping),
            infix: None,
            precedence: Precedence::None,
        },
        // RightParen
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // LeftBrace
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // RightBrace
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Comma
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Dot
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Minus
        ParseRule {
            prefix: Some(unary),
            infix: Some(binary),
            precedence: Precedence::Term,
        },
        // Plus
        ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Term,
        },
        // Semicolon
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Slash
        ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Factor,
        },
        // Star
        ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Factor,
        },
        // Bang
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // BangEqual
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Equal
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // EqualEqual
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Greater
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // GreaterEqual
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Less
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // LessEqual
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Identifier
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // String
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Number
        ParseRule {
            prefix: Some(number),
            infix: None,
            precedence: Precedence::None,
        },
        // And
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Class
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Else
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // False
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // For
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Fun
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // If
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Nil
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Or
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Print
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Return
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Super
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // This
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // True
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Var
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // While
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        // Eof
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    ]
}

pub struct Parser<'src> {
    scanner: Scanner<'src>,
    current: Token<'src>,
    previous: Token<'src>,
    had_error: bool,
    panic_mode: bool,
    errors: Vec<ParseError>,
    chunk: Chunk,
    rules: [ParseRule; NUM_TOKEN_TYPES],
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        let eof = Token {
            token_type: TokenType::Eof,
            start: "",
            line: 1,
        };
        Self {
            scanner: Scanner::new(source),
            current: eof,
            previous: eof,
            had_error: false,
            panic_mode: false,
            errors: Vec::new(),
            chunk: Chunk::default(),
            rules: rules(),
        }
    }

    pub fn compile(mut self) -> Result<Chunk, Vec<ParseError>> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();

        if self.had_error {
            Err(self.errors)
        } else {
            Ok(self.chunk)
        }
    }

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            match self.scanner.scan_token() {
                Some(Ok(token)) => {
                    self.current = token;
                    break;
                }
                Some(Err(err)) => {
                    self.report_scan_error(err);
                }
                None => {
                    self.current = Token {
                        token_type: TokenType::Eof,
                        start: "",
                        line: self.previous.line,
                    };
                    break;
                }
            }
        }
    }

    fn consume(&mut self, tt: TokenType, message: &'static str) {
        if self.current.token_type == tt {
            self.advance()
        } else {
            self.error_at_current(message)
        }
    }

    fn error_at(&mut self, token: &Token<'_>, message: &'static str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        let location = match token.token_type {
            TokenType::Eof => " at end".to_string(),
            _ => format!(" at '{}'", token.start),
        };
        let text = format!("Error{location}: {message}");

        eprintln!("[line {}] {text}", token.line);
        self.errors.push(ParseError {
            line: token.line,
            message: text,
        });
        self.had_error = true;
    }

    fn error(&mut self, message: &'static str) {
        let token = self.previous;
        self.error_at(&token, message);
    }

    fn error_at_current(&mut self, message: &'static str) {
        let token = self.current;
        self.error_at(&token, message);
    }

    fn report_scan_error(&mut self, err: ScanError) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        let text = format!("Error: {}", err.message);
        eprintln!("[line {}] {text}", err.line);
        self.errors.push(ParseError {
            line: err.line,
            message: text,
        });
        self.had_error = true;
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.line;
        codegen::emit_byte(&mut self.chunk, byte, line);
    }

    fn emit_return(&mut self) {
        let line = self.previous.line;
        codegen::emit_return(&mut self.chunk, line);
    }

    fn emit_constant(&mut self, value: Value) {
        let line = self.previous.line;
        match codegen::make_constant(&mut self.chunk, value) {
            Ok(idx) => {
                codegen::emit_bytes(&mut self.chunk, OpCode::Constant as u8, idx, line);
            }
            Err(_) => {
                self.error("Too many constants in one chunk.");
            }
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let Some(prefix_rule) = self.rules[self.previous.token_type as usize].prefix else {
            self.error("Expect expression.");
            return;
        };
        prefix_rule(self);

        while precedence <= self.rules[self.current.token_type as usize].precedence {
            self.advance();
            let infix_rule = self.rules[self.previous.token_type as usize].infix;
            if let Some(infix_rule) = infix_rule {
                infix_rule(self);
            }
        }
    }
}

fn number<'src>(parser: &mut Parser<'src>) {
    let value: f64 = parser.previous.start.parse().unwrap_or_default();
    parser.emit_constant(value.into());
}

fn grouping<'src>(parser: &mut Parser<'src>) {
    parser.expression();
    parser.consume(TokenType::RightParen, "Expect ')' after expression.");
}

fn unary<'src>(parser: &mut Parser<'src>) {
    let operator_type = parser.previous.token_type;

    parser.parse_precedence(Precedence::Unary);

    if operator_type == TokenType::Minus {
        parser.emit_byte(OpCode::Negate as u8);
    }
}

fn binary<'src>(parser: &mut Parser<'src>) {
    let operator_type = parser.previous.token_type;
    let rule = parser.rules[operator_type as usize];
    parser.parse_precedence(Precedence::next(rule.precedence));

    match operator_type {
        TokenType::Plus => parser.emit_byte(OpCode::Add as u8),
        TokenType::Minus => parser.emit_byte(OpCode::Subtract as u8),
        TokenType::Star => parser.emit_byte(OpCode::Multiply as u8),
        TokenType::Slash => parser.emit_byte(OpCode::Divide as u8),
        _ => {}
    }
}

impl fmt::Debug for Parser<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parser")
            .field("current", &self.current)
            .field("previous", &self.previous)
            .field("had_error", &self.had_error)
            .field("panic_mode", &self.panic_mode)
            .field("errors", &self.errors)
            .finish_non_exhaustive()
    }
}
