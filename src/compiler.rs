pub mod scanner;

use scanner::{ScanError, Scanner};

pub fn compile(source: &str) -> Result<(), ScanError> {
    let scanner = Scanner::new(source);

    let mut line = 0;
    scanner.into_iter().try_for_each(|result| {
        let token = result?;
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ")
        }
        println!("{:2} '{}'", token.token_type as u8, token.start);
        Ok(())
    })
}
