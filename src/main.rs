mod compiler;
mod vm;

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use vm::Vm;
use vm::error::RoxError;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let mut vm = Vm::default();

    let result = match args.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, &args[1]),
        _ => {
            eprintln!("Usage: rox [path]");
            return ExitCode::from(64);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(RoxError::Io(e)) => {
            eprintln!("{e}");
            ExitCode::from(74)
        }
        Err(RoxError::Compile(e)) => {
            eprintln!("{e}");
            ExitCode::from(65)
        }
        Err(RoxError::Runtime(e)) => {
            eprintln!("{e}");
            ExitCode::from(70)
        }
    }
}

fn repl(vm: &mut Vm) -> Result<(), RoxError> {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            println!();
            break;
        }

        vm.interpret(&line)?;
    }
    Ok(())
}

fn run_file(vm: &mut Vm, path: &str) -> Result<(), RoxError> {
    let source = std::fs::read_to_string(path)?;
    vm.interpret(&source)?;
    Ok(())
}
