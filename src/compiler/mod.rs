pub mod scanner;

use scanner::{Scanner, TokenType};

pub fn compile(source: &str) {
    let mut scanner = Scanner::new(source);

    let mut line = 0;
    loop {
        let token = scanner.scan_token();
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        println!("{:2} '{}'", token.token_type as u8, token.start);

        if token.token_type == TokenType::Eof {
            break;
        }
    }
}
